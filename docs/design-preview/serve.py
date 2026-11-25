#!/usr/bin/env python3
"""Unified local static server for Xiaoban's cross-platform design navigator.

Serves the monorepo root so all relative design assets resolve cleanly, redirects
`/` to `/docs/design-preview/index.html`, and maps `/_blob/<id>` to the Claude
Design canvas blob folder so the desktop UI upgrade boards render inline.

Usage:
    python3 docs/design-preview/serve.py
    python3 docs/design-preview/serve.py --port 1455 --open
"""
from __future__ import annotations

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import socket
from urllib.parse import parse_qs, urlsplit
import webbrowser

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
UI_UPGRADE_BLOB_DIR = (
    REPO_ROOT / "client" / "xiaoban-desktop" / "docs" / "design-specs" / "ui-upgrade" / "_blob"
)
DEFAULT_INDEX = "/docs/design-preview/index.html"


class DesignHubHandler(SimpleHTTPRequestHandler):
    def end_headers(self) -> None:
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def do_GET(self) -> None:
        parsed = urlsplit(self.path)
        if parsed.path in ("", "/"):
            target = DEFAULT_INDEX + (f"?{parsed.query}" if parsed.query else "")
            self.send_response(302)
            self.send_header("Location", target)
            self.end_headers()
            return

        if parsed.path == "/__design_hub_status":
            payload = json.dumps(
                {
                    "server": "xiaoban-design-preview-hub",
                    "blobMapped": UI_UPGRADE_BLOB_DIR.is_dir(),
                    "root": str(REPO_ROOT),
                }
            ).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return

        if parsed.path == "/__vite_status":
            qs = parse_qs(parsed.query)
            origin = (qs.get("origin") or ["http://127.0.0.1:1420"])[0]
            target = urlsplit(origin)
            host = target.hostname or "127.0.0.1"
            port = target.port or (443 if target.scheme == "https" else 80)
            alive = False
            if host in ("127.0.0.1", "localhost", "::1"):
                try:
                    with socket.create_connection((host, port), timeout=0.25):
                        alive = True
                except OSError:
                    alive = False
            payload = json.dumps({"alive": alive, "origin": origin}).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
            return

        super().do_GET()

    def translate_path(self, path: str) -> str:
        parsed = urlsplit(path)
        if parsed.path.startswith("/_blob/"):
            rel = parsed.path[len("/_blob/") :].lstrip("/")
            candidate = (UI_UPGRADE_BLOB_DIR / rel).resolve()
            try:
                candidate.relative_to(UI_UPGRADE_BLOB_DIR.resolve())
                return str(candidate)
            except ValueError:
                pass
        return super().translate_path(path)

    def log_message(self, format: str, *args: object) -> None:
        # Keep terminal output concise.
        pass


def main() -> None:
    parser = argparse.ArgumentParser(description="Start Xiaoban unified design navigator server.")
    parser.add_argument("--host", default="127.0.0.1", help="Bind address (default: 127.0.0.1)")
    parser.add_argument("--port", type=int, default=1455, help="Port number (default: 1455)")
    parser.add_argument("--open", action="store_true", help="Open browser automatically")
    args = parser.parse_args()

    handler = partial(DesignHubHandler, directory=str(REPO_ROOT))
    server = ThreadingHTTPServer((args.host, args.port), handler)
    url = f"http://{args.host}:{args.port}/"
    print(f"小伴设计稿总导航已启动：{url}")
    print(f"仓库根目录：{REPO_ROOT}")
    print("提示：若需预览 4 份桌面 Vite 编译稿，请另起终端运行 npm --prefix client/xiaoban-desktop run dev")
    if args.open:
        webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n已停止预览服务。")
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
