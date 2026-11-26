<script setup lang="ts">
import StoreDialog from './StoreDialog.vue';
import StoreRemoveAction from './StoreRemoveAction.vue';
// 专家团 / 技能套件：一次审查全部成员，整组安装或整组撤销。
import { computed, ref, watch } from 'vue';
import { NAlert } from 'naive-ui';
import { Check, Download, Layers, RefreshCw, ShieldCheck, Users } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { Skill } from './catalog.ts';
import { hueStyle } from './hues.ts';
import { shortDigest, variantLabel, type GroupReview } from './market.ts';
import './store-cards.css';

const props = defineProps<{ item: Skill | null; kind: 'skills' | 'experts' }>();
const emit = defineEmits<{ close: []; done: [message: string]; changed: [] }>();
const review = ref<GroupReview | null>(null);
const loading = ref(false), busy = ref(false), error = ref('');
let token = 0;
const pending = computed(() => review.value?.members.filter(m => m.state === 'pending') ?? []);
const installed = computed(() => review.value?.members.filter(m => m.state === 'installed') ?? []);
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
async function remove() {
  const item = props.item, members = [...installed.value];
  if (!item || busy.value || !members.length) return;
  busy.value = true; error.value = '';
  let removed = 0;
  function markRemoved(slug: string) {
    if (review.value) review.value = { ...review.value, members: review.value.members.map(member => member.slug === slug ? { ...member, state: 'pending' } : member) };
  }
  try {
    const local = await invoke<Skill[]>('ai_store_installed');
    // Review pins ownership and version. A package replaced since the preview
    // must not be removed merely because its slug is the same.
    for (const member of members) {
      const saved = local.find(skill => skill.slug === member.slug);
      if (!saved) { markRemoved(member.slug); continue; }
      if (!saved.removable || !saved.market || saved.namespace !== member.namespace || saved.version !== member.manifest.version || saved.manifestSha256 !== member.sha256) throw new Error(`「${member.name || member.slug}」的安装来源或版本已变化，请重新读取后再移除。`);
    }
    for (const member of members) {
      if (!local.some(skill => skill.slug === member.slug)) continue;
      await invoke('ai_store_uninstall', { slug: member.slug });
      removed++; markRemoved(member.slug);
    }
    emit('done', `已移除「${item.name}」的 ${removed} ${unit.value}`);
  } catch (e) { error.value = `${removed ? `已移除 ${removed} ${unit.value}；` : ''}移除失败：${String(e)}`; }
  finally { busy.value = false; emit('changed'); }
}
function close() { if (!busy.value) emit('close'); }
</script>

