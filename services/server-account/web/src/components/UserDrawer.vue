<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { api, errorMessage, formatTime } from '../api';
import type { UserDetail } from '../types';

const props = defineProps<{ uid: string }>();
const emit = defineEmits<{ close: []; changed: [] }>();

const detail = ref<UserDetail | null>(null);
const error = ref('');
const busy = ref(false);

const KIND_LABEL: Record<string, string> = { email: '邮箱', phone: '手机号' };

async function load() {
  error.value = '';
  try {
    detail.value = await api.user(props.uid);
  } catch (e) {
    error.value = errorMessage(e);
  }
}
watch(() => props.uid, load);
onMounted(load);

async function act(message: string, action: () => Promise<unknown>) {
  if (!window.confirm(message)) return;
  busy.value = true;
  error.value = '';
  try {
    await action();
    await load();
    emit('changed');
  } catch (e) {
    error.value = errorMessage(e);
  } finally {
    busy.value = false;
  }
}

function toggleStatus() {
  if (!detail.value) return;
  const disabling = detail.value.status === 'active';
  act(
    disabling ? '停用后该用户无法登录，且所有设备会立即下线。确定停用？' : '确定重新启用该用户？',
    () => api.setUserStatus(props.uid, disabling ? 'disabled' : 'active'),
  );
}

function revokeAll() {
  act('该用户所有设备将需要重新登录（已发出的访问令牌最长再存活约 30 分钟）。确定？', () => api.revokeSessions(props.uid));
}

function onKey(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close');
}
onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <aside class="drawer" role="dialog" aria-modal="true" aria-label="用户详情">
      <header>
        <h3>用户详情</h3>
        <button type="button" class="btn small" @click="emit('close')">关闭</button>
      </header>
      <p v-if="error" class="error" role="alert">{{ error }}</p>

      <template v-if="detail">
        <dl>
          <dt>昵称</dt><dd>{{ detail.user.nickname }}</dd>
          <dt>UID</dt><dd class="mono">{{ detail.user.uid }}</dd>
          <dt>状态</dt>
          <dd><span class="badge" :class="detail.status === 'active' ? 'ok' : 'bad'">{{ detail.status === 'active' ? '正常' : '已停用' }}</span></dd>
          <dt>密码</dt><dd>{{ detail.user.has_password ? '已设置' : '未设置（仅验证码登录）' }}</dd>
          <dt>注册时间</dt><dd>{{ formatTime(detail.user.created_at) }}</dd>
          <dt>最近登录</dt><dd>{{ formatTime(detail.last_login_at) }}</dd>
          <dt>头像</dt><dd class="mono wrap">{{ detail.user.avatar_url ?? '—' }}</dd>
        </dl>

        <h4>登录标识</h4>
        <ul class="plain">
          <li v-for="(identity, index) in detail.user.identities" :key="index">
            <span class="badge">{{ KIND_LABEL[identity.kind] ?? identity.kind }}</span> <span class="mono">{{ identity.masked }}</span>
          </li>
          <li v-if="!detail.user.identities.length" class="muted">无</li>
        </ul>

        <h4>有效会话（{{ detail.sessions.length }}）</h4>
        <div class="table-wrap">
          <table v-if="detail.sessions.length" class="table">
            <thead><tr><th>应用</th><th>设备</th><th>IP</th><th>登录时间</th></tr></thead>
            <tbody>
              <tr v-for="(s, index) in detail.sessions" :key="index">
                <td class="mono">{{ s.client_id }}</td>
                <td>{{ s.device_name ?? '—' }}</td>
                <td class="mono">{{ s.ip ?? '—' }}</td>
                <td>{{ formatTime(s.created_at) }}</td>
              </tr>
            </tbody>
          </table>
          <p v-else class="muted">当前没有有效会话</p>
        </div>

        <footer>
          <button type="button" class="btn" :disabled="busy || !detail.sessions.length" @click="revokeAll">下线全部设备</button>
          <button type="button" class="btn" :class="{ danger: detail.status === 'active' }" :disabled="busy" @click="toggleStatus">
            {{ detail.status === 'active' ? '停用用户' : '启用用户' }}
          </button>
        </footer>
      </template>
      <p v-else-if="!error" class="muted">加载中…</p>
    </aside>
  </div>
</template>

<style scoped>
.overlay { position: fixed; inset: 0; background: rgba(28, 32, 51, 0.4); z-index: 40; display: flex; justify-content: flex-end; }
.drawer { width: min(520px, 100%); height: 100%; overflow-y: auto; background: var(--surface); padding: 22px 24px; display: flex; flex-direction: column; gap: 14px; }
header { display: flex; align-items: center; justify-content: space-between; }
h4 { margin: 8px 0 0; font-size: 14px; }
dl { display: grid; grid-template-columns: 76px 1fr; gap: 8px 12px; margin: 0; }
dt { color: var(--muted); }
dd { margin: 0; min-width: 0; }
.wrap { word-break: break-all; }
ul.plain { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
footer { display: flex; justify-content: flex-end; gap: 10px; margin-top: auto; padding-top: 14px; border-top: 1px solid var(--border); }
p { margin: 0; }
</style>
