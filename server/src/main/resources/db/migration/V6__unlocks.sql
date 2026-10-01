-- 表情图鉴：网页版能解锁 30 个，全套 75 个在相伴 App 里集齐（两边共用这张表）
create table unlocks (
    user_id       bigint      not null references users (id) on delete cascade,
    expression_id text        not null,
    unlocked_at   timestamptz not null default now(),
    reason        text        not null,
    primary key (user_id, expression_id)
);
