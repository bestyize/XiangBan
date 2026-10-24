<script setup lang="ts">
// 网页点「在小伴中安装」唤起后的确认窗口。链接只能请求展示，不会安装；
// 链接里的版本和摘要先与市场的记录比对，再进入完整的审查。
import { computed, ref, watch } from 'vue';
import { NAlert, NModal } from 'naive-ui';
import { ArrowRight, Link2, RefreshCw } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import { judgeLink, kindLabel, shortDigest, variantLabel, type LinkVerdict, type MarketDetail, type MarketLink } from './market.ts';
import './store-cards.css';

const props = defineProps<{ link: MarketLink | null }>();
const emit = defineEmits<{ close: []; go: [detail: MarketDetail, version: string | null] }>();
const detail = ref<MarketDetail | null>(null);
const verdict = ref<LinkVerdict | null>(null);
const loading = ref(false), error = ref('');
let token = 0;

async function load() {
  const link = props.link, id = ++token;
  detail.value = null; verdict.value = null; error.value = '';
  if (!link) return;
  loading.value = true;
  try {
    const result = await invoke<MarketDetail>('ai_market_detail', { kind: link.kind, namespace: link.namespace, slug: link.slug });
    if (id !== token) return;
    detail.value = result;
    verdict.value = judgeLink(link, result);
  } catch (e) { if (id === token) error.value = String(e); }
  finally { if (id === token) loading.value = false; }
}
watch(() => props.link, () => void load(), { immediate: true });

const item = computed(() => detail.value?.item);
const refused = computed(() => verdict.value?.state === 'mismatch' || verdict.value?.state === 'missing');
const warning = computed(() => {
  const v = verdict.value, link = props.link;
  if (!v || !link) return '';
  if (v.state === 'newer') return `链接指定的 v${link.version} 不是最新版本，小伴市场当前为 v${v.latest}。将为你打开 v${v.latest} 的审查。`;
  if (v.state === 'mismatch') return '链接中的摘要与小伴市场当前的记录不一致，已拒绝安装。链接可能被改动，或该版本已被替换。';
  if (v.state === 'missing') return link.version ? `小伴市场里没有 v${link.version}，或它已被撤回。` : '这个条目暂时没有可安装的版本。';
  return '';
});
const goLabel = computed(() => verdict.value?.state === 'newer' ? `查看 v${verdict.value.version} 并安装` : '查看并安装');
const title = computed(() => props.link?.action === 'open' ? '来自网页的查看请求' : '来自网页的安装请求');
</script>

<template>
  <NModal :show="!!link" preset="card" :title="title" class="store-dialog link-dialog" style="width:520px;max-width:calc(100vw - 36px)" @update:show="!$event && emit('close')">
    <template v-if="link">
      <div v-if="loading" class="review-loading"><RefreshCw :size="17" />正在向小伴市场核对…</div>
      <NAlert v-if="error" type="error" :show-icon="false">{{ error }} <button class="text-action" @click="load">重试</button></NAlert>
      <template v-if="item">
        <div class="link-card"><div class="link-icon"><Link2 :size="20" /></div><div><b>{{ item.name }}</b><span>{{ item.description }}</span></div></div>
        <dl class="link-facts">
          <dt>类型</dt><dd>{{ kindLabel(link.kind) }}<template v-if="variantLabel(item.variant)"> · {{ variantLabel(item.variant) }}</template></dd>
          <dt>来源</dt><dd>小伴市场{{ item.source === 'official' ? ' · 官方' : '' }}</dd>
          <dt>链接指定</dt><dd>{{ link.version ? 'v' + link.version : '最新版本' }}</dd>
          <template v-if="link.sha256"><dt>清单摘要</dt><dd><code>{{ shortDigest(link.sha256) }}</code></dd></template>
        </dl>
        <NAlert v-if="warning" :type="verdict?.state === 'newer' ? 'warning' : 'error'" :show-icon="false" class="link-warning">{{ warning }}</NAlert>
        <p class="detail-note">网页只能请求打开这个确认窗口，不会自动安装。下一步仍会展示完整的文件清单和脚本预览，由你确认。</p>
      </template>
      <div class="dialog-actions"><button class="store-secondary" @click="emit('close')">{{ refused ? '关闭' : '取消' }}</button><button v-if="detail && verdict && !refused" class="store-primary" @click="emit('go', detail, verdict.version)">{{ link.action === 'open' ? '打开' : goLabel }}<ArrowRight :size="14" /></button></div>
    </template>
  </NModal>
</template>

<style scoped>
.link-card{display:flex;gap:12px;align-items:center;padding:14px;margin-top:6px;border-radius:14px;box-shadow:0 0 0 1px var(--ui-border)}
.link-card b{display:block;font-size:14.5px}
.link-card span{display:block;margin-top:2px;font-size:12px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:360px}
.link-icon{width:40px;height:40px;border-radius:12px;display:grid;place-items:center;flex:none;background:var(--ui-soft);color:var(--ui-ink)}
.link-facts{display:grid;grid-template-columns:72px 1fr;gap:8px 12px;margin:14px 0 0;padding:14px;border-radius:12px;box-shadow:0 0 0 1px var(--ui-border);font-size:12.5px}
.link-facts dt{color:var(--ui-text-tertiary)}
.link-facts dd{margin:0;color:var(--ui-text-primary);overflow-wrap:anywhere}
.link-warning{margin-top:14px}
</style>