<template>
  <StoreDialog :show="!!item" preset="card" :title="item?.name" hide-title class="store-dialog group-dialog" :mask-closable="!busy" :closable="!busy" style="width:680px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
    <template v-if="item">
      <div class="dlg-hero" :style="hueStyle(item.category ?? undefined, item.slug)">
        <div class="xc-ic"><Users v-if="item.variant === 'team' || kind === 'experts'" :size="24" :stroke-width="1.85" aria-hidden="true" /><Layers v-else :size="24" :stroke-width="1.85" aria-hidden="true" /></div>
        <div class="dlg-hero-main">
          <div class="dlg-hero-title"><h2>{{ item.name }}</h2></div>
          <p v-if="item.description" class="dlg-hero-desc">{{ item.description }}</p>
          <div class="dlg-meta-row">
            <span class="store-tag">{{ variantLabel(item.variant) || '组合' }}</span>
            <span class="store-tag">小伴市场</span>
            <span v-if="review" class="store-tag">共 {{ review.members.length }} {{ unit }}</span>
            <span v-if="review && installed.length" class="store-tag is-ok">{{ installed.length }} 项已安装将跳过</span>
            <span v-if="review && pending.length" class="store-tag">本次新增 {{ pending.length }} 项</span>
          </div>
        </div>
      </div>
      <div v-if="loading" class="review-loading"><RefreshCw :size="17" />正在逐个读取成员的文件与脚本…</div>
      <NAlert v-if="error" type="error" :show-icon="false">{{ error }} <button class="text-action" :disabled="busy" @click="load">重新读取</button></NAlert>
      <template v-if="review">
        <div class="dlg-section">
          <div class="dlg-section-head"><h4>包含成员</h4><span>{{ pending.length ? `将新增 ${files} 个文件 · ${bytes(total)}` : '全部成员已就绪' }}</span></div>
          <div class="group-members">
            <details v-for="m in review.members" :key="m.slug" class="member-item">
              <summary class="member-summary">
                <span class="member-ic"><Check v-if="m.state === 'installed'" :size="14" :stroke-width="2.6" /><Users v-else-if="kind === 'experts'" :size="14" /><Layers v-else :size="14" /></span>
                <span class="member-info"><b>{{ m.name || m.slug }}</b><small>v{{ m.manifest.version }} · {{ m.manifest.fileCount }} 个文件</small></span>
                <span class="member-state" :class="{ done: m.state === 'installed' }"><Check v-if="m.state === 'installed'" :size="11" :stroke-width="2.8" />{{ m.state === 'installed' ? '已安装 · 跳过' : '待安装' }}</span>
              </summary>
              <div v-if="m.state === 'pending'" class="member-expand">
                <pre v-if="m.content" class="review-content">{{ m.content }}</pre>
                <ul v-if="m.manifest.files.length" class="review-files inner"><li v-for="f in m.manifest.files" :key="f.path" :title="`SHA256 ${f.sha256}`"><div><code>{{ f.path }}</code><span>{{ bytes(f.size) }}{{ f.executableScript ? ' · 脚本' : '' }}</span></div></li></ul>
                <details v-for="s in m.scripts" :key="s.path" class="script-preview"><summary>{{ s.path }} · 脚本预览{{ s.previewTruncated ? '（已截断）' : '' }}</summary><pre>{{ s.preview || '此脚本暂无文本预览，请在确认来源后再安装。' }}</pre></details>
              </div>
              <div v-else class="member-expand"><p class="member-done-note">此成员已在本机安装，本次整组安装会自动跳过，不覆盖原版本。</p></div>
            </details>
          </div>
        </div>
        <div class="group-summary"><span class="audit-ok"><ShieldCheck :size="14" />整组原子安装，任一校验失败自动回滚</span><span>整组摘要 <code>{{ shortDigest(review.sha256) }}</code></span></div>
        <p class="detail-note">全部通过校验后一起生效；任何一个失败，会撤销本次新增的内容，不影响已安装的。安装不会执行脚本；脚本运行时需单独确认，并使用本机账户权限。</p>
        <div v-if="!pending.length" class="installed-detail"><Check :size="17" /><p>这个组合的成员都已安装。</p></div>
      </template>
      <StoreRemoveAction :name="item.name" :label="`移除已安装成员（${installed.length}）`" :removable="!!installed.length" :description="`将移除以下 ${installed.length} 个已安装成员，使用它们的其他专家工作流和进行中的任务可能受影响。之后可以重新安装。`" :busy="busy" @confirm="remove">
        <template #details><ul class="group-remove-members"><li v-for="member in installed" :key="member.slug">{{ member.name || member.slug }}</li></ul></template>
        <button class="store-secondary" :disabled="busy" @click="close">取消</button><button v-if="pending.length || !review" class="store-primary" :disabled="!review || !pending.length || busy" @click="install">{{ busy ? '正在安装…' : `安装${variantLabel(item.variant) || '组合'}（${pending.length}）` }}<Download :size="14" /></button>
      </StoreRemoveAction>
    </template>
  </StoreDialog>
</template>

<style scoped>
.group-members{display:flex;flex-direction:column;gap:8px}
.member-item{border-radius:12px;background:var(--ui-surface-sunken);box-shadow:inset 0 0 0 1px var(--ui-line);overflow:hidden}
.member-summary{list-style:none;display:flex;align-items:center;gap:10px;padding:11px 14px;cursor:pointer;user-select:none}
.member-summary::-webkit-details-marker{display:none}
.member-summary:hover{background:var(--ui-surface-hover)}
.member-ic{width:30px;height:30px;border-radius:8px;background:var(--ui-surface);box-shadow:inset 0 0 0 1px var(--ui-line);color:var(--ui-text-secondary);display:grid;place-items:center;flex:none}
.member-info{flex:1;min-width:0}
.member-info b{display:block;font-size:13px;font-weight:600;color:var(--ui-text-primary)}
.member-info small{font-size:11.5px;color:var(--ui-text-tertiary)}
.member-state{display:inline-flex;align-items:center;gap:4px;height:22px;padding:0 8px;border-radius:6px;background:var(--ui-surface);box-shadow:inset 0 0 0 1px var(--ui-border-strong);color:var(--ui-text-primary);font-size:11px;font-weight:500;white-space:nowrap}
.member-state.done{background:var(--ui-control);box-shadow:none;color:var(--ui-text-tertiary)}
.member-expand{padding:10px 14px 12px;border-top:1px solid var(--ui-line)}
.member-done-note{margin:0;font-size:12px;color:var(--ui-text-tertiary)}
.group-members .inner{margin-top:8px;background:var(--ui-surface)}
.group-summary{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap;margin-top:14px;padding:11px 14px;border-radius:11px;background:var(--ui-surface-sunken);box-shadow:inset 0 0 0 1px var(--ui-line);font-size:12px;color:var(--ui-text-secondary)}
.group-summary code{font-family:var(--ui-font-mono,monospace);font-size:11.5px;color:var(--ui-text-primary)}
.group-remove-members{margin:0 0 14px;padding-left:20px;color:var(--ui-text-secondary);font-size:12.5px;line-height:1.8}
</style>
