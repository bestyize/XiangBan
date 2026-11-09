"""Pack SmartIR Midea IR captures for offline use in the remote app."""

import argparse
import base64
import binascii
import json
import lzma
import shutil
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "androidApp/src/main/assets/game-packs-v2/remote-control"
MODES = {
    "cool": ("cool",),
    "heat": ("heat",),
    "dry": ("dry",),
    "air": ("fan_only", "fan"),
    "auto": ("auto", "heat_cool"),
}


def pulses(value):
    if not isinstance(value, str):
        return None
    try:
        packet = base64.b64decode("".join(value.split()), validate=True)
    except binascii.Error:
        return None
    if len(packet) < 7 or packet[:2] != b"\x26\x00":
        return None
    length = int.from_bytes(packet[2:4], "little")
    end = 4 + length
    if end > len(packet) or any(packet[end:]):
        return None

    # Broadlink uses 2^-15-second ticks with big-endian escapes for long durations.
    durations = []
    cursor = 4
    while cursor < end:
        ticks = packet[cursor]
        cursor += 1
        if not ticks:
            if cursor + 2 > end:
                return None
            ticks = int.from_bytes(packet[cursor:cursor + 2], "big")
            cursor += 2
        if not ticks:
            return None
        durations.append(round(ticks * 1_000_000 / 32768))
    if 2 <= len(durations) <= 1024 and not len(durations) % 2 and sum(durations) <= 1_000_000:
        return durations
    return None


def read_profile(path):
    data = json.loads(path.read_bytes())
    if (
        data["manufacturer"] != "Midea"
        or data["commandsEncoding"] != "Base64"
        or data["supportedController"] != "Broadlink"
        or not 16 <= data["minTemperature"] <= data["maxTemperature"] <= 32
    ):
        return None
    commands = data["commands"]
    off = pulses(commands.get("off"))
    if off is None:
        return None
    settings = {}
    for mode, names in MODES.items():
        speeds = next((commands[name] for name in names if name in commands), {})
        if not isinstance(speeds, dict):
            continue
        mode_settings = {}
        for speed in data["fanModes"]:
            captured = speeds.get(speed)
            if not isinstance(captured, dict):
                continue
            temperatures = {}
            for degree in range(int(data["minTemperature"]), int(data["maxTemperature"]) + 1):
                pattern = pulses(captured.get(str(degree)))
                if pattern is not None:
                    temperatures[str(degree)] = pattern
            if temperatures:
                mode_settings[speed] = temperatures
        if mode_settings:
            settings[mode] = mode_settings
    if "cool" not in settings:
        return None
    return {
        "id": f"ac:smartir-{path.stem}",
        "models": data["supportedModels"],
        "off": off,
        "settings": settings,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="SmartIR checkout at the pinned commit in the Android README")
    args = parser.parse_args()
    profiles = []
    # Lead with the KFR units in the Chinese market, followed by named remotes.
    preferred = (1383, 1386)
    remaining = [code for code in range(1382, 1398) if code not in preferred]
    for code in (*preferred, *remaining, 1380, 1381):
        profile = read_profile(args.source / f"codes/climate/{code}.json")
        if profile is not None:
            profiles.append(profile)
    if not profiles:
        raise ValueError("No Midea IR profiles found")
    with tempfile.TemporaryDirectory(dir=ROOT) as scratch:
        packed = Path(scratch) / "midea.json.xz"
        payload = json.dumps(profiles, ensure_ascii=False, separators=(",", ":")).encode()
        packed.write_bytes(lzma.compress(payload))
        shutil.copyfile(args.source / "LICENSE", PACK / "smartir-license.txt")
        packed.replace(PACK / packed.name)
    print(f"Packed {len(profiles)} Midea profiles")


if __name__ == "__main__":
    main()
