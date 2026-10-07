-- 埋点事件（PRD 10）；注销账号时随账号一起删除
create table events (
    id         bigserial primary key,
    user_id    bigint      references users (id) on delete cascade,
    name       text        not null,
    props      jsonb       not null default '{}',
    client_ts  timestamptz,
    user_agent text,
    created_at timestamptz not null default now()
);

create index events_name_time on events (name, created_at);
create index events_user on events (user_id);

-- 主动勾选同意（《个人信息保护法》）：协议版本、同意时间；内测期服务器在境外，需要单独同意
alter table users add column terms_version text;
alter table users add column terms_accepted_at timestamptz;
alter table users add column overseas_consent_at timestamptz;
