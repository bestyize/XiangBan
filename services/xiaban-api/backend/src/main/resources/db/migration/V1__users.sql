create table users (
    id           bigserial primary key,
    city_code    text,
    timezone     text        not null default 'Asia/Shanghai',
    work_start   time        not null default '09:00',
    work_end     time        not null default '18:00',
    onboarded_at timestamptz,
    created_at   timestamptz not null default now()
);

-- 匿名设备标识只存哈希；一个账号可以有多台设备（绑定邮箱后在新设备登录）
create table devices (
    device_hash  text primary key,
    user_id      bigint      not null references users (id) on delete cascade,
    created_at   timestamptz not null default now(),
    last_seen_at timestamptz not null default now()
);

create index devices_user_id on devices (user_id);

create table companions (
    user_id     bigint primary key references users (id) on delete cascade,
    name        text        not null default '小伴',
    personality text        not null default 'toxic',
    nickname    text,
    updated_at  timestamptz not null default now()
);
