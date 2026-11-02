#!/usr/bin/env python3
"""Publish plainly named runtime libraries and pin the verified ModelScope revisions for APK builds.

Requires modelscope-hub==0.6.1 only when files need uploading; authentication comes from
MODELSCOPE_API_TOKEN or an existing official SDK login. Existing model files are never changed.
"""
import argparse
import hashlib
import json
import os
import re
import urllib.request
from pathlib import Path

REPO = "bestyize/xiaoban-asr"
FILES_API = f"https://modelscope.cn/api/v1/models/{REPO}/repo/files?Revision=master&Recursive=true"
NAMES = {"libort_sherpa.so", "libsherpa-onnx-jni.so", "libonnxruntime.so", "libonnxruntime4j_jni.so"}
REMOTE_DIRECTORY = ""


def expected_files(directory):
    manifest = json.loads((directory / "assets/voice-runtime.json").read_text())
    if manifest["format"] != 1 or manifest["abi"] != "arm64-v8a" or {item["name"] for item in manifest["libraries"]} != NAMES:
        raise RuntimeError("Unsupported runtime manifest")
    items = []
    for library in manifest["libraries"]:
        if not re.fullmatch("[a-f0-9]{64}", library["sha256"]):
            raise RuntimeError("Invalid runtime checksum")
        filename = library["name"]
        local = directory / "downloads" / filename
        with local.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        if digest != library["sha256"] or local.stat().st_size != library["bytes"]:
            raise RuntimeError(f"Local runtime checksum mismatch: {library['name']}")
        items.append((local, REMOTE_DIRECTORY + filename, library))
    if len(items) != 4:
        raise RuntimeError("Expected all four native speech libraries")
    return items


def remote_files():
    request = urllib.request.Request(FILES_API, headers={"Cache-Control": "no-cache"})
    with urllib.request.urlopen(request, timeout=30) as response:
        result = json.load(response)
    if result.get("Code") != 200:
        raise RuntimeError("Unable to read the ModelScope runtime repository")
    return {item["Path"]: item for item in result["Data"]["Files"]}


def missing_files(items, remote):
    return [item for item in items if not (
        remote.get(item[1], {}).get("Sha256") == item[2]["sha256"]
        and remote.get(item[1], {}).get("Size") == item[2]["bytes"]
    )]


def publish(directory):
    items = expected_files(directory)
    remote = remote_files()
    missing = missing_files(items, remote)
    if missing:
        try:
            from modelscope_hub import HubApi
            api = HubApi(token=os.environ.get("MODELSCOPE_API_TOKEN") or None)
            api.whoami()
        except Exception:
            raise RuntimeError("ModelScope runtime files are missing. Configure MODELSCOPE_API_TOKEN with write access to bestyize/xiaoban-asr, or upload downloads/*.so into the repository root manually before publishing the APK.") from None
        for local, path, _ in missing:
            api.upload_file(repo_id=REPO, repo_type="model", path_or_fileobj=local, path_in_repo=path,
                            commit_message="Publish Android runtime libraries", disable_tqdm=True)
        remote = remote_files()
    if missing_files(items, remote):
        raise RuntimeError("ModelScope publication could not be verified; APK publication stopped")
    # Short filenames remain stable; the URL's Git revision keeps old APKs on their compatible binaries.
    revisions = {}
    for _, path, library in items:
        revision = remote[path].get("Revision", "")
        if not re.fullmatch("[a-f0-9]{40}", revision):
            raise RuntimeError("ModelScope did not return an immutable file revision; APK publication stopped")
        revisions[library["name"]] = {"sha256": library["sha256"], "revision": revision}
    receipt = directory / "published-revisions.json"
    temporary = receipt.with_suffix(".part")
    temporary.write_text(json.dumps(revisions, indent=2) + "\n")
    temporary.replace(receipt)
    print("Verified runtime files and pinned their ModelScope revisions for the APK")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--directory", required=True, type=Path)
    args = parser.parse_args()
    publish(args.directory)
