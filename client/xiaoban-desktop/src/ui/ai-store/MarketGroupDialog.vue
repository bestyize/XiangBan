<script setup lang="ts">
// 专家团 / 技能套件：一次审查全部成员，整组安装或整组撤销。
import { computed, ref, watch } from 'vue';
import { NAlert, NModal } from 'naive-ui';
import { Check, Download, RefreshCw } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { Skill } from './catalog.ts';
import { shortDigest, variantLabel, type GroupReview } from './market.ts';
import './store-cards.css';

const props = defineProps<{ item: Skill | null; kind: 'skills' | 'experts' }>();
const emit = defineEmits<{ close: []; done: [message: string] }>();
const review = ref<GroupReview | null>(null);
const loading = ref(false), busy = ref(false), error = ref('');
let token = 0;
const pending = computed(() => review.value?.members.filter(m => m.state === 'pending') ?? []);
const files = computed(() => pending.value.reduce((n, m) => n + m.manifest.fileCount, 0));
const bytes = (n: number) => n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(1)} MB`;
const total = computed(() => pending.value.reduce((n, m) => n + m.manifest.totalBytes, 0));
const unit = computed(() => props.kind === 'experts' ? '位专家' : '个技能');

async function load() {
  const item = props.item, id = ++token;
  review.value = null; error.value = '';
  if (!item?.namespace) return;
  loading.value = true;
  try {
    const result = await invoke<GroupReview>('ai_market_group_inspect', { kind: props.kind, namespace: item.namespace, slug: item.slug });
    if (id === token) review.value = result;
  } catch (e) { if (id === token) error.value = String(e); }
  finally { if (id === token) loading.value = false; }
}
watch(() => props.item, () => void load(), { immediate: true });

async function install() {
  const item = props.item, r = review.value;
  if (!item || !r || busy.value || !pending.value.length) return;
  busy.value = true; error.value = '';
  try {
    const result = await invoke<{ installed: number; skipped: number }>('ai_market_group_install', { kind: props.kind, namespace: item.namespace, slug: item.slug, sha256: r.sha256 });
    emit('done', `已安装「${item.name}」的 ${result.installed} ${unit.value}${result.skipped ? `，跳过 ${result.skipped} 个已安装` : ''}`);
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
function close() { if (!busy.value) emit('close'); }
</script>

<template>
  <NModal :show="!!item" preset="card" :title="item?.name" class="store-dialog group-dialog" :mask-closable="!busy" :closable="!busy" style="width:720px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
    <template v-if="item">
      <div class="detail-tags"><span class="store-tag">{{ variantLabel(item.variant) || '组合' }}</span><span class="store-tag">小伴市场</span><span v-if="review" class="store-tag">{{ review.members.length }} {{ unit }}</span></div>
      <p>{{ item.description }}</p>
      <div v-if="loading" class="review-loading"><RefreshCw :size="17" />正在逐个读取成员的文件与脚本…</div>
      <NAlert v-if="error" type="error" :show-icon="false">{{ error }} <button v-if="!review" class="text-action" :disabled="busy" @click="load">重试</button></NAlert>
      <template v-if="review">
        <h4>成员 <small>{{ review.members.length - pending.length }} 个已安装，将跳过</small></h4>
        <ul class="review-files group-members">
          <li v-for="m in review.members" :key="m.slug">
            <div><span><b>{{ m.name || m.slug }}</b> <code>v{{ m.manifest.version }}</code> · {{ m.manifest.fileCount }} 个文件</span><span class="member-state" :class="{ done: m.state === 'installed' }"><Check v-if="m.state === 'installed'" :size="11" :stroke-width="3" />{{ m.state === 'installed' ? '已安装 · 跳过' : '待安装' }}</span></div>
            <details v-if="m.state === 'pending'"><summary>查看说明与文件清单</summary>
              <pre class="review-content">{{ m.content }}</pre>
              <ul class="review-files inner"><li v-for="f in m.manifest.files" :key="f.path"><div><code>{{ f.path }}</code><span>{{ bytes(f.size) }}{{ f.executableScript ? ' · 脚本' : '' }}</span></div><small>SHA256 {{ f.sha256 }}</small></li></ul>
              <details v-for="s in m.scripts" :key="s.path" class="script-preview"><summary>{{ s.path }} · 脚本预览{{ s.previewTruncated ? '（已截断）' : '' }}</summary><pre>{{ s.preview || '此脚本暂无文本预览，请在确认来源后再安装。' }}</pre></details>
            </details>
          </li>
        </ul>
        <div class="group-summary"><span>将安装 {{ pending.length }} {{ unit }} · 共 {{ files }} 个文件 · {{ bytes(total) }}</span><span>整组摘要 <code>{{ shortDigest(review.sha256) }}</code></span></div>
        <p class="detail-note">全部通过校验后一起生效；任何一个失败，会撤销本次新增的内容，不影响已安装的。安装不会执行脚本；脚本运行时需单独确认，并使用本机账户权限。</p>
        <div v-if="!pending.length" class="installed-detail"><Check :size="17" /><p>这个组合的成员都已安装。</p></div>
      </template>
      <div class="dialog-actions"><button class="store-secondary" :disabled="busy" @click="close">取消</button><button class="store-primary" :disabled="!review || !pending.length || busy" @click="install">{{ busy ? '正在安装…' : `安装${variantLabel(item.variant) || '组合'}（${pending.length}）` }}<Download :size="14" /></button></div>
    </template>
  </NModal>
</template>

<style scoped>
.group-members li>div{align-items:center}
.group-members b{font-weight:600;font-size:13px}
.member-state{display:inline-flex;align-items:center;gap:4px;height:22px;padding:0 8px;border-radius:6px;background:var(--ui-soft);color:var(--ui-ink);font-size:11.5px;white-space:nowrap}
.member-state.done{background:var(--ui-control);color:var(--ui-text-secondary)}
.group-members details{margin:6px 0 2px 2px;font-size:12px}
.group-members .inner{margin-top:8px;box-shadow:none;background:var(--ui-surface-sunken)}
.group-summary{display:flex;justify-content:space-between;gap:12px;flex-wrap:wrap;margin-top:14px;padding:12px 14px;border-radius:12px;box-shadow:0 0 0 1px var(--ui-border);font-size:12px;color:var(--ui-text-secondary)}
.group-summary code{font-size:11.5px}
</style>
