#!/usr/bin/env python3
"""Apply the image proxy fix to an installed 9Router 0.5.112 artifact.

This patches image requests and Codex token refresh. The provider proxy resolver
and HTTP transport are reused; no credentials are read or embedded here.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


MARKER = "/* xiangban:image-proxy:v2 */"
LEGACY_MARKER = "/* xiangban:image-proxy:v1 */"
REFRESH_MARKER = "/* xiangban:codex-refresh-proxy:v2 */"
LEGACY_REFRESH_MARKER = "/* xiangban:codex-refresh-proxy:v1 */"


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError("Installed image route differs from the supported build")
    return source.replace(before, after, 1)


def patch_module(source, start, end, replacements):
    left = source.index(start)
    right = source.index(end, left)
    module = source[left:right]
    for before, after in replacements:
        module = replace_once(module, before, after)
    return source[:left] + module + source[right:]


def patch_transport(source):
    source = patch_module(source, "72301:(a,b,c)=>", "},73024:", [
        ("var e=c(43659)", "var xbProxy=c(39326),xbFetch=c(39609),e=c(43659)"),
        ("let y=(0,i.j)(w);", "let xbProxyOptions=await xbProxy.B(c?.providerSpecificData||{});let y=(0,i.j)(w);"),
        ("y.executeViaExecutor(x,a,c,d)", "y.executeViaExecutor(x,a,c,d,xbProxyOptions)"),
        ('fetch(r,{method:"POST",headers:s,body:l(t)})',
         '(0,xbFetch.proxyAwareFetch)(r,{method:"POST",headers:s,body:l(t)},xbProxyOptions)'),
        ("z.refreshCredentials(c,d)", "z.refreshCredentials(c,d,xbProxyOptions)"),
        ('fetch(e,{method:"POST",headers:d,body:l(b)})',
         '(0,xbFetch.proxyAwareFetch)(e,{method:"POST",headers:d,body:l(b)},xbProxyOptions)'),
    ])
    source = patch_module(source, "91285:(a,b,c)=>", "},91645:", [
        ("async executeViaExecutor(a,b,c,d){", "async executeViaExecutor(a,b,c,d,xbProxyOptions){"),
        ("stream:!1,credentials:c,log:d}", "stream:!1,credentials:c,log:d,proxyOptions:xbProxyOptions}"),
    ])
    return source


def patch(source):
    if not source.startswith(LEGACY_MARKER):
        source = patch_transport(source)
    source = patch_module(source, "55436:(a,b,c)=>", "},55511:", [
        ('{accessToken:a.accessToken,refreshToken:a.refreshToken,providerSpecificData:a.providerSpecificData,testStatus:"active"}',
         '{...a,existingProviderSpecificData:b.providerSpecificData||{},testStatus:"active"}'),
    ])
    return MARKER + "\n" + source


def patch_refresh(source):
    transport_changes = [
        ("async function r(a,b){", "async function r(a,b,xbProxyOptions=null){"),
        ('fetch(e.NA.openai.token,{method:"POST",headers:{"Content-Type":"application/json",Accept:"application/json"},body:JSON.stringify({client_id:d.xq.codex.clientId,grant_type:"refresh_token",refresh_token:a})})',
         '(0,f.proxyAwareFetch)(e.NA.openai.token,{method:"POST",headers:{"Content-Type":"application/json",Accept:"application/json"},body:JSON.stringify({client_id:d.xq.codex.clientId,grant_type:"refresh_token",refresh_token:a})},xbProxyOptions)'),
        ("codex:(a,b)=>r(a.refreshToken,b)",
         'codex:async(a,b)=>{await c.e(8416);return r(a.refreshToken,b,await c(39326).B(a.providerSpecificData||{}))}'),
    ]
    if not source.startswith(LEGACY_REFRESH_MARKER):
        source = patch_module(source, "8521:(a,b,c)=>", "},18080:", transport_changes)
    source = patch_module(source, "8521:(a,b,c)=>", "},18080:", [
        ("var d=c(35024)", "var xbRefreshFetch=c(39609),d=c(35024)"),
        ("(0,f.proxyAwareFetch)(e.NA.openai.token,", "(0,xbRefreshFetch.proxyAwareFetch)(e.NA.openai.token,"),
    ])
    return REFRESH_MARKER + "\n" + source


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package_root", type=Path)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--backup-dir", type=Path)
    args = parser.parse_args()
    if json.loads((args.package_root / "package.json").read_text())["version"] != "0.5.112":
        raise ValueError("Only 9Router 0.5.112 is supported; review newer source first")
    build = args.package_root / "app/.next-cli-build/server"
    changes = []
    for route, marker, transform, name in [
        (build / "app/api/v1/images/generations/route.js", MARKER, patch, "images-generations-route.js"),
        (build / "chunks/3943.js", REFRESH_MARKER, patch_refresh, "oauth-refresh.js"),
    ]:
        source = route.read_text()
        if source.startswith(marker):
            continue
        changes.append((route, transform(source), name))
    if not changes:
        print("Image and Codex refresh proxy patches are already installed")
        return
    with tempfile.TemporaryDirectory(prefix="xb-image-proxy-") as temp:
        for route, patched, name in changes:
            candidate = Path(temp) / name
            candidate.write_text(patched)
            subprocess.run(["node", "--check", str(candidate)], check=True)
            print("Validated " + name + "; sha256=" + hashlib.sha256(patched.encode()).hexdigest())
        if args.apply:
            if not args.backup_dir:
                parser.error("--apply requires --backup-dir")
            args.backup_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
            for route, patched, name in changes:
                backup = args.backup_dir / (name + ".before")
                if backup.exists():
                    raise ValueError("Backup already exists; refusing to overwrite it")
            for route, patched, name in changes:
                shutil.copy2(route, args.backup_dir / (name + ".before"))
            for route, patched, name in changes:
                staging = route.with_name(route.name + ".xb-proxy-new")
                try:
                    shutil.copy2(route, staging)
                    staging.write_text(patched)
                    os.replace(staging, route)
                finally:
                    staging.unlink(missing_ok=True)
            print("Applied; restart 9router.service to load the patches")


if __name__ == "__main__":
    main()
