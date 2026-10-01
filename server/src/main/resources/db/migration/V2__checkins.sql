create table checkins (
    id            bigserial primary key,
    user_id       bigint      not null references users (id) on delete cascade,
    work_day      date        not null,
    checked_at    timestamptz not null,
    title         text        not null,
    copy_id       text        not null,
    library_reply text        not null,
    final_reply   text,
    reply_source  text        not null default 'library',
    revoked_at    timestamptz,
    created_at    timestamptz not null default now()
);

-- 每个下班日只能有一次有效打卡；撤销后的记录保留，用来限制“一天只能撤销一次”
create unique index checkins_one_per_day on checkins (user_id, work_day) where revoked_at is null;
create index checkins_user_day on checkins (user_id, work_day desc);
