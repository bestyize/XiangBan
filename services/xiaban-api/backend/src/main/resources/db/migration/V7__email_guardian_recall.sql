-- 绑定邮箱：换设备时找回账号，也用于平安守护提醒和召回邮件
alter table users add column email text unique;
alter table users add column email_verified_at timestamptz;

-- 召回邮件：用户自愿订阅，每一段流失期最多 2 封
alter table users add column recall_opt_in boolean not null default false;
alter table users add column recall_count int not null default 0;
alter table users add column last_recall_at timestamptz;

-- 每一次发信尝试都记一笔；收件人只存打码后的地址
create table mail_logs (
    id         bigserial primary key,
    kind       text        not null,
    ref        text,
    recipient  text        not null,
    attempt    int         not null,
    ok         boolean     not null,
    error      text,
    created_at timestamptz not null default now()
);

create table guardians (
    user_id        bigint primary key references users (id) on delete cascade,
    contact_name   text        not null,
    contact_email  text        not null,
    self_name      text        not null,
    threshold_days smallint    not null check (threshold_days between 1 and 3),
    paused         boolean     not null default false,
    -- 开启或恢复守护的时间：比最后一次打卡晚时，从这里开始算
    base_at        timestamptz not null,
    next_remind_at timestamptz,
    next_notify_at timestamptz,
    consented_at   timestamptz not null,
    informed_at    timestamptz not null,
    created_at     timestamptz not null default now(),
    updated_at     timestamptz not null default now()
);

-- 同一个到期时间的提醒和通知各只发一次（幂等）
create table guardian_alerts (
    id         bigserial primary key,
    user_id    bigint      not null references users (id) on delete cascade,
    kind       text        not null check (kind in ('remind', 'notify')),
    due_at     timestamptz not null,
    status     text        not null default 'queued',
    created_at timestamptz not null default now(),
    unique (user_id, kind, due_at)
);
