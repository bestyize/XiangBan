-- One progress document per account and game pack. The website stores it; only the pack interprets it.
create table game_saves (
    user_id bigint not null references users(id) on delete cascade,
    game_id text not null,
    data jsonb not null,
    client_updated_at bigint not null,
    updated_at timestamptz not null default now(),
    primary key (user_id, game_id)
);
