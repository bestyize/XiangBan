create table quiz_attempts (
    user_id bigint not null references users(id) on delete cascade,
    quiz_id text not null,
    version integer not null,
    answers jsonb not null,
    cursor integer not null,
    completed_at bigint,
    client_updated_at bigint not null,
    updated_at timestamptz not null default now(),
    primary key (user_id, quiz_id, version)
);
