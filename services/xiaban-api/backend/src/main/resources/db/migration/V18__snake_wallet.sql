-- A wallet is owned by the authenticated account, never by a caller-supplied user ID.
create table snake_wallets (
    user_id bigint primary key references users(id) on delete cascade,
    coins integer not null default 40 check (coins >= 0),
    shields integer not null default 0 check (shields >= 0),
    start_shield_seconds integer not null default 0 check (start_shield_seconds in (0,5)),
    earning_day date not null,
    earned_today integer not null default 0 check (earned_today between 0 and 300),
    shared_day date,
    updated_at timestamptz not null default now()
);

create table snake_wallet_sessions (
    session_id uuid primary key references game_sessions(id) on delete cascade,
    user_id bigint not null references users(id) on delete cascade,
    revived boolean not null default false,
    start_shield_consumed boolean not null default false,
    settled boolean not null default false,
    coins_earned integer not null default 0 check (coins_earned between 0 and 80)
);
create index snake_wallet_sessions_user on snake_wallet_sessions(user_id);

create table snake_wallet_requests (
    user_id bigint not null references users(id) on delete cascade,
    request_id text not null check (char_length(request_id) between 1 and 120),
    action text not null,
    session_id uuid,
    response jsonb not null,
    created_at timestamptz not null default now(),
    primary key(user_id,request_id)
);
