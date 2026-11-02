#!/usr/bin/env python3
"""接收用户从后台复制的受限采集令牌，写入本机私有文件。"""
import getpass, os, pathlib
value = getpass.getpass('粘贴论坛前沿采集令牌（不会显示）：').strip()
if not value or len(value) > 4096: raise SystemExit('令牌为空或过长')
path = pathlib.Path.home() / '.config' / 'xiaoban-frontier' / 'forum-token'
path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
with os.fdopen(fd, 'w') as f: f.write(value)
print('受限令牌已保存在本机私有文件。')
