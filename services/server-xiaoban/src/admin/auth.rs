use crate::{
    error::AppError,
    store::{AdminAccount, Store},
};
use argon2::{
    password_hash::{phc::PasswordHash, PasswordHasher, PasswordVerifier},
    Argon2,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};
use tracing::warn;

pub const DEFAULT_USERNAME: &str = "admin";
pub const DEFAULT_PASSWORD: &str = "123456";
pub const SESSION_COOKIE: &str = "admin_session";

const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 128;
const SESSION_IDLE: Duration = Duration::from_secs(4 * 3600);
const SESSION_MAX: Duration = Duration::from_secs(24 * 3600);
pub const SESSION_COOKIE_MAX_AGE: u64 = 24 * 3600;

const FAILURES_BEFORE_LOCK: u32 = 5;
const BASE_LOCK: Duration = Duration::from_secs(60);
const MAX_LOCK: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, Serialize)]
pub struct AccountInfo {
    pub username: String,
    /// True while the account still has the password it was created with (123456).
    pub is_default_password: bool,
}

#[derive(Debug, Clone)]
pub struct AdminSession {
    pub token: String,
    pub username: String,
}

struct SessionEntry {
    username: String,
    created: Instant,
    last_seen: Instant,
}

#[derive(Default)]
struct Throttle {
    failures: u32,
    lock_level: u32,
    locked_until: Option<Instant>,
}

struct Inner {
    store: Option<Store>,
    sessions: Mutex<HashMap<String, SessionEntry>>,
    throttle: Mutex<Throttle>,
}

#[derive(Clone)]
pub struct AdminService {
    inner: Arc<Inner>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash: PasswordHash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Password hashing task failed: {e}")))?
}

async fn verify_password(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || Argon2::default().verify_password(password.as_bytes(), hash.as_str()).is_ok())
        .await
        .unwrap_or(false)
}

fn validate_username(name: &str) -> Result<(), AppError> {
    let valid_chars = name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
    if !(3..=32).contains(&name.chars().count()) || !valid_chars {
        return Err(AppError::BadRequest(
            "用户名需为 3 到 32 位，只能包含字母、数字、下划线、点和短横线".to_string(),
        ));
    }
    Ok(())
}

fn validate_new_password(password: &str) -> Result<(), AppError> {
    let length = password.chars().count();
    if !(MIN_PASSWORD_LEN..=MAX_PASSWORD_LEN).contains(&length) {
        return Err(AppError::BadRequest(format!(
            "新密码需为 {MIN_PASSWORD_LEN} 到 {MAX_PASSWORD_LEN} 位"
        )));
    }
    if password == DEFAULT_PASSWORD {
        return Err(AppError::BadRequest("新密码不能是默认密码".to_string()));
    }
    Ok(())
}

fn new_token() -> String {
    format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple())
}

impl AdminService {
    /// Creates the default account (admin / 123456) the first time the database is used.
    pub async fn bootstrap(store: Option<Store>) -> Result<Self, AppError> {
        if let Some(store) = &store {
            if store.admin_account().await?.is_none() {
                let account = AdminAccount {
                    username: DEFAULT_USERNAME.to_string(),
                    password_hash: hash_password(DEFAULT_PASSWORD.to_string()).await?,
                    is_default_password: true,
                };
                if store.create_admin_account_if_absent(&account).await? {
                    warn!(
                        "Admin console created with the default login {DEFAULT_USERNAME} / {DEFAULT_PASSWORD}. \
                         Sign in and change the password."
                    );
                }
            }
        }
        Ok(Self {
            inner: Arc::new(Inner {
                store,
                sessions: Mutex::new(HashMap::new()),
                throttle: Mutex::new(Throttle::default()),
            }),
        })
    }

    pub fn is_available(&self) -> bool {
        self.inner.store.is_some()
    }

    pub fn store(&self) -> Result<&Store, AppError> {
        self.inner.store.as_ref().ok_or_else(|| {
            AppError::ServiceUnavailable("管理后台需要 PostgreSQL，请先在 .env 中配置 DATABASE_URL 后重启服务".to_string())
        })
    }

    fn check_throttle(&self) -> Result<(), AppError> {
        let throttle = lock(&self.inner.throttle);
        match throttle.locked_until.and_then(|t| t.checked_duration_since(Instant::now())) {
            Some(wait) => Err(AppError::TooManyRequests(format!(
                "尝试次数过多，请 {} 秒后再试",
                wait.as_secs() + 1
            ))),
            None => Ok(()),
        }
    }

    fn record_failure(&self) {
        let mut throttle = lock(&self.inner.throttle);
        throttle.failures += 1;
        if throttle.failures >= FAILURES_BEFORE_LOCK {
            throttle.failures = 0;
            throttle.lock_level += 1;
            let wait = BASE_LOCK
                .saturating_mul(1u32 << (throttle.lock_level - 1).min(10))
                .min(MAX_LOCK);
            throttle.locked_until = Some(Instant::now() + wait);
        }
    }

    fn record_success(&self) {
        *lock(&self.inner.throttle) = Throttle::default();
    }

    pub async fn account_info(&self) -> Result<AccountInfo, AppError> {
        let account = self
            .store()?
            .admin_account()
            .await?
            .ok_or_else(|| AppError::Internal("Admin account is missing".to_string()))?;
        Ok(AccountInfo {
            username: account.username,
            is_default_password: account.is_default_password,
        })
    }

