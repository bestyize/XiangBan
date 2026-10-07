create table administrators (
    username text primary key,
    password_hash text not null,
    session_version integer not null default 1,
    updated_at timestamptz not null default now()
);

create table runtime_settings (
    key text primary key,
    encrypted_value text not null,
    updated_at timestamptz not null default now()
);
