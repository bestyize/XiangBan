//! 接收端的重放防护。WebSocket 本身保序，所以同一会话内要求序号严格递增（允许跳号，离线时会丢）。
//!
//! 会话（`ep`）的新旧看 `ep` 前 8 字节的启动计数：
//! - 计数比当前大：对方重启了，切到新会话，序号重新开始；
//! - 计数相同：必须是同一个 `ep`，且序号大于已收到的；
//! - 计数更小：旧会话的信封，一律拒绝，防止攻击者把早先截获的信封重放进来。
//!
//! 因此必须把 [`ReplayGuard::state`] 持久化，接收端重启后用 [`ReplayGuard::restore`] 恢复。
use crate::{envelope::{ep_counter, Envelope}, Error};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as B64, Engine};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReplayGuard {
    /// (启动计数, 完整 ep 的 base64, 已收到的最大序号)
    state: Option<(u64, String, u64)>,
}

impl ReplayGuard {
    pub fn new() -> Self { Self::default() }

    pub fn restore(state: Option<(u64, String, u64)>) -> Self { Self { state } }
    pub fn state(&self) -> Option<&(u64, String, u64)> { self.state.as_ref() }

    /// 在信封**解密成功之后**调用（能解密说明它由对方生成）。通过则记下，失败不改变状态。
    pub fn check(&mut self, env: &Envelope) -> Result<(), Error> {
        let raw = B64.decode(&env.ep).map_err(|_| Error::BadEncoding)?;
        let counter = ep_counter(&raw).ok_or(Error::BadEncoding)?;
        match &self.state {
            None => {}
            Some((cur, _, _)) if counter > *cur => {}
            Some((cur, ep, last)) if counter == *cur => {
                if *ep != env.ep || env.seq <= *last {
                    return Err(Error::Replay);
                }
            }
            Some(_) => return Err(Error::Replay),
        }
        self.state = Some((counter, env.ep.clone(), env.seq));
        Ok(())
    }
}