    /// Checks the credentials and opens a session. Failed attempts count toward a lockout that
    /// lengthens each time it triggers, which keeps the default password from being guessed
    /// against a server that listens on the network.
    pub async fn login(&self, username: &str, password: &str) -> Result<(String, AccountInfo), AppError> {
        let store = self.store()?;
        self.check_throttle()?;

        let account = store
            .admin_account()
            .await?
            .ok_or_else(|| AppError::Internal("Admin account is missing".to_string()))?;

        // The hash is always verified so that a wrong username takes as long as a wrong password.
        let password_ok = verify_password(password.to_string(), account.password_hash.clone()).await;
        if !(password_ok && username == account.username) {
            self.record_failure();
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }
        self.record_success();

        let token = new_token();
        let now = Instant::now();
        lock(&self.inner.sessions).insert(
            token.clone(),
            SessionEntry {
                username: account.username.clone(),
                created: now,
                last_seen: now,
            },
        );

        Ok((
            token,
            AccountInfo {
                username: account.username,
                is_default_password: account.is_default_password,
            },
        ))
    }

    pub fn logout(&self, token: &str) {
        lock(&self.inner.sessions).remove(token);
    }

    /// Looks up a session and extends its idle timer.
    pub fn session(&self, token: &str) -> Option<AdminSession> {
        let mut sessions = lock(&self.inner.sessions);
        let now = Instant::now();
        sessions.retain(|_, s| now.duration_since(s.last_seen) < SESSION_IDLE && now.duration_since(s.created) < SESSION_MAX);
        let entry = sessions.get_mut(token)?;
        entry.last_seen = now;
        Some(AdminSession {
            token: token.to_string(),
            username: entry.username.clone(),
        })
    }

    /// Changes the username and/or password. The current password is required either way, and
    /// changing the password signs out every other session.
    pub async fn update_account(
        &self,
        session: &AdminSession,
        current_password: &str,
        new_username: Option<&str>,
        new_password: Option<&str>,
    ) -> Result<AccountInfo, AppError> {
        let store = self.store()?;
        if new_username.is_none() && new_password.is_none() {
            return Err(AppError::BadRequest("没有需要修改的内容".to_string()));
        }
        if let Some(name) = new_username {
            validate_username(name)?;
        }
        if let Some(password) = new_password {
            validate_new_password(password)?;
        }

        self.check_throttle()?;
        let account = store
            .admin_account()
            .await?
            .ok_or_else(|| AppError::Internal("Admin account is missing".to_string()))?;

        if !verify_password(current_password.to_string(), account.password_hash.clone()).await {
            self.record_failure();
            return Err(AppError::BadRequest("当前密码不正确".to_string()));
        }
        self.record_success();

        let updated = AdminAccount {
            username: new_username.map_or(account.username.clone(), str::to_string),
            password_hash: match new_password {
                Some(password) => hash_password(password.to_string()).await?,
                None => account.password_hash.clone(),
            },
            is_default_password: if new_password.is_some() { false } else { account.is_default_password },
        };
        store.save_admin_account(&updated).await?;

        let mut sessions = lock(&self.inner.sessions);
        if new_password.is_some() {
            sessions.retain(|token, _| *token == session.token);
        }
        for entry in sessions.values_mut() {
            entry.username = updated.username.clone();
        }

        Ok(AccountInfo {
            username: updated.username,
            is_default_password: updated.is_default_password,
        })
    }

    #[cfg(test)]
    fn failures_until_lock(&self) -> u32 {
        FAILURES_BEFORE_LOCK - lock(&self.inner.throttle).failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> AdminService {
        AdminService {
            inner: Arc::new(Inner {
                store: None,
                sessions: Mutex::new(HashMap::new()),
                throttle: Mutex::new(Throttle::default()),
            }),
        }
    }

    #[tokio::test]
    async fn password_hash_round_trip() {
        let hash = hash_password("correct horse".to_string()).await.unwrap();
        assert!(hash.starts_with("$argon2id$"), "unexpected hash format: {hash}");
        assert!(verify_password("correct horse".to_string(), hash.clone()).await);
        assert!(!verify_password("wrong".to_string(), hash).await);
    }

    #[test]
    fn repeated_failures_lock_login_with_growing_delay() {
        let svc = service();
        for _ in 0..FAILURES_BEFORE_LOCK - 1 {
            svc.record_failure();
            assert!(svc.check_throttle().is_ok());
        }
        assert_eq!(svc.failures_until_lock(), 1);
        svc.record_failure();
        assert!(matches!(svc.check_throttle(), Err(AppError::TooManyRequests(_))));

        let first_lock = lock(&svc.inner.throttle).locked_until.unwrap();
        lock(&svc.inner.throttle).locked_until = None;
        for _ in 0..FAILURES_BEFORE_LOCK {
            svc.record_failure();
        }
        let second_lock = lock(&svc.inner.throttle).locked_until.unwrap();
        assert!(second_lock > first_lock + Duration::from_secs(30), "second lock should be longer");

        svc.record_success();
        assert!(svc.check_throttle().is_ok());
    }

    #[test]
    fn credential_rules() {
        assert!(validate_username("ab").is_err());
        assert!(validate_username("has space").is_err());
        assert!(validate_username("管理员").is_err());
        assert!(validate_username("site.admin_01").is_ok());
        assert!(validate_new_password("short").is_err());
        assert!(validate_new_password("123456").is_err());
        assert!(validate_new_password("a-long-enough-one").is_ok());
    }

    #[test]
    fn sessions_are_distinct_and_revocable() {
        let svc = service();
        let (a, b) = (new_token(), new_token());
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);

        let now = Instant::now();
        lock(&svc.inner.sessions).insert(
            a.clone(),
            SessionEntry {
                username: "admin".into(),
                created: now,
                last_seen: now,
            },
        );
        assert_eq!(svc.session(&a).unwrap().username, "admin");
        assert!(svc.session(&b).is_none());
        svc.logout(&a);
        assert!(svc.session(&a).is_none());
    }
}
