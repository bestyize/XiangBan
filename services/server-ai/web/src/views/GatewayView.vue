<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime } from '../format';
import { notify } from '../toast';
import type { GatewaySettings } from '../types';
import Icon from '../components/Icon.vue';
import PageHead from '../components/PageHead.vue';

const emit = defineEmits<{ changed: [] }>();

const settings = ref<GatewaySettings | null>(null);
const loadError = ref('');

type Health = { state: 'idle' } | { state: 'checking' } | { state: 'ok' } | { state: 'bad'; message: string };
const health = ref<Health>({ state: 'idle' });

const token = ref('');
const reveal = ref(false);
const saving = ref(false);
const saveError = ref('');

const tone = computed(() => {
  if (health.value.state === 'ok') return 'ok';
  if (health.value.state === 'bad') return 'bad';
  return settings.value?.configured ? 'wait' : 'off';
});
const headline = computed(() => {
  if (health.value.state === 'ok') return '已连接';
  if (health.value.state === 'bad') return '连不上或令牌无效';
  if (health.value.state === 'checking') return '正在检查…';
  return '还没有设置令牌';
});
const sourceText = computed(() => {
  switch (settings.value?.source) {
    case 'console':
      return `在这个后台填写的（尾号 ${settings.value.hint ?? '未知'}）`;
    case 'env':
      return `来自服务器配置文件（尾号 ${settings.value.hint ?? '未知'}）。在这里填写后会优先使用后台的`;
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
  try {
    const result = await api.saveGatewayToken(token.value);
    token.value = '';
    reveal.value = false;
    notify(`令牌已验证并保存（尾号 ${result.hint}），马上生效，不需要重启`);
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
    <PageHead title="网关连接" desc="本服务通过一个管理令牌调用 AI 网关（New API）：为用户开通账号、增减额度、核对用量。令牌在这里填写，不用放进服务器的配置文件。" />

    <div v-if="loadError" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ loadError }}</span></div>

    <div v-if="settings" class="cols">
      <section class="panel status" :class="tone" aria-label="连接状态">
        <div class="orb"><span class="ring" aria-hidden="true"></span><Icon name="plug" :size="26" :stroke="1.7" /></div>
        <div class="line">
          <h2>{{ headline }}</h2>
          <button v-if="settings.configured" type="button" class="btn small quiet" :disabled="health.state === 'checking'" @click="test"><Icon name="refresh" :size="14" />重新检查</button>
        </div>
        <p v-if="health.state === 'bad'" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ health.message }}</span></p>
        <p v-if="settings.problem" class="notice warn" role="alert"><Icon name="alert" :size="17" /><span>{{ settings.problem }}</span></p>
        <p v-if="!settings.configured" class="notice warn"><Icon name="alert" :size="17" /><span>没有令牌时，用户无法开通，也不能调用 AI。填写右边的令牌后立即生效。</span></p>

        <dl>
          <div><dt>网关地址</dt><dd class="mono">{{ settings.base_url }}</dd></div>
          <div><dt>当前令牌</dt><dd>{{ sourceText }}</dd></div>
          <div v-if="settings.updated_at"><dt>更新时间</dt><dd class="num">{{ formatTime(settings.updated_at) }}</dd></div>
        </dl>
      </section>

      <form class="panel form" @submit.prevent="save">
        <h2>{{ settings.configured ? '更换令牌' : '填写令牌' }}</h2>

        <ol class="steps">
          <li><span class="n">1</span><span>登录 AI 网关（New API）的后台，进入个人设置里的访问令牌。</span></li>
          <li><span class="n">2</span><span>新建一个令牌，权限只勾选「用户」的<strong>查看</strong>和<strong>编辑</strong>，其余都不要勾。</span></li>
          <li><span class="n">3</span><span>复制令牌，粘贴到下面。保存前会先向网关验证，填错不会影响现在正在用的令牌。</span></li>
        </ol>

        <div v-if="saveError" class="notice error" role="alert"><Icon name="circle-alert" :size="17" /><span>{{ saveError }}</span></div>

        <div class="field">
          <label for="gw-token">管理令牌</label>
          <div class="token-row">
            <input id="gw-token" v-model="token" class="input mono" :type="reveal ? 'text' : 'password'" autocomplete="off" autocapitalize="off" spellcheck="false" placeholder="nap_…" />
            <button type="button" class="btn icon" :aria-label="reveal ? '隐藏令牌' : '显示令牌'" @click="reveal = !reveal"><Icon :name="reveal ? 'eye-off' : 'eye'" :size="17" /></button>
          </div>
          <span class="hint">保存后令牌会加密存放，之后在这里只能看到尾号，看不到完整内容。</span>
        </div>

        <div><button type="submit" class="btn primary" :disabled="saving || !token.trim()"><Icon name="shield" :size="16" />{{ saving ? '验证中…' : '验证并保存' }}</button></div>
      </form>
    </div>
    <div v-else-if="!loadError" class="cols"><i class="skel" style="height: 280px; border-radius: 14px"></i><i class="skel" style="height: 280px; border-radius: 14px"></i></div>
  </div>
</template>

<style scoped>
.notice { margin: 0; }
.cols { display: grid; grid-template-columns: minmax(0, 5fr) minmax(0, 6fr); gap: 22px; align-items: start; }

.status { --c: var(--faint); padding: 28px 26px 24px; display: flex; flex-direction: column; gap: 14px; }
.status.ok { --c: var(--tide); }
.status.bad { --c: var(--rose); }
.status.wait, .status.off { --c: var(--amber-bright); }
.orb { position: relative; display: grid; place-items: center; width: 64px; height: 64px; border-radius: 20px; background: color-mix(in srgb, var(--c) 12%, #fff); color: var(--c); }
.ring { position: absolute; inset: -6px; border-radius: 25px; border: 2px solid color-mix(in srgb, var(--c) 30%, transparent); }
.status.ok .ring { animation: ping 2.6s ease-out infinite; }
@keyframes ping { from { transform: scale(0.9); opacity: 1; } to { transform: scale(1.22); opacity: 0; } }
.line { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.line h2 { font-size: 20px; letter-spacing: -0.01em; }
dl { margin: 6px 0 0; display: flex; flex-direction: column; gap: 10px; padding-top: 16px; border-top: 1px solid var(--line); }
dl > div { display: grid; grid-template-columns: 74px 1fr; gap: 12px; align-items: baseline; }
dt { color: var(--muted); font-weight: 600; font-size: 13px; }
dd { margin: 0; overflow-wrap: anywhere; }

.form { padding: 26px; display: flex; flex-direction: column; gap: 18px; }
.steps { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 10px; color: var(--muted); }
.steps li { display: flex; gap: 12px; align-items: flex-start; }
.steps strong { color: var(--text); }
.n { flex: none; display: grid; place-items: center; width: 22px; height: 22px; border-radius: 50%; background: var(--tide-soft); color: var(--tide); font-size: 12px; font-weight: 700; margin-top: 1px; }
.token-row { display: flex; gap: 8px; }

@media (max-width: 960px) { .cols { grid-template-columns: 1fr; } }
</style>
