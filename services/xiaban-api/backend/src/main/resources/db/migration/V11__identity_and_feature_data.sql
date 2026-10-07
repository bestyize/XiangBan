-- Identity is keyed by the account's immutable numeric ID; feature data owns separate tables.
create table user_profiles (
    user_id bigint primary key references users(id) on delete cascade,
    nickname text,
    nickname_key text unique,
    updated_at timestamptz not null default now(),
    check ((nickname is null) = (nickname_key is null))
);

with normalized as (
    select u.id, nullif(regexp_replace(normalize(btrim(c.nickname), NFKC), '\s+', ' ', 'g'), '') as name
    from users u join companions c on c.user_id=u.id
), valid as (
    select id, case when char_length(replace(name,' ','')) between 2 and 24
        and replace(name,' ','') !~ '^[0-9]+$'
        and (replace(name,' ','') !~ '^[A-Za-z0-9]+$' or char_length(replace(name,' ','')) >= 4)
        then name else null end as name from normalized
), ranked as (
    select id,name,row_number() over(partition by lower(name) order by id) as position from valid
)
insert into user_profiles(user_id,nickname,nickname_key)
select id,case when position=1 then name end,case when position=1 then lower(name) end from ranked;

comment on column companions.nickname is 'Legacy nickname snapshot; active identity lives in user_profiles.';

create table user_avatars (
    user_id bigint primary key references users(id) on delete cascade,
    asset_key text not null unique,
    image bytea not null,
    updated_at timestamptz not null default now(),
    check (octet_length(image) between 1 and 65536)
);

create table game_sessions (
    id uuid primary key,
    user_id bigint not null references users(id) on delete cascade,
    game_id text not null,
    started_at timestamptz not null default now(),
    finished_at timestamptz,
    score bigint not null default 0 check (score between 0 and 1000000000),
    lines integer not null default 0 check (lines between 0 and 1000000),
    max_tile integer not null default 0 check (max_tile between 0 and 1073741824)
);
create index game_sessions_user_game on game_sessions(user_id,game_id,started_at desc);

create table afterwork_choices (
    user_id bigint not null references users(id) on delete cascade,
    work_day date not null,
    activity_id text not null,
    selected_at timestamptz not null default now(),
    primary key(user_id,work_day)
);
create table afterwork_preferences (
    user_id bigint primary key references users(id) on delete cascade,
    preferred_activity_id text,
    prompt_dismissed_at jsonb not null default '{}'
);
