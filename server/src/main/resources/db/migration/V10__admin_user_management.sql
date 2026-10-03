create table admin_user_notes (
    user_id bigint primary key references users(id) on delete cascade,
    note text not null default '',
    updated_by text not null,
    updated_at timestamptz not null default now(),
    constraint admin_note_length check (length(note) <= 2000)
);

create table admin_audit_log (
    id bigserial primary key,
    actor text not null,
    action text not null,
    target_user_id bigint references users(id) on delete set null,
    summary text not null,
    created_at timestamptz not null default now()
);

create index users_created_id on users(created_at desc, id desc);
create index users_email_verified on users(email_verified_at desc) where email is not null;
create index devices_user_seen on devices(user_id, last_seen_at desc);
create index events_user_created on events(user_id, created_at desc);
create index admin_audit_created on admin_audit_log(created_at desc, id desc);
