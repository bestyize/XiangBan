<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { api, errorMessage } from '../api';
import { formatTime } from '../format';
import { notify } from '../toast';
import type { GatewaySettings, ModelOption } from '../types';
import Icon from '../components/Icon.vue';
import PageHead from '../components/PageHead.vue';

const emit = defineEmits<{ changed: [] }>();

const settings = ref<GatewaySettings | null>(null);
const loadError = ref('');

type Health =
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'ok'; ms: number }
  | { state: 'bad'; message: string };
const health = ref<Health>({ state: 'idle' });

const token = ref('');
const reveal = ref(false);
const saving = ref(false);
const saveError = ref('');

const models = ref<ModelOption[]>([]);
const defaultRatio = ref(37.5);
const modelsLoading = ref(false);
const onlyWarn = ref(false);

const tone = computed(() => {
  if (health.value.state === 'ok') return 'ok';
  if (health.value.state === 'bad') return 'bad';
  return settings.value?.configured ? 'wait' : 'unset';
});

const headline = computed(() => {
  if (health.value.state === 'ok') return '已连接到 New API 网关';
  if (health.value.state === 'bad') return '网关连接失败或令牌无效';
  if (health.value.state === 'checking') return '正在探测连接…';
  return '尚未配置管理令牌';
});

const sourceText = computed(() => {
  switch (settings.value?.source) {
    case 'console':
      return `后台加密存储（尾号 ${settings.value.hint ?? '未知'}）`;
    case 'env':
      return `服务器环境变量（尾号 ${settings.value.hint ?? '未知'}），后台填写后优先使用后台值`;
    default:
      return '尚未设置';
  }
});

const tokenCheck = computed(() => {
  const v = token.value.trim();
  if (!v) return { text: '保存前会先在线验证', tone: '' };
  if (!v.startsWith('nap_')) return { text: '建议使用 nap_ 开头的专用访问令牌', tone: 'warn' };
  return { text: '格式有效，可验证保存', tone: 'ok' };
});

const warnModels = computed(() => models.value.filter((m) => m.ratio_warning));
const shownModels = computed(() => (onlyWarn.value ? warnModels.value : models.value));

async function loadModels() {
  modelsLoading.value = true;
  try {
    const res = await api.models();
    models.value = res.items;
    defaultRatio.value = res.default_ratio;
  } catch {
    models.value = [];
  } finally {
    modelsLoading.value = false;
  }
}

