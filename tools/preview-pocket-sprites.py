"""Serve the local Pocket Sprites preview, including correct Windows ESM MIME types."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import argparse

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--port', type=int, default=1473)
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
SimpleHTTPRequestHandler.extensions_map.update({'.mjs': 'text/javascript', '.js': 'text/javascript'})
handler = partial(SimpleHTTPRequestHandler, directory=str(root))
print(f'http://127.0.0.1:{args.port}/assets/game-packs/pocket-sprites/index.html', flush=True)
try:
    ThreadingHTTPServer(('127.0.0.1', args.port), handler).serve_forever()
except KeyboardInterrupt:
    pass
