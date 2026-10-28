"""Regenerate tiny, self-authored 440 Hz fixtures. Requires Python and ffmpeg.

Usage: python tests/generate_fixtures.py /path/to/ffmpeg
No encoder or external executable is required by cargo test or the library.
"""
import base64
import io
import math
from pathlib import Path
import struct
import subprocess
import sys
import wave

buffer = io.BytesIO()
with wave.open(buffer, "wb") as wav:
    wav.setnchannels(1)
    wav.setsampwidth(2)
    wav.setframerate(44100)
    wav.writeframes(b"".join(struct.pack("<h", round(8000 * math.sin(2 * math.pi * 440 * i / 44100))) for i in range(11025)))
destination = Path(__file__).parent / "fixtures"
destination.mkdir(exist_ok=True)
for name, codec, fmt in [("mp3", "libmp3lame", "mp3"), ("flac", "flac", "flac"), ("ogg", "libvorbis", "ogg")]:
    data = subprocess.run([sys.argv[1], "-hide_banner", "-loglevel", "error", "-i", "pipe:0", "-map_metadata", "-1", "-c:a", codec, "-f", fmt, "pipe:1"], input=buffer.getvalue(), stdout=subprocess.PIPE, check=True).stdout
    # Text files only; generated audio is embedded in memory by tests/example.
    encoded = base64.b64encode(data).decode("ascii")
    (destination / f"{name}.b64").write_text("\n".join(encoded[i:i+100] for i in range(0, len(encoded), 100)) + "\n", encoding="ascii")
    print(f"{name}: {len(data)} bytes")
