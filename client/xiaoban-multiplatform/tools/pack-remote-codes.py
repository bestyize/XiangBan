"""Build the offline TV and fixed-code AC resource from the pinned public snapshots."""

import argparse
import base64
import binascii
import gzip
import hashlib
import io
import json
import struct
import subprocess
import tarfile
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "androidApp/src/main/assets/game-packs-v2/remote-control/codes.tar.xz"
BASE_SHA256 = "44dcf13d815383c907884f7cdb294afcd085b7063c1de4780589e0626cb61895"
COMMUNITY_COMMIT = "35dcc787a2fc734a05fb684e923f0451009a20e8"
COMMUNITY_SHA256 = "4b9ef452ae6926a3b86b4a62ab672f0e370e41f061592d7a084449bf38c4830b"
FIXED_CODE_KEY = b"fd7e915003168929c1a9b0ec32a60788"
TV_BUTTONS = {
    "power": ("power",), "home": ("home",), "menu": ("menu",),
    "up": ("up",), "down": ("down",), "left": ("left",), "right": ("right",),
    "ok": ("ok", "enter"), "volumeUp": ("vol+",), "volumeDown": ("vol-",),
    "back": ("back", "return", "exit"), "input": ("input", "tv_av"),
}
AC_BUTTONS = {
    "power": ("power",), "tempUp": ("temperature_up",), "tempDown": ("temperature_down",),
    "mode": ("mode",), "fan": ("fanspeed", "wind_speed"),
    "swing": ("shake_wind", "ud_wind_mode_swing", "oscillation"),
    "direction": ("WIND DIRECTION", "ud_wind_mode", "ud_wind_mode_fix"),
    "cool": ("COOL",), "heat": ("HEAT",), "dry": ("DRY",), "air": ("FAN",),
    "auto": ("automatic", "AUTO"),
}


def read_snapshot(source, sha256, prefix, group, categories, indexes, models):
    with source.open("rb") as stream:
        if hashlib.file_digest(stream, "sha256").hexdigest() != sha256:
            raise ValueError(f"Unexpected snapshot checksum: {source}")
        stream.seek(0)
        with tarfile.open(fileobj=stream, mode="r|*") as archive:
            for entry in archive:
                if not entry.isfile() or not entry.name.startswith(prefix):
                    continue
                path = entry.name.removeprefix(prefix)
                category = path.split("/", 1)[0].removesuffix(".json")
                if category not in categories or not path.endswith(".json") or not 1 <= entry.size <= 512_000:
                    continue

                with archive.extractfile(entry) as source_file:
                    payload = source_file.read()
                name = f"{group}/{path}"
                if path.startswith(f"{category}/models/"):
                    data = json.loads(payload).get("data")
                    if not isinstance(data, dict) or (category == "3_AC" and data.get("type") != 1):
                        continue
                    frequency = data.get("frequency")
                    keys = data.get("key")
                    if isinstance(frequency, int) and 20_000 <= frequency <= 60_000 and isinstance(keys, dict):
                        buttons = TV_BUTTONS if category == "1_TV" else AC_BUTTONS
                        models.append((name, frequency, keys, buttons))
                else:
                    indexes.append((name, payload))


def decode_patterns(models):
    # ECB blocks are independent, so one OpenSSL invocation can decrypt the whole batch.
    spans = {}
    encrypted = bytearray()
    for _, _, keys, buttons in models:
        for candidates in buttons.values():
            for key in candidates:
                for field in (key, key + "_r"):
                    code = keys.get(field)
                    if not isinstance(code, str) or not code.strip() or code in spans:
                        continue
                    try:
                        blocks = base64.b64decode(code, validate=False)
                    except (ValueError, binascii.Error):
                        continue
                    if blocks and len(blocks) % 16 == 0:
                        spans[code] = (len(encrypted), len(blocks))
                        encrypted.extend(blocks)
    decoded = subprocess.run(
        ["openssl", "enc", "-d", "-aes-256-ecb", "-K", FIXED_CODE_KEY.hex(), "-nopad"],
        input=encrypted, check=True, capture_output=True,
    ).stdout
    patterns = {}
    for code, (start, size) in spans.items():
        try:
            pattern = json.loads(gzip.decompress(decoded[start:start + size].rstrip(b" ")))
        except (OSError, EOFError, ValueError):
            continue
        if (
            isinstance(pattern, list) and 2 <= len(pattern) <= 1024
            and all(type(pulse) is int and 1 <= pulse <= 1_000_000 for pulse in pattern)
            and sum(pattern) <= 1_000_000
        ):
            patterns[code] = base64.b64encode(struct.pack(f">{len(pattern)}I", *pattern)).decode()
    return patterns


def write_model_entries(models, patterns, destination):
    seen = set()
    for name, frequency, keys, buttons in models:
        path = name.split("/", 1)[1]
        if path in seen:
            continue
        commands = {}
        for action, candidates in buttons.items():
            for key in candidates:
                code = keys.get(key)
                if not isinstance(code, str) or code not in patterns:
                    continue
                commands[action] = [patterns[code]]
                alternate = keys.get(key + "_r")
                if isinstance(alternate, str) and alternate in patterns:
                    commands[action].append(patterns[alternate])
                break
        if "power" in commands:
            seen.add(path)
            data = {"frequency": frequency, "commands": commands}
            add_entry(destination, name, json.dumps({"data": data}, separators=(",", ":")).encode())


def add_entry(destination, name, payload):
    packed = tarfile.TarInfo(name)
    packed.size = len(payload)
    destination.addfile(packed, io.BytesIO(payload))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path, help="ysard v2.1 database_dump.tar.xz")
    parser.add_argument("community", type=Path, help=f"halifox {COMMUNITY_COMMIT} source tar.gz")
    args = parser.parse_args()

    indexes = []
    models = []
    read_snapshot(args.base, BASE_SHA256, "database_dump/", "base", {"1_TV", "3_AC"}, indexes, models)
    read_snapshot(
        args.community, COMMUNITY_SHA256,
        f"android_xiaomi_ir_lab-{COMMUNITY_COMMIT}/app/src/main/assets/",
        "community", {"1_TV"}, indexes, models,
    )
    patterns = decode_patterns(models)
    if not patterns:
        raise ValueError("No valid IR pulses found")
    with tempfile.TemporaryDirectory(dir=ROOT) as scratch:
        temporary = Path(scratch) / OUTPUT.name
        with tarfile.open(temporary, mode="w:xz", preset=6) as destination:
            for name, payload in indexes:
                add_entry(destination, name, payload)
            write_model_entries(models, patterns, destination)
        temporary.replace(OUTPUT)
    print(f"{OUTPUT.name}: {OUTPUT.stat().st_size:,} bytes")


if __name__ == "__main__":
    main()
