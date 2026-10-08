#!/usr/bin/env python3
"""Check monorepo project entrypoints, README links, and client import boundaries."""
from pathlib import Path
from urllib.parse import unquote, urlsplit
import re
import sys

ROOT = Path(__file__).resolve().parent.parent
SKIP = {'node_modules', 'target', 'build', 'dist', 'dev-dist', 'artifacts', '.git', '.gradle', '.kotlin', '.dev', 'gen'}
ERRORS = []

def fail(message):
    ERRORS.append(message)

def source_files(directory, pattern):
    for file in directory.rglob(pattern):
        if not SKIP.intersection(file.relative_to(ROOT).parts):
            yield file

entrypoints = {
    'client/xiaban-web': 'package.json',
    'client/xiaoban-desktop-wallpaper': 'src-tauri/Cargo.toml',
    'services/xiaban-api/backend': 'build.gradle.kts',
    'services/xiaban-api/api-contracts': 'build.gradle.kts',
    'services/server-pan': 'backend/Cargo.toml',
    'services/server-xiaoban-wallpaper': 'backend/Cargo.toml',
}
planned = ['client/xiaoban-multiplatform', 'client/xiaoban-realbot',
           'services/server-asr', 'services/server-edgetts', 'services/server-websearch']
for project in [*entrypoints, *planned]:
    if not (ROOT/project/'README.md').is_file(): fail(f'Missing project README: {project}')
for project, entrypoint in entrypoints.items():
    if not (ROOT/project/entrypoint).is_file(): fail(f'Missing entrypoint: {project}/{entrypoint}')
for old in ['web', 'server', 'shared', 'deploy', 'compose.yaml']:
    if (ROOT/old).exists(): fail(f'Obsolete root path remains: {old}')

for directory in [ROOT/'client', ROOT/'services', ROOT/'infra', ROOT/'docs']:
    for readme in source_files(directory, '*.md'):
        text = readme.read_text(encoding='utf-8-sig')
        for match in re.finditer(r'\[[^\]]*\]\(([^\n)]+)\)', text):
            target = match.group(1).strip().strip('<>')
            if urlsplit(target).scheme or target.startswith('#'): continue
            target = unquote(target.split('#')[0])
            if target and not (readme.parent/target).exists(): fail(f'Broken link: {readme.relative_to(ROOT)} -> {target}')

for readme in [ROOT/'README.md']:
    for match in re.finditer(r'\[[^\]]*\]\(([^\n)]+)\)', readme.read_text(encoding='utf-8-sig')):
        target = match.group(1).strip().strip('<>')
        if urlsplit(target).scheme or target.startswith('#'): continue
        target = unquote(target.split('#')[0])
        if target and not (ROOT/target).exists(): fail(f'Broken root README link: {target}')

imports = re.compile(r'''(?:\bfrom\s*|\bimport\s*(?:\(\s*)?)['"](\.{1,2}/[^'"]+)['"]''')
for project in (ROOT/'client').iterdir():
    if not project.is_dir(): continue
    for suffix in ['*.ts', '*.tsx', '*.mjs']:
        for file in source_files(project, suffix):
            for target in imports.findall(file.read_text(encoding='utf-8-sig')):
                resolved = (file.parent/target).resolve()
                if not resolved.is_relative_to(project.resolve()):
                    fail(f'Cross-client source import: {file.relative_to(ROOT)} -> {target}')

if ERRORS:
    print('\n'.join(ERRORS))
    sys.exit(1)
print(f'PASS: {len(entrypoints)} implemented projects, {len(planned)} planned READMEs, local documentation links and client import boundaries.')
