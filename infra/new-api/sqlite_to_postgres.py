#!/usr/bin/env python3
"""把 New API 的 SQLite 库复制进 PostgreSQL。

目标库的表必须先由新版 New API 二进制建好（让它连上空库启动一次，再停掉），本脚本只负责搬数据：

    sudo python3 sqlite_to_postgres.py <SQLite 快照文件> <目标库名>

- 只读 SQLite，不改线上库；请先用 `sqlite3 <库> ".backup <快照>"` 取一份快照再导入。
- 要导入的表先 TRUNCATE 再整表导入，全部在一个事务里：失败不会留下半截数据。
- 目标库名必须以 newapi 开头，防止清空别的库。
- 按列名取交集：SQLite 里有、PostgreSQL 里没有的列会被丢掉并在输出里列出；反过来的列用默认值。
- 运行期临时数据（SKIP 里的表）不导入，新进程启动后会重建。
- 输出只有表名、行数和哈希比对结果，不打印任何密钥或用户数据。

需要 root（通过 runuser 以 postgres 身份调用 psql）和 python3 标准库，不需要 psycopg2。
"""
import hashlib
import sqlite3
import subprocess
import sys

SKIP = {"system_tasks", "system_task_locks", "system_instances", "perf_metrics", "login_encryption_keys", "auth_flows"}

# 导入后逐行核对的列。md5 列只比较哈希（里面可能是密钥），其余列直接比较。
CHECKS = {
    "users": [("id", ""), ("username", ""), ("role", ""), ("status", ""), ("quota", ""), ("used_quota", "")],
    "tokens": [("id", ""), ("user_id", ""), ("name", ""), ("remain_quota", ""), ("status", ""), ("key", "md5")],
    "channels": [("id", ""), ("name", ""), ("type", ""), ("status", ""), ("key", "md5")],
    "options": [("key", ""), ("value", "md5")],
}

INT_TYPES = {"smallint", "integer", "bigint"}
FLOAT_TYPES = {"real", "double precision"}

PSQL = ["runuser", "-u", "postgres", "--", "psql", "-X", "-v", "ON_ERROR_STOP=1"]

RESET_SEQUENCES = """
DO $$
DECLARE r record; m bigint;
BEGIN
  FOR r IN SELECT table_name, column_name FROM information_schema.columns
           WHERE table_schema = 'public' AND (column_default LIKE 'nextval%' OR is_identity = 'YES')
  LOOP
    EXECUTE format('SELECT COALESCE(MAX(%I), 0) FROM %I', r.column_name, r.table_name) INTO m;
    EXECUTE format('SELECT setval(pg_get_serial_sequence(%L, %L), GREATEST(%s, 1), %L::boolean)',
                   quote_ident(r.table_name), r.column_name, m, m > 0);
  END LOOP;
END $$;
"""


def fail(message):
    sys.exit(f"错误：{message}")


def pg_rows(db, sql):
    done = subprocess.run(PSQL + ["-At", "-F", "\t", "-d", db, "-c", sql], capture_output=True, text=True)
    if done.returncode:
        fail(f"查询失败：{done.stderr.strip()[:300]}")
    return [line.split("\t") for line in done.stdout.split("\n") if line != ""]


def escape(text):
    return text.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")


def convert(value, data_type):
    if value is None:
        return "\\N"
    if data_type == "boolean":
        return "t" if str(value).lower() in ("1", "t", "true") else "f"
    if data_type in INT_TYPES:
        return str(int(value))
    if data_type in FLOAT_TYPES:
        return repr(float(value))
    if data_type == "numeric":
        return str(value)
    if data_type == "bytea":
        raw = bytes(value) if isinstance(value, (bytes, bytearray)) else str(value).encode()
        return "\\\\x" + raw.hex()
    if isinstance(value, (bytes, bytearray)):
        value = bytes(value).decode("utf-8", "replace")
    return escape(str(value).replace("\x00", ""))


