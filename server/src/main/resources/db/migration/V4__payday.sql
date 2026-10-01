-- 发薪倒计时（可选）：每月几号发薪
alter table users add column payday smallint check (payday between 1 and 31);
