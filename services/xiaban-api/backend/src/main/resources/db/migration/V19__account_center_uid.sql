-- Link local users to the unified account center (server-account); sub claim of its tokens.
alter table users add column account_uid uuid;
create unique index users_account_uid on users(account_uid) where account_uid is not null;
