#!/usr/bin/env python3
"""登录运营账号，保存在本机私有文件中供后续安全刷新。"""
import getpass
from auth_store import request, save
password = getpass.getpass('小伴前沿运营账号密码（不会显示）：')
response = request('/api/auth/password/login', {'identifier':'frontier_editor','password':password,'client_id':'xiaoban-frontier','device_name':'小伴前沿运营'})
save(response)
print('登录成功，运营会话已保存在本机私有文件。')
