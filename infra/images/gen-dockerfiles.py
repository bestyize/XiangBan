#!/usr/bin/env python3
"""按 services.json 生成各服务的运行时 Dockerfile（kind=rust）。改清单后重新运行本脚本，把结果一起提交。

用法：python3 infra/images/gen-dockerfiles.py          # 写文件
      python3 infra/images/gen-dockerfiles.py --check  # 只检查仓库里的 Dockerfile 是否和清单一致（CI 和本地都能跑）
"""
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]


def render(s: dict) -> str:
    has_web = "web" in s
    data = s.get("data")
    env = dict(s["env"])
    if has_web:
        env[s["web"]["env"]] = "/app/web"
    lines = [
        f"# {s['title']}的运行时镜像（由 infra/images/gen-dockerfiles.py 按 infra/images/services.json 生成，不要手改）。",
        f"# 二进制{'与前端静态资源' if has_web else ''}由构建机（ubuntu 24.04 x64）编译后放进构建上下文，布局：./{s['bin']}" + ("、./web" if has_web else "") + "。",
        "# 二进制动态链接 glibc，运行时基础镜像要不低于构建机的 glibc 版本。",
        "FROM ubuntu:24.04",
        "",
        "RUN apt-get update \\",
        "    && apt-get install -y --no-install-recommends ca-certificates curl \\",
        "    && rm -rf /var/lib/apt/lists/* \\",
        f"    && useradd --uid {s['uid']} --no-create-home --shell /usr/sbin/nologin {s['user']}" + (f" \\\n    && mkdir {data} && chown {s['user']} {data}" if data else ""),
        "",
        "WORKDIR /app",
        f"COPY {s['bin']} /app/{s['bin']}",
    ]
    if has_web:
        lines.append("COPY web /app/web")
    lines += ["", "ENV " + " \\\n    ".join(f"{k}={v}" for k, v in env.items()), "", f"USER {s['user']}", f"EXPOSE {s['port']}"]
    if data:
        lines.append(f"VOLUME {data}")
    lines += [
        "HEALTHCHECK --interval=15s --timeout=3s --start-period=20s --retries=5 \\",
        f"    CMD curl -fsS \"http://127.0.0.1:${{{s['portEnv']}}}{s['health']}\" || exit 1",
        "",
        f"ENTRYPOINT [\"/app/{s['bin']}\"]",
        "",
    ]
    return "\n".join(lines)


def main() -> int:
    check = "--check" in sys.argv
    services = json.loads((ROOT / "infra/images/services.json").read_text(encoding="utf-8"))["services"]
    bad = 0
    for s in services:
        if s["kind"] != "rust":
            continue
        path = ROOT / s["crate"] / "Dockerfile"
        text = render(s)
        if check:
            if not path.exists() or path.read_text(encoding="utf-8") != text:
                print(f"不一致：{path.relative_to(ROOT)}（运行 python3 infra/images/gen-dockerfiles.py 重新生成）")
                bad += 1
        else:
            path.write_text(text, encoding="utf-8")
            print(f"写入 {path.relative_to(ROOT)}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
