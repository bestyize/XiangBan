-- 分享卡归因：二维码里只放邀请码，不暴露用户 ID
alter table users add column invite_code text unique;
alter table users add column referred_by bigint references users (id) on delete set null;
alter table users add column ref_source text;

update users set invite_code = substr(md5(random()::text || id::text), 1, 10) where invite_code is null;
alter table users alter column invite_code set not null;

-- 隐私：分享卡默认不显示城市，用户自己决定
alter table users add column show_city_on_card boolean not null default false;