async function test() {
  health.value = { state: 'checking' };
  const t0 = performance.now();
  try {
    const result = await api.testGateway();
    const ms = Math.max(1, Math.round(performance.now() - t0));
    if (result.ok) {
      health.value = { state: 'ok', ms };
      loadModels();
    } else {
      health.value = { state: 'bad', message: result.message ?? '网关没有接受这个令牌' };
    }
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
    notify(`令牌已验证并加密保存（尾号 ${result.hint}），立即生效`);
    settings.value = (await api.settings()).newapi;
    await test();
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
    <PageHead
      title="网关连接"
      desc="配置本服务访问上游 New API 的管理令牌，用于自动开通用户、同步增减额度与拉取可用模型定价。"
    />

    <div v-if="loadError" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ loadError }}</span></div>

    <template v-if="settings">
      <section class="card gw-hero" aria-label="连接状态">
        <div class="gw-orb" :class="tone"><Icon name="plug" :size="22" /></div>
        <div class="gw-hero-info">
          <h2>
            <span>{{ headline }}</span>
            <span v-if="health.state === 'ok'" class="tag ok num">探测正常 · {{ health.ms }}ms</span>
            <span v-else-if="health.state === 'bad'" class="tag danger">连接失败</span>
            <span v-else-if="!settings.configured" class="tag warn">待配置</span>
          </h2>
          <div class="gw-meta-row">
            <span>地址：<code class="mono">{{ settings.base_url }}</code></span>
            <span>令牌来源：{{ sourceText }}</span>
            <span v-if="settings.updated_at">更新时间：<span class="num">{{ formatTime(settings.updated_at) }}</span></span>
          </div>
        </div>
        <button v-if="settings.configured" type="button" class="btn" :disabled="health.state === 'checking'" @click="test">
          <Icon name="refresh" :size="14" />{{ health.state === 'checking' ? '探测中…' : '重新探测' }}
        </button>
      </section>

      <div v-if="health.state === 'bad'" class="notice error" style="margin-bottom: 14px" role="alert">
        <Icon name="circle-alert" :size="16" /><span>{{ health.message }}</span>
      </div>
      <div v-if="settings.problem" class="notice warn" style="margin-bottom: 14px" role="alert">
        <Icon name="alert" :size="16" /><span>{{ settings.problem }}</span>
      </div>
      <div v-if="!settings.configured" class="notice warn" style="margin-bottom: 14px">
        <Icon name="alert" :size="16" /><span>未配置管理令牌时，新用户无法开通，也无法同步额度。在下方填写后立即生效。</span>
      </div>

      <div class="gw-cols">
        <form class="card gw-panel" @submit.prevent="save">
          <h3>{{ settings.configured ? '更换管理令牌' : '填写 New API 管理令牌' }}</h3>
          <div v-if="saveError" class="notice error" role="alert"><Icon name="circle-alert" :size="16" /><span>{{ saveError }}</span></div>

          <div class="field">
            <label for="gw-token">
              <span>访问令牌（nap_ 前缀）</span>
              <span class="check-hint" :class="tokenCheck.tone">{{ tokenCheck.text }}</span>
            </label>
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
              <button type="button" class="btn icon" :aria-label="reveal ? '隐藏令牌' : '显示令牌'" @click="reveal = !reveal">
                <Icon :name="reveal ? 'eye-off' : 'eye'" :size="15" />
              </button>
            </div>
            <span class="hint">使用 AES-256-GCM 加密存库并热生效，无需重启服务；验证不通过不会覆盖当前有效令牌。</span>
          </div>

          <div>
            <button type="submit" class="btn primary" :disabled="saving || !token.trim()">
              <Icon name="shield" :size="14" />{{ saving ? '验证中…' : '验证并保存' }}
            </button>
          </div>
        </form>

        <div class="card gw-panel">
          <h3>最小权限配置指引</h3>
          <ol class="step-list">
            <li><span class="idx">1</span><span>登录 New API 后台，使用专用管理员账号进入「个人设置 → 访问令牌」。</span></li>
            <li><span class="idx">2</span><span>新建令牌时<strong>仅勾选用户管理</strong>的 <code class="mono">user:read</code> 与 <code class="mono">user:write</code> 权限，禁止使用 root 令牌。</span></li>
            <li><span class="idx">3</span><span>上线前请确认 New API 的「新用户注册初始额度」已设为 <strong>0</strong>，避免白送额度触发对账异常。</span></li>
          </ol>
        </div>
      </div>

      <!-- 复用 /api/admin/models 的模型定价健康自检 -->
      <section v-if="models.length || modelsLoading" class="section">
        <div class="section-head">
          <h2>
            <span>已同步模型与定价自检</span>
            <span class="tag num">共 {{ models.length }} 个模型</span>
            <span v-if="warnModels.length" class="tag warn num">{{ warnModels.length }} 个疑似默认倍率</span>
            <span v-else-if="models.length" class="tag ok">倍率全部已配置</span>
          </h2>
          <div class="seg-pills">
            <button type="button" :class="{ active: !onlyWarn }" @click="onlyWarn = false">全部模型</button>
            <button type="button" :class="{ active: onlyWarn }" @click="onlyWarn = true">仅看默认倍率预警</button>
          </div>
        </div>

        <div v-if="modelsLoading" class="model-audit-grid">
          <i v-for="n in 6" :key="n" class="skel" style="height: 62px"></i>
        </div>
        <div v-else-if="shownModels.length" class="model-audit-grid">
          <div v-for="m in shownModels" :key="m.model_name" class="ma-item" :class="{ warn: m.ratio_warning }">
            <div class="mono truncate ma-name" :title="m.model_name">{{ m.model_name }}</div>
            <div class="ma-tags">
              <span class="tag num">输入 {{ m.model_ratio }}</span>
              <span class="tag num">输出 ×{{ m.completion_ratio }}</span>
              <span v-if="m.ratio_warning" class="tag warn">默认 {{ defaultRatio }}</span>
            </div>
          </div>
        </div>
        <div v-else class="card" style="padding: 28px; text-align: center">
          <span class="hint">当前没有默认倍率预警模型。</span>
        </div>
      </section>
    </template>

    <div v-else-if="!loadError" class="gw-cols">
      <i class="skel" style="height: 220px"></i>
      <i class="skel" style="height: 220px"></i>
    </div>
  </div>
</template>

<style scoped>
.gw-hero {
  padding: 18px 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.gw-orb {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  flex: none;
  display: grid;
  place-items: center;
  background: var(--canvas-soft);
  color: var(--ink-2);
  border: 1px solid var(--line);
}
.gw-orb.ok { background: var(--ok-soft); color: var(--ok); border-color: var(--ok-line); }
.gw-orb.bad { background: var(--danger-soft); color: var(--danger); border-color: var(--danger-line); }
.gw-orb.unset, .gw-orb.wait { background: var(--warn-soft); color: var(--warn); border-color: var(--warn-line); }

.gw-hero-info { flex: 1; min-width: 240px; }
.gw-hero-info h2 { font-size: 16px; font-weight: 700; display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.gw-meta-row { display: flex; flex-wrap: wrap; gap: 6px 20px; margin-top: 6px; font-size: 12.5px; color: var(--ink-2); }

.gw-cols { display: grid; grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr); gap: 14px; align-items: start; }
.gw-panel { padding: 20px; display: flex; flex-direction: column; gap: 14px; }
.gw-panel h3 { font-size: 14.5px; font-weight: 680; }
.token-row { display: flex; gap: 8px; }
.check-hint { font-size: 11.5px; font-weight: 500; color: var(--ink-3); }
.check-hint.ok { color: var(--ok); }
.check-hint.warn { color: var(--warn); }

.step-list { display: flex; flex-direction: column; gap: 11px; }
.step-list li { display: flex; gap: 10px; align-items: flex-start; font-size: 13px; color: var(--ink-2); }
.step-list strong { color: var(--ink); }
.idx {
  width: 20px;
  height: 20px;
  border-radius: 6px;
  flex: none;
  background: var(--canvas-soft);
  border: 1px solid var(--line);
  font-size: 11px;
  font-weight: 700;
  color: var(--ink);
  display: grid;
  place-items: center;
  margin-top: 1px;
}

.model-audit-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(235px, 1fr));
  gap: 8px;
}
.ma-item {
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px solid var(--line);
  background: var(--surface);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ma-item.warn { border-color: var(--warn-line); background: var(--warn-soft); }
.ma-name { font-weight: 650; }
.ma-tags { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }

@media (max-width: 960px) {
  .gw-cols { grid-template-columns: 1fr; }
}
</style>
