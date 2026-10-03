-- Remove unverified guest accounts so the database only contains real registered users
delete from users where email is null and email_verified_at is null;
