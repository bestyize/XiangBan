#!/usr/bin/env python3
"""Prepare pinned, prebuilt arm64 speech libraries for on-demand delivery (no native compilation).

assets/voice-runtime.json is embedded in the APK; downloads/* is published beside the APK.
Mirror downloads/* to bestyize/xiaoban-asr on ModelScope under android-voice/.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
import tarfile
import tomllib
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path

SHERPA_VERSION = "v1.13.8"
SHERPA_SHA256 = "2ff63469a71cb6009aa2e3ed5f4a670f8abdcbe4bb9ffd23776afc792a6b4f44"
ORT_VERSION = "1.30.0"
ORT_SHA256 = "e7fb945e402205f6db858d65bb78d2bdb0812317b383976c9e3bceb4862c73f1"
ABI = "arm64-v8a"
NAMES = ("libort_sherpa.so", "libsherpa-onnx-jni.so", "libonnxruntime.so", "libonnxruntime4j_jni.so")
MODEL_SOURCE = "https://modelscope.cn/api/v1/models/bestyize/xiaoban-asr/repo?Revision=master&FilePath=android-voice/"


def sha(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def fetch(url, path, expected):
    if path.is_file() and sha(path) == expected:
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".part")
    request = urllib.request.Request(url, headers={"User-Agent": "XiaoBan-build/1.0"})
    with urllib.request.urlopen(request, timeout=120) as response, temporary.open("wb") as target:
        shutil.copyfileobj(response, target)
    if sha(temporary) != expected:
        temporary.unlink()
        raise ValueError(f"Checksum mismatch: {path.name}")
    temporary.replace(path)


def native_cache_valid(native, stamp):
    try:
        data = json.loads(stamp.read_text())
        return data["recipe"] == sha(Path(__file__)) and all(
            (native / name).is_file() and sha(native / name) == data["hashes"][name] for name in NAMES
        )
    except (OSError, ValueError, KeyError):
        return False


def prepare_native(output, sherpa_archive=None, ort_aar=None):
    native = output / "native"
    native.mkdir(parents=True, exist_ok=True)
    stamp = output / "prepared.json"
    if native_cache_valid(native, stamp):
        return native
    patchelf = shutil.which("patchelf")
    if not patchelf:
        raise RuntimeError("Install patchelf to prepare the speech component (e.g. pip install patchelf)")
    archive = Path(sherpa_archive) if sherpa_archive else output / "upstream/sherpa.tar.bz2"
    aar = Path(ort_aar) if ort_aar else output / "upstream/onnxruntime.aar"
    if not sherpa_archive:
        fetch(f"https://github.com/k2-fsa/sherpa-onnx/releases/download/{SHERPA_VERSION}/sherpa-onnx-{SHERPA_VERSION}-android.tar.bz2", archive, SHERPA_SHA256)
    if not ort_aar:
        fetch(f"https://repo.maven.apache.org/maven2/com/microsoft/onnxruntime/onnxruntime-android/{ORT_VERSION}/onnxruntime-android-{ORT_VERSION}.aar", aar, ORT_SHA256)
    if sha(archive) != SHERPA_SHA256 or sha(aar) != ORT_SHA256:
        raise ValueError("Upstream speech runtime checksum mismatch")
    # Extract only the two reviewed files, never arbitrary archive paths.
    with tarfile.open(archive) as source:
        for original, target in (("libonnxruntime.so", "libort_sherpa.so"), ("libsherpa-onnx-jni.so", "libsherpa-onnx-jni.so")):
            member = next(m for m in source.getmembers() if m.name.endswith(f"/jniLibs/{ABI}/{original}") or m.name == f"jniLibs/{ABI}/{original}")
            with source.extractfile(member) as stream, (native / target).open("wb") as destination:
                shutil.copyfileobj(stream, destination)
    subprocess.run([patchelf, "--set-soname", "libort_sherpa.so", str(native / "libort_sherpa.so")], check=True)
    subprocess.run([patchelf, "--replace-needed", "libonnxruntime.so", "libort_sherpa.so", str(native / "libsherpa-onnx-jni.so")], check=True)
    with zipfile.ZipFile(aar) as source:
        for name in NAMES[2:]:
            (native / name).write_bytes(source.read(f"jni/{ABI}/{name}"))
    stamp.write_text(json.dumps({"recipe": sha(Path(__file__)), "hashes": {name: sha(native / name) for name in NAMES}}))
    return native


def publish_files(native, output, release_tag):
    libraries = []
    downloads = output / "downloads"
    downloads.mkdir(parents=True, exist_ok=True)
    for old in downloads.glob("*.so"):
        old.unlink()
    for name in NAMES:
        source = native / name
        digest = sha(source)
        filename = f"android-voice-arm64-{digest}-{name}"
        shutil.copyfile(source, downloads / filename)
        libraries.append({
            "name": name, "bytes": source.stat().st_size, "sha256": digest,
            "sources": [MODEL_SOURCE + filename, f"https://github.com/bestyize/XiangBan/releases/download/{urllib.parse.quote(release_tag, safe='')}/{filename}"],
        })
    version = hashlib.sha256("".join(item["sha256"] for item in libraries).encode()).hexdigest()
    manifest = {"format": 1, "abi": ABI, "version": version, "libraries": libraries}
    assets = output / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    (assets / "voice-runtime.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Speech runtime {version[:12]}: {sum(item['bytes'] for item in libraries):,} bytes; native cache reused when unchanged")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--release-tag", default="xiaoban-wallpaper-v3.0.0")
    parser.add_argument("--sherpa-archive")
    parser.add_argument("--onnx-aar")
    args = parser.parse_args()
    versions = Path(__file__).resolve().parents[2] / "gradle/libs.versions.toml"
    if tomllib.loads(versions.read_text())["versions"]["onnxruntime"] != ORT_VERSION:
        raise ValueError("Update the pinned ORT version and checksum together with libs.versions.toml")
    native = prepare_native(args.output, args.sherpa_archive, args.onnx_aar)
    publish_files(native, args.output, args.release_tag)


if __name__ == "__main__":
    main()
