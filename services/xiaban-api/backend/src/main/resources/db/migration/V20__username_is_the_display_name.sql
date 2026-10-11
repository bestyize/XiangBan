-- The display name is now the account center's unique, unchangeable username, mirrored at login; uniqueness lives there.
alter table user_profiles drop column nickname_key;
