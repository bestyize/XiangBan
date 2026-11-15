#!/usr/bin/env python3
"""Index a published, path-based asset tree by content without changing its URLs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile
from urllib.parse import quote


def digest_file(path: Path) -> tuple[str, str, int]:
    md5, sha256, size = hashlib.md5(), hashlib.sha256(), 0
    with path.open('rb') as stream:
        while chunk := stream.read(1024 * 1024):
            md5.update(chunk)
            sha256.update(chunk)
            size += len(chunk)
    return md5.hexdigest(), sha256.hexdigest(), size


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', required=True, type=Path)
    parser.add_argument('--objects', required=True, type=Path)
    parser.add_argument('--manifest', required=True, type=Path)
    parser.add_argument('--url-prefix', required=True)
    args = parser.parse_args()
    if not args.url_prefix.startswith('https://res.xiangban.xyz/'):
        parser.error('URL prefix must be on the resource host')
    source = args.source.resolve(strict=True)
    args.objects.mkdir(parents=True, exist_ok=True)
    records = []
    for path in sorted(source.rglob('*')):
        if path.is_symlink():
            raise ValueError(f'symlink not allowed: {path}')
        if not path.is_file():
            continue
        md5, sha256, size = digest_file(path)
        suffix = path.suffix.lower().lstrip('.')
        if not suffix.isascii() or not suffix.isalnum() or len(suffix) > 8:
            suffix = 'bin'
        target = args.objects / f'{md5}.{suffix}'
        try:
            os.link(path, target)
        except FileExistsError:
            if target.is_symlink() or digest_file(target)[1:] != (sha256, size):
                raise ValueError(f'content address collision: {target}')
        relative = path.relative_to(source).as_posix()
        records.append({'path': relative, 'url': args.url_prefix.rstrip('/') + '/' + quote(relative),
                        'object': target.name, 'size': size, 'sha256': sha256})
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', dir=args.manifest.parent, delete=False) as output:
        name = output.name
        json.dump(records, output, ensure_ascii=False, separators=(',', ':'))
        output.write('\n')
    os.chmod(name, 0o644)
    os.replace(name, args.manifest)
    print(f'Indexed {len(records)} files')


if __name__ == '__main__':
    main()
