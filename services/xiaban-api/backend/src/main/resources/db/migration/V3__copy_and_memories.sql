-- 连续 3 天很晚下班后进入关心模式，结果页换成关心的表情和文案
alter table checkins add column care_mode boolean not null default false;

-- 记忆：网页和相伴 App 共用，V1 由打卡里的关键事件生成
create table memories (
    id         bigserial primary key,
    user_id    bigint      not null references users (id) on delete cascade,
    kind       text        not null check (kind in ('event', 'preference', 'emotion')),
    content    text        not null,
    source     text        not null,
    source_ref text,
    created_at timestamptz not null default now()
);

create index memories_user_recent on memories (user_id, created_at desc);

-- 被举报的文案立即下架；需要恢复时由管理接口删除这一行
create table disabled_copies (
    copy_id    text primary key,
    reason     text        not null,
    created_at timestamptz not null default now()
);

create table copy_reports (
    id         bigserial primary key,
    user_id    bigint      references users (id) on delete set null,
    checkin_id bigint      references checkins (id) on delete set null,
    copy_id    text        not null,
    text       text        not null,
    reason     text,
    created_at timestamptz not null default now()
);
