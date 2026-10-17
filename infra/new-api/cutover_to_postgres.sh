#!/bin/bash
# 在首尔机上把线上 New API 升级到新二进制，并从 SQLite 切到 PostgreSQL。任何一步失败都自动回滚。
#
# 前提（本次操作里已准备好）：
#   /tmp/new-api.new               新版二进制（上游 main + 管理员代建令牌补丁，本地交叉编译）
#   /tmp/sqlite_to_postgres.py     同目录的加载脚本
#   /opt/new-api/.pg_password      newapi 角色的密码（root 可读）
#   newapi_trial 库                新二进制已在里面建好表（本脚本用它做模板，省得再起一个实例）
#
# 回滚材料：/opt/new-api/new-api.bak.<时间> 与 /opt/new-api/backups/<时间>/（.env、SQLite 快照）。
# SQLite 文件本身从头到尾只读，所以回滚就是换回旧二进制和旧 .env。
set -uo pipefail
umask 077

APP=/opt/new-api
TS=$(date +%Y%m%d-%H%M%S)
TSID=${TS//-/_}
BK=$APP/backups/$TS
SNAP=$BK/one-api.snapshot.db
NEW_BIN=/tmp/new-api.new
STOPPED=0

log() { echo "[$(date +%H:%M:%S)] $*"; }
pg() { runuser -u postgres -- psql -X -v ON_ERROR_STOP=1 -q "$@"; }
version() { curl -s -m 5 http://127.0.0.1:3000/api/status | python3 -c 'import sys,json; print(json.load(sys.stdin)["data"]["version"])' 2>/dev/null || echo "不可达"; }
wait_up() { for _ in $(seq 1 60); do curl -s -m 2 -o /dev/null http://127.0.0.1:3000/api/status && return 0; sleep 1; done; return 1; }

rollback() {
    log "!!! 回滚：换回旧二进制和旧 .env，继续用 SQLite"
    systemctl stop new-api
    install -m "$BIN_MODE" -o root -g root "$APP/new-api.bak.$TS" "$APP/new-api"
    install -m "$ENV_MODE" -o "${ENV_OWN%%:*}" -g "${ENV_OWN##*:}" "$BK/env.before" "$APP/.env"
    systemctl start new-api
    wait_up || log "回滚后服务没有起来，需要人工处理"
    log "回滚后：$(systemctl is-active new-api)，版本 $(version)，二进制校验 $(sha256sum "$APP/new-api" | cut -c1-16)（旧版应为 $OLD_SHA）"
}
fail() { log "失败：$*"; [ "$STOPPED" = 1 ] && rollback; exit 1; }

# 用一个 root 用户的令牌发一次最小的聊天请求，比较切换前后。结果放进 CHAT_CODE。
chat_test() {
    local label=$1 key=$2 hdr out
    hdr=$(mktemp) out=$(mktemp)
    printf 'Authorization: Bearer sk-%s\n' "$key" > "$hdr"
    CHAT_CODE=$(curl -s -m 90 -o "$out" -w '%{http_code}' http://127.0.0.1:3000/v1/chat/completions \
        -H @"$hdr" -H 'Content-Type: application/json' \
        -d '{"model":"deepseek-v4.1-flash","max_tokens":5,"messages":[{"role":"user","content":"hi"}]}')
    log "$label 聊天请求：HTTP $CHAT_CODE；$(python3 - "$out" <<'PY'
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    print("收到回复，模型字段=%s，用量=%s" % (d.get("model"), (d.get("usage") or {}).get("total_tokens")) if "choices" in d else "错误=%s" % str(d.get("error", d))[:160])
except Exception:
    print("响应无法解析")
PY
)"
    rm -f "$hdr" "$out"
}

[ "$(id -u)" = 0 ] || { echo "需要 root"; exit 1; }
for f in "$NEW_BIN" /tmp/sqlite_to_postgres.py "$APP/.pg_password"; do [ -s "$f" ] || { echo "缺少 $f"; exit 1; }; done

BIN_MODE=$(stat -c %a "$APP/new-api")
ENV_MODE=$(stat -c %a "$APP/.env")
ENV_OWN=$(stat -c %U:%G "$APP/.env")
OLD_SHA=$(sha256sum "$APP/new-api" | cut -c1-16)
PW=$(cat "$APP/.pg_password")

log "== 切换开始：线上 $(systemctl is-active new-api)，版本 $(version)"
mkdir -p "$BK" && chmod 700 "$BK" || fail "无法创建备份目录"
cp -p "$APP/new-api" "$APP/new-api.bak.$TS" && cp -p "$APP/.env" "$BK/env.before" || fail "备份失败"
log "已备份：new-api.bak.$TS、backups/$TS/env.before（旧二进制校验 $OLD_SHA）"

# 切换前先在旧版上做一次基线测试，之后才能分清“新版坏了”还是“上游渠道本来就不通”。
KEY=$(sqlite3 "$APP/data/one-api.db" "SELECT t.key FROM tokens t JOIN users u ON u.id = t.user_id WHERE t.status = 1 AND u.role = 100 ORDER BY t.id LIMIT 1")
BASE=skip
OLD_MODELS=$(curl -s -m 5 http://127.0.0.1:3000/api/pricing | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["data"]))' 2>/dev/null || echo "?")
if [ -n "$KEY" ]; then chat_test "旧版基线" "$KEY"; BASE=$CHAT_CODE; else log "没有 root 用户的可用令牌，跳过聊天基线"; fi

# 不停机的准备：用演练库当模板克隆出正式库，并清掉演练实例留下的运行期临时数据。
if [ -n "$(pg -Atc "SELECT 1 FROM pg_database WHERE datname = 'newapi'")" ]; then
    pg -c "ALTER DATABASE newapi RENAME TO newapi_old_$TSID" || fail "无法改名已存在的 newapi 库"
    log "已存在的 newapi 库改名为 newapi_old_$TSID"
fi
pg -c "CREATE DATABASE newapi TEMPLATE newapi_trial OWNER newapi" || fail "无法克隆正式库"
pg -d newapi -c "DO \$\$ DECLARE t text; BEGIN FOREACH t IN ARRAY ARRAY['system_tasks','system_task_locks','system_instances','perf_metrics','login_encryption_keys','auth_flows'] LOOP IF to_regclass('public.'||t) IS NOT NULL THEN EXECUTE format('TRUNCATE TABLE %I', t); END IF; END LOOP; END \$\$;" || fail "无法清理临时表"
log "正式库 newapi 已准备好（结构来自演练库）"

# ---- 停机窗口 ----
log "停止线上 new-api"
systemctl stop new-api || fail "停服务失败"
STOPPED=1
STOP_AT=$(date +%s)

sqlite3 "$APP/data/one-api.db" ".backup $SNAP" || fail "SQLite 快照失败"
[ "$(sqlite3 "$SNAP" 'PRAGMA integrity_check')" = ok ] || fail "SQLite 快照完整性检查没通过"
log "已取最终快照（服务已停，数据不会再变）"
SRC_USERS=$(sqlite3 "$SNAP" 'SELECT COUNT(*) FROM users')
SRC_CHANNELS=$(sqlite3 "$SNAP" 'SELECT COUNT(*) FROM channels')

python3 /tmp/sqlite_to_postgres.py "$SNAP" newapi > "$BK/loader.log" 2>&1 || { tail -n 15 "$BK/loader.log"; fail "数据导入失败"; }
sed -n '/^核对/,$p' "$BK/loader.log" | grep -E "行数|内容|全部一致" | sed 's/^/    /'

install -m "$BIN_MODE" -o root -g root "$NEW_BIN" "$APP/new-api" || fail "替换二进制失败"
{
    sed 's/^SQLITE_PATH=/#SQLITE_PATH=/' "$BK/env.before"
    printf '\n# %s 改用 PostgreSQL。回滚：换回 new-api.bak.%s，并用 backups/%s/env.before 覆盖本文件。\nSQL_DSN=postgres://newapi:%s@127.0.0.1:5432/newapi?sslmode=disable\n' "$(date +%F)" "$TS" "$TS" "$PW"
} > "$APP/.env.new"
install -m "$ENV_MODE" -o "${ENV_OWN%%:*}" -g "${ENV_OWN##*:}" "$APP/.env.new" "$APP/.env" && rm -f "$APP/.env.new"
[ "$(grep -c '^SQL_DSN=' "$APP/.env")" = 1 ] && [ "$(grep -c '^SQLITE_PATH=' "$APP/.env")" = 0 ] || fail ".env 改写不符合预期"

systemctl start new-api
wait_up || fail "新版没有在 60 秒内起来"
DOWN=$(( $(date +%s) - STOP_AT ))
log "新版已响应，停机约 ${DOWN} 秒"

# ---- 冒烟：任何一项不过都回滚 ----
sleep 5
V=$(version);                                                    log "版本：$V";                                  case "$V" in *upstream.6461f08*) ;; *) fail "版本不对";; esac
JOURNAL=$(journalctl -u new-api --since "-90 sec" --no-pager)   # 先存变量：pipefail 下 grep -q 提前退出会让 journalctl 吃 SIGPIPE
grep -q "using PostgreSQL" <<<"$JOURNAL" || fail "日志里没有 using PostgreSQL"
log "数据库：PostgreSQL（日志确认）"
N=$(curl -s -m 5 http://127.0.0.1:3000/api/pricing | python3 -c 'import sys,json; print(len(json.load(sys.stdin)["data"]))' 2>/dev/null || echo "?")
log "模型数：$N（切换前 $OLD_MODELS）"; [ "$N" = "$OLD_MODELS" ] || fail "模型数和切换前不一致"
U=$(pg -d newapi -Atc "SELECT COUNT(*) FROM users"); C=$(pg -d newapi -Atc "SELECT COUNT(*) FROM channels")
log "用户 $U（快照 $SRC_USERS），渠道 $C（快照 $SRC_CHANNELS）"; [ "$U" = "$SRC_USERS" ] && [ "$C" = "$SRC_CHANNELS" ] || fail "用户或渠道数量与快照不一致"
H=$(curl -s -m 10 --resolve newapi.thinkbody.xyz:443:127.0.0.1 -o /dev/null -w '%{http_code}' https://newapi.thinkbody.xyz/api/status); log "经 nginx 的 HTTPS 状态：$H"; [ "$H" = 200 ] || fail "经 nginx 访问失败"
CONN=$(pg -Atc "SELECT COUNT(*) FROM pg_stat_activity WHERE usename = 'newapi' AND datname = 'newapi'"); log "newapi 进程到 PostgreSQL 的连接数：$CONN"; [ "$CONN" -ge 1 ] || fail "PostgreSQL 里看不到新版的连接"
if [ -n "$KEY" ]; then
    M=$(curl -s -m 15 -o /dev/null -w '%{http_code}' http://127.0.0.1:3000/v1/models -H "Authorization: Bearer sk-$KEY"); log "令牌鉴权（/v1/models）：HTTP $M"; [ "$M" = 200 ] || fail "令牌鉴权失败"
    chat_test "新版" "$KEY"
    if [ "$BASE" = 200 ] && [ "$CHAT_CODE" != 200 ]; then fail "旧版聊天正常而新版不正常"; fi
    [ "$BASE" = 200 ] || log "注意：旧版基线本来就不是 200（$BASE），聊天结果仅供参考"
fi
log "== 切换完成：升级并改用 PostgreSQL，停机 ${DOWN} 秒。回滚材料保留在 $BK 和 new-api.bak.$TS"
