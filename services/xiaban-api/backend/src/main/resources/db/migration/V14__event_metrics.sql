-- 数据监控：游客也上报匿名埋点，按“人”去重统计每天的访问、打卡、分享、游戏和测试人数。
-- actor：注册用户为 'u:<user_id>'，游客为 'd:<设备标识的 SHA-256>'；服务器不保存原始设备标识。
-- user_type：'registered' 或 'guest'，用来在后台区分两类人群。
-- work_day：按北京时间、凌晨 05:00 切分的统计日，与打卡的“下班日”一致。
alter table events add column actor text;
alter table events add column user_type text not null default 'registered' check (user_type in ('registered', 'guest'));
alter table events add column work_day date;

update events set
    actor = 'u:' || user_id,
    work_day = ((created_at at time zone 'Asia/Shanghai') - interval '5 hours')::date
where user_id is not null;
update events set work_day = ((created_at at time zone 'Asia/Shanghai') - interval '5 hours')::date where work_day is null;
alter table events alter column work_day set not null;

create index events_metrics on events (work_day, name, actor);

-- 注销账号时注册用户的埋点随账号删除；游客埋点没有账号可挂，180 天后由清理任务删除（见隐私政策）。
create index events_guest_created on events (created_at) where user_type = 'guest';
