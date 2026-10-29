<script setup lang="ts">
// 更新审查：先看新版本与已安装版本的文件差异，再确认；旧版本保留为备份。
import { computed, ref, watch } from 'vue';
import { NAlert, NModal } from 'naive-ui';
import { ArrowRight, RefreshCw } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { SkillReview } from './catalog.ts';
import { diffFiles, shortDigest, type UpdateInfo } from './market.ts';
import './store-cards.css';

export interface InstalledPackage { slug: string; name: string; namespace?: string; version: string; marketKind?: string | null; sha256?: string; files?: { path: string; sha256: string }[] }
const props = defineProps<{ pkg: InstalledPackage | null; info: UpdateInfo | null }>();
const emit = defineEmits<{ close: []; done: [message: string] }>();
const review = ref<SkillReview | null>(null);
const loading = ref(false), busy = ref(false), error = ref('');
let token = 0;
const changes = computed(() => review.value ? diffFiles(props.pkg?.files ?? [], review.value.manifest.files) : []);
const count = (kind: string) => changes.value.filter(c => c.change === kind).length;
const label = { added: '新增', changed: '修改', removed: '删除' } as const;
const bytes = (n?: number) => n === undefined ? '' : n < 1024 ? `${n} B` : `${(n / 1024).toFixed(1)} KB`;
const newScripts = computed(() => review.value?.scripts.filter(s => changes.value.some(c => c.path === s.path && c.change !== 'removed')) ?? []);
const kind = computed(() => props.pkg?.marketKind === 'expert' ? 'experts' : 'skills');

async function load() {
  const { pkg, info } = props, id = ++token;
  review.value = null; error.value = '';
  if (!pkg || !info || !pkg.namespace) return;
  loading.value = true;
  try {
    const result = await invoke<SkillReview>('ai_market_inspect', { kind: kind.value, namespace: pkg.namespace, slug: pkg.slug, version: info.latest });
    if (id === token) review.value = result;
  } catch (e) { if (id === token) error.value = String(e); }
  finally { if (id === token) loading.value = false; }
}
watch(() => [props.pkg?.slug, props.info?.latest], () => void load(), { immediate: true });

async function update() {
  const { pkg, info } = props, r = review.value;
  if (!pkg || !info || !r || busy.value) return;
  busy.value = true; error.value = '';
  try {
    await invoke('ai_market_update', { kind: kind.value, namespace: pkg.namespace, slug: pkg.slug, version: info.latest, sha256: r.sha256 });
    emit('done', `「${pkg.name}」已更新到 v${info.latest}，旧版本已备份`);
  } catch (e) { error.value = String(e); }
  finally { busy.value = false; }
}
function close() { if (!busy.value) emit('close'); }
</script>

<template>
  <NModal :show="!!pkg && !!info" preset="card" :title="`更新「${pkg?.name}」`" class="store-dialog update-dialog" :mask-closable="!busy" :closable="!busy" style="width:680px;max-width:calc(100vw - 36px)" @update:show="!$event && close()">
    <template v-if="pkg && info">
      <div class="detail-tags"><span class="store-tag">v{{ pkg.version }}</span><ArrowRight :size="14" class="to" /><span class="store-tag now">v{{ info.latest }}</span><span class="store-tag">小伴市场</span></div>
      <div v-if="loading" class="review-loading"><RefreshCw :size="17" />正在读取新版本的文件与脚本…</div>
      <NAlert v-if="error" type="error" :show-icon="false">{{ error }} <button v-if="!review" class="text-action" :disabled="busy" @click="load">重试</button></NAlert>
      <template v-if="review">
        <h4>文件变化 <small>{{ count('changed') }} 个修改 · {{ count('added') }} 个新增 · {{ count('removed') }} 个删除</small></h4>
        <ul class="review-files">
          <li v-for="c in changes" :key="c.path"><div><span class="change" :class="c.change">{{ label[c.change] }}</span><code :class="{ gone: c.change === 'removed' }">{{ c.path }}</code><span>{{ bytes(c.size) }}{{ c.executableScript ? ' · 脚本' : '' }}</span></div></li>
          <li v-if="!changes.length"><div><span>文件内容与已安装版本相同，只有版本号不同。</span></div></li>
        </ul>
        <details v-for="s in newScripts" :key="s.path" class="script-preview"><summary>{{ s.path }} · 脚本预览{{ s.previewTruncated ? '（已截断）' : '' }}</summary><pre>{{ s.preview || '此脚本暂无文本预览，请在确认来源后再更新。' }}</pre></details>
        <div class="digest-line"><span>清单摘要</span><code>{{ shortDigest(pkg.sha256 || '') || '—' }}</code><ArrowRight :size="12" /><code>{{ shortDigest(review.sha256) }}</code></div>
        <p class="detail-note">先完整校验新版本，再替换已安装的版本；旧版本保留为备份，更新失败时自动恢复。新增或修改的脚本仍需在运行时单独确认。</p>
      </template>
      <div class="dialog-actions"><button class="store-secondary" :disabled="busy" @click="close">暂不更新</button><button class="store-primary" :disabled="!review || busy" @click="update">{{ busy ? '正在更新…' : `更新到 v${info.latest}` }}</button></div>
    </template>
  </NModal>
</template>

<style scoped>
.to{color:var(--ui-text-tertiary);align-self:center}
.store-tag.now{background:var(--ui-soft);color:var(--ui-ink)}
.change{display:inline-flex;align-items:center;height:20px;margin-right:8px;padding:0 7px;border-radius:6px;font-size:11px;background:var(--ui-control);color:var(--ui-text-secondary)}
.change.added{background:color-mix(in srgb,var(--ui-success) 14%,transparent);color:var(--ui-success-text)}
.change.changed{background:color-mix(in srgb,var(--ui-warning) 16%,transparent);color:var(--ui-warning-text)}
.change.removed{background:color-mix(in srgb,var(--ui-danger) 12%,transparent);color:var(--ui-danger)}
.gone{text-decoration:line-through;color:var(--ui-text-tertiary)}
.digest-line{display:flex;align-items:center;gap:8px;margin-top:14px;padding:12px 14px;border-radius:12px;box-shadow:0 0 0 1px var(--ui-border);font-size:12px;color:var(--ui-text-secondary)}
.digest-line code{font-size:11.5px;color:var(--ui-text-primary)}
</style>