def main():
    if len(sys.argv) != 3:
        fail("用法：sqlite_to_postgres.py <SQLite 快照文件> <目标库名>")
    snapshot, db = sys.argv[1], sys.argv[2]
    if not db.startswith("newapi") or not db.replace("_", "").isalnum():
        fail("目标库名必须以 newapi 开头，且只含字母、数字和下划线")

    source = sqlite3.connect(f"file:{snapshot}?mode=ro", uri=True)
    source.text_factory = lambda raw: raw.decode("utf-8", "replace")
    source_tables = [r[0] for r in source.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")]

    target_columns = {}
    for table, column, data_type in pg_rows(db, "SELECT table_name, column_name, data_type FROM information_schema.columns WHERE table_schema='public' ORDER BY table_name, ordinal_position"):
        target_columns.setdefault(table, {})[column] = data_type
    if "users" not in target_columns:
        fail("目标库里还没有 New API 的表：先让新版二进制连上它启动一次")

    blocks, loaded, source_counts = [], [], {}
    print(f"{'表':34}{'源行数':>8}  说明")
    for table in source_tables:
        count = source.execute(f'SELECT COUNT(*) FROM "{table}"').fetchone()[0]
        if table in SKIP:
            print(f"{table:34}{count:>8}  跳过（运行期临时数据）")
            continue
        if table not in target_columns:
            print(f"{table:34}{count:>8}  目标库没有这张表，未导入")
            continue
        source_cols = [r[1] for r in source.execute(f'PRAGMA table_info("{table}")')]
        columns = [c for c in source_cols if c in target_columns[table]]
        dropped = [c for c in source_cols if c not in target_columns[table]]
        added = [c for c in target_columns[table] if c not in source_cols]
        notes = []
        if dropped:
            notes.append(f"丢弃列 {dropped}")
        if added:
            notes.append(f"目标新增列 {added}（用默认值）")
        print(f"{table:34}{count:>8}  {'；'.join(notes) or '导入'}")

        column_list = ", ".join(f'"{c}"' for c in columns)
        lines = [f'COPY "{table}" ({column_list}) FROM STDIN;']
        for row in source.execute(f'SELECT {column_list} FROM "{table}"'):
            lines.append("\t".join(convert(v, target_columns[table][c]) for v, c in zip(row, columns)))
        lines.append("\\.")
        blocks.append("\n".join(lines))
        loaded.append(table)
        source_counts[table] = count

    truncates = "".join(f'TRUNCATE TABLE "{t}";\n' for t in loaded)
    script = "SET session_replication_role = replica;\n" + truncates + "\n".join(blocks) + "\n" + RESET_SEQUENCES
    done = subprocess.run(PSQL + ["-q", "-1", "-d", db], input=script.encode(), capture_output=True)
    if done.returncode:
        fail(f"导入失败，事务已回滚，目标库没有改变：{done.stderr.decode(errors='replace').strip()[:400]}")

    print("\n核对（源 → 目标）：")
    bad = 0
    for table in loaded:
        got = int(pg_rows(db, f'SELECT COUNT(*) FROM "{table}"')[0][0])
        same = got == source_counts[table]
        bad += not same
        if source_counts[table] or got:
            print(f"  行数  {table:30}{source_counts[table]:>6} → {got:<6}{'OK' if same else '不一致'}")
    for table, spec in CHECKS.items():
        if table not in loaded:
            continue
        select = ", ".join(f'md5("{c}"::text)' if kind else f'"{c}"::text' for c, kind in spec)
        want = []
        for row in source.execute(f'SELECT {", ".join(chr(34) + c + chr(34) for c, _ in spec)} FROM "{table}"'):
            want.append(tuple(hashlib.md5(("" if v is None else str(v)).encode()).hexdigest() if kind else ("" if v is None else str(v)) for v, (_, kind) in zip(row, spec)))
        have = [tuple(r + [""] * (len(spec) - len(r))) for r in pg_rows(db, f'SELECT {select} FROM "{table}"')]
        same = sorted(want) == sorted(have)
        bad += not same
        print(f"  内容  {table:30}{len(want):>6} 行逐列比对  {'OK' if same else '不一致'}")
    if bad:
        fail(f"{bad} 项核对不一致，不要切换")
    print("\n全部一致。")


if __name__ == "__main__":
    main()
