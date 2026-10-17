<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime } from '../format';
import type { GatewaySettings } from '../types';

const emit = defineEmits<{ changed: [] }>();

const settings = ref<GatewaySettings | null>(null);
const loadError = ref('');

type Health = { state: 'idle' } | { state: 'checking' } | { state: 'ok' } | { state: 'bad'; message: string };
const health = ref<Health>({ state: 'idle' });

const token = ref('');
const reveal = ref(false);
const saving = ref(false);
const saveError = ref('');
const saved = ref('');

const sourceText = computed(() => {
  switch (settings.value?.source) {
    case 'console':
      return `在这个后台填写的（尾号 ${settings.value.hint ?? '未知'}）`;
    case 'env':
      return `来自服务器配置文件（尾号 ${settings.value.hint ?? '未知'}），在这里填写后会优先使用后台的`;
    default:
      return '还没有设置';
  }
});

async function test() {
  health.value = { state: 'checking' };
  try {
    const result = await api.testGateway();
    health.value = result.ok ? { state: 'ok' } : { state: 'bad', message: result.message ?? '网关没有接受这个令牌' };
  } catch (e) {
    health.value = { state: 'bad', message: errorMessage(e) };
  }
}

async function load() {
  try {
    settings.value = (await api.settings()).newapi;
    if (settings.value.configured) await test();
  } catch (e) {
    loadError.value = errorMessage(e);
  }
}
onMounted(load);

async function save() {
  saving.value = true;
  saveError.value = '';
  saved.value = '';
  try {
    const result = await api.saveGatewayToken(token.value);
    token.value = '';
    reveal.value = false;
    saved.value = `已验证并保存（尾号 ${result.hint}），马上生效，不需要重启。`;
    settings.value = (await api.settings()).newapi;
    health.value = { state: 'ok' };
    emit('changed');
  } catch (e) {
    saveError.value = errorMessage(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div>
    <div class="page-head">
      <div>
        <h1>网关连接</h1>
        <p>本服务通过一个管理令牌调用 AI 网关（New API）：为用户开通账号、加减额度、核对用量。令牌在这里填写，不用放进服务器的配置文件。</p>
      </div>
    </div>

    <div v-if="loadError" class="notice error" role="alert">{{ loadError }}</div>

    <template v-if="settings">
      <section class="panel status">
        <div class="status-line">
          <span class="dot" :class="health.state === 'ok' ? 'ok' : health.state === 'bad' ? 'bad' : settings.configured ? 'wait' : 'off'" aria-hidden="true"></span>
          <strong v-if="health.state === 'ok'">已连接</strong>
          <strong v-else-if="health.state === 'bad'">连不上或令牌无效</strong>
          <strong v-else-if="health.state === 'checking'">正在检查…</strong>
          <strong v-else>还没有设置令牌</strong>
          <button v-if="settings.configured" type="button" class="btn small ghost retest" :disabled="health.state === 'checking'" @click="test">重新检查</button>
        </div>
        <p v-if="health.state === 'bad'" class="notice error" role="alert">{{ health.message }}</p>
        <p v-if="settings.problem" class="notice warn" role="alert">{{ settings.problem }}</p>
        <p v-if="!settings.configured" class="notice warn">没有令牌时，用户无法开通，也不能调用 AI。填写下面的令牌后立即生效。</p>

        <dl>
          <div><dt>网关地址</dt><dd class="mono">{{ settings.base_url }}</dd></div>
          <div><dt>当前令牌</dt><dd>{{ sourceText }}</dd></div>
          <div v-if="settings.updated_at"><dt>更新时间</dt><dd class="num">{{ formatTime(settings.updated_at) }}</dd></div>
        </dl>
      </section>

      <form class="panel form" @submit.prevent="save">
        <h2>{{ settings.configured ? '更换令牌' : '填写令牌' }}</h2>

        <ol class="steps">
          <li>登录 AI 网关（New API）的后台，进入个人设置里的访问令牌。</li>
          <li>新建一个令牌，权限只勾选「用户」的<strong>查看</strong>和<strong>编辑</strong>，其余都不要勾。</li>
          <li>复制令牌，粘贴到下面。保存前会先向网关验证，填错不会影响现在正在用的令牌。</li>
        </ol>

        <div v-if="saveError" class="notice error" role="alert">{{ saveError }}</div>
        <div v-if="saved" class="notice ok" role="status">{{ saved }}</div>

        <div class="field">
          <label for="gw-token">管理令牌</label>
          <div class="token-row">
            <input
              id="gw-token"
              v-model="token"
              class="input mono"
              :type="reveal ? 'text' : 'password'"
              autocomplete="off"
              autocapitalize="off"
              spellcheck="false"
              placeholder="nap_…"
            />
            <button type="button" class="btn" @click="reveal = !reveal">{{ reveal ? '隐藏' : '显示' }}</button>
          </div>
          <span class="hint">保存后令牌会加密存放，之后在这里只能看到尾号，看不到完整内容。</span>
        </div>

        <div class="actions">
          <button type="submit" class="btn primary" :disabled="saving || !token.trim()">{{ saving ? '验证中…' : '验证并保存' }}</button>
        </div>
      </form>
    </template>
    <p v-else-if="!loadError" class="muted">加载中…</p>
  </div>
</template>

<style scoped>
.notice { margin: 0 0 14px; }
.status { padding: 18px 20px; margin-bottom: 18px; }
.status-line { display: flex; align-items: center; gap: 10px; margin-bottom: 14px; }
.status-line strong { font-size: 16px; }
.retest { margin-left: auto; }
.dot { width: 10px; height: 10px; border-radius: 50%; background: var(--line-strong); flex: none; }
.dot.ok { background: var(--primary); box-shadow: 0 0 0 4px var(--primary-soft); }
.dot.bad { background: var(--danger); box-shadow: 0 0 0 4px var(--danger-soft); }
.dot.wait { background: #c77d17; }
dl { margin: 0; display: flex; flex-direction: column; gap: 8px; }
dl > div { display: grid; grid-template-columns: 84px 1fr; gap: 12px; align-items: baseline; }
dt { color: var(--muted); }
dd { margin: 0; overflow-wrap: anywhere; }

.form { padding: 18px 20px; display: flex; flex-direction: column; gap: 14px; max-width: 640px; }
.steps { margin: 0; padding-left: 20px; color: var(--muted); display: flex; flex-direction: column; gap: 4px; max-width: 60ch; }
.steps strong { color: var(--ink); font-weight: 600; }
.token-row { display: flex; gap: 8px; }
.actions { display: flex; gap: 10px; }

@media (max-width: 520px) {
  dl > div { grid-template-columns: 1fr; gap: 2px; }
}
</style>
