-- 绑定到后台管理员的相伴账号（账号中心用户）。只有在这里登记过的账号才能登录后台。
CREATE TABLE admin_accounts (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL UNIQUE,
    label      TEXT NOT NULL,
    bound_by   TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
