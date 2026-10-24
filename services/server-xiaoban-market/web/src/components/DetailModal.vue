<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ApiError, market } from '@/lib/api';
import { deepLink, formatCount, formatSize, formatTime, shortHash } from '@/lib/format';
import type { ItemDetail, Plural } from '@/lib/types';
import Avatar from './Avatar.vue';
import Icon from './Icon.vue';

const props = defineProps<{ kind: Plural; ns: string; slug: string }>();
const emit = defineEmits<{ close: [] }>();

const detail = ref<ItemDetail | null>(null);
const error = ref('');
const loading = ref(true);
const dialog = ref<HTMLElement | null>(null);

async function load() {
  loading.value = true;
  error.value = '';
  detail.value = null;
  try {
    detail.value = await market.detail(props.kind, props.ns, props.slug);
  } catch (e) {
    error.value = e instanceof ApiError && e.status === 404 ? '这个条目不存在，或已经下架。' : (e as Error).message;
  } finally {
    loading.value = false;
  }
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') emit('close');
}

onMounted(() => {
  load();
  document.addEventListener('keydown', onKey);
  document.body.style.overflow = 'hidden';
  dialog.value?.focus();
});
onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey);
  document.body.style.overflow = '';
});
watch(() => [props.kind, props.ns, props.slug], load);

const item = computed(() => detail.value?.item);
const latest = computed(() => detail.value?.versions.find((v) => !v.yanked) ?? null);
const SCRIPT = /\.(py|sh|js|mjs|cjs|ps1|rb)$/i;
const scripts = computed(() => (detail.value?.files ?? []).filter((f) => SCRIPT.test(f.path)));
const ORIGIN: Record<string, string> = { skillhub: 'SkillHub', modelscope: '魔搭社区', manual: '小伴整理' };
const SOURCE: Record<string, string> = { official: '官方', community: '社区', enterprise: '企业' };
const originName = computed(() => (item.value ? ORIGIN[item.value.origin] ?? item.value.origin : ''));

const action = computed(() => {
  const it = item.value;
  if (!it) return null;
  const kind = it.kind;
  const verb = kind === 'skill' ? 'install' : kind === 'connector' ? 'connect' : 'open';
  const label = kind === 'skill' ? '在小伴桌面中安装' : kind === 'connector' ? '在小伴桌面中连接' : '在小伴桌面中打开';
  return { href: deepLink(verb, kind, it.namespace, it.slug, latest.value?.version, latest.value?.manifestSha256), label };
});

const configJson = computed(() => {
  const configs = item.value?.metadata.configs;
  if (!configs?.length) return '';
  const servers = Object.fromEntries(configs.map((c) => [c.name, c.config]));
  return JSON.stringify({ mcpServers: servers }, null, 2);
});
</script>

<template>
  <div class="scrim" @mousedown.self="emit('close')">
    <div ref="dialog" class="dialog" role="dialog" aria-modal="true" tabindex="-1" :aria-label="item?.name ?? '详情'">
      <button class="close" type="button" aria-label="关闭" @click="emit('close')"><Icon name="close" :size="20" :stroke="1.8" /></button>

      <div v-if="loading" class="state"><span class="spinner" /> 加载中…</div>
      <div v-else-if="error" class="state"><div class="error-box">{{ error }}</div></div>

      <template v-else-if="item">
        <header class="head">
          <Avatar :name="item.name" :seed="item.slug" :icon-url="item.iconUrl" :size="item.kind === 'connector' ? 76 : 88" />
          <div class="head-main">
            <h1>{{ item.name }}</h1>
            <div class="actions">
              <a v-if="action" class="btn btn-primary" :href="action.href">{{ action.label }}</a>
              <a v-if="item.kind === 'skill' && latest?.downloadUrl" class="btn" :href="latest.downloadUrl" download>
                <Icon name="download" :size="16" />下载 ZIP · {{ formatSize(latest.size) }}
              </a>
            </div>
            <p v-if="item.kind === 'connector'" class="local-note">令牌只保存在你的电脑上，不会上传到小伴市场。</p>
          </div>
        </header>

        <div class="body">
          <p class="lead">{{ item.summary || '暂无简介' }}</p>
          <div v-if="item.tags.length" class="tags"><span v-for="t in item.tags" :key="t" class="chip">{{ t }}</span></div>

          <div class="stats">
            <div><b class="mono-num">{{ item.version ?? '—' }}</b><span>版本</span></div>
            <div><b class="mono-num"><Icon name="bolt" :size="18" /> {{ formatCount(item.heat) }}</b><span>热度</span></div>
            <div>
              <b>
                <Icon name="verified" :size="18" class="verified" />
                <a v-if="item.originUrl" :href="item.originUrl" target="_blank" rel="noopener noreferrer">{{ originName }}<Icon name="external" :size="14" /></a>
                <template v-else>{{ originName }}</template>
              </b>
              <span>来源 · {{ SOURCE[item.source] }}</span>
            </div>
          </div>

          <section v-if="item.kind === 'skill'">
            <h2><Icon name="shield" :size="20" />安装前请知道</h2>
            <div class="notice">
              <template v-if="scripts.length">包含 <b>{{ scripts.length }} 个脚本</b>（{{ scripts.map((s) => s.path).join('、') }}）。脚本不在沙箱中运行，可能访问你的文件和网络；运行前小伴桌面会逐次向你确认。</template>
              <template v-else>这个包不含可执行脚本。</template>
              <div>摘要只保证内容与下方清单一致，不代表安全。</div>
            </div>
            <dl v-if="latest" class="kv">
              <div><dt>清单摘要</dt><dd class="mono">{{ latest.manifestSha256 }}</dd></div>
              <div><dt>ZIP 摘要</dt><dd class="mono">{{ latest.packageSha256 }}</dd></div>
            </dl>
          </section>

          <section v-if="item.prompts.length">
            <h2><Icon name="bulb" :size="20" />{{ item.kind === 'expert' ? '专家帮你做' : '试试这样用' }}</h2>
            <ul class="prompts">
              <li v-for="p in item.prompts" :key="p"><span>“{{ p }}”</span><Icon name="chat" :size="22" /></li>
            </ul>
          </section>

          <section v-if="item.kind === 'connector' && configJson">
            <h2><Icon name="code" :size="20" />配置模板</h2>
            <pre class="code">{{ configJson }}</pre>
            <p class="hint">模板仅供阅读，不会自动运行；你在小伴桌面里填写凭据并确认后才会启动。</p>
          </section>

          <section v-if="item.kind === 'expert' && item.description">
            <h2><Icon name="list" :size="20" />工作流</h2>
            <pre class="text">{{ item.description }}</pre>
            <div v-if="item.metadata.dependencies?.length" class="deps">
              依赖技能：<span v-for="d in item.metadata.dependencies" :key="d.slug" class="chip mono">{{ d.slug }}</span>
            </div>
          </section>

          <section v-if="item.kind === 'skill' && detail && detail.files.length">
            <h2 class="split"><span><Icon name="file" :size="20" />文件清单</span><small>{{ detail.files.length }} 个文件</small></h2>
            <ul class="files">
              <li v-for="f in detail.files" :key="f.path"><span class="mono path">{{ f.path }}</span><span class="size">{{ formatSize(f.size) }}</span><span class="mono sha">{{ shortHash(f.sha256, 6) }}</span></li>
            </ul>
            <details v-if="item.description" class="skill-md"><summary>查看 SKILL.md</summary><pre class="text">{{ item.description }}</pre></details>
          </section>

          <section v-if="item.kind === 'skill' && detail && detail.versions.length">
            <h2><Icon name="refresh" :size="20" />版本历史</h2>
            <ul class="versions">
              <li v-for="v in detail.versions" :key="v.version" :class="{ yanked: v.yanked }">
                <span class="mono">{{ v.version }}</span>
                <span class="when">{{ formatTime(v.createdAt) }}</span>
                <span class="note">{{ v.yanked ? '该版本已撤回，请使用更新的版本。' : v.changelog || '—' }}</span>
                <span v-if="v.yanked" class="badge badge-bad">已撤回</span>
              </li>
            </ul>
          </section>

          <section>
            <h2><Icon name="list" :size="20" />更多</h2>
            <dl class="kv">
              <div><dt>标识</dt><dd class="mono">{{ item.namespace }} / {{ item.slug }}</dd></div>
              <div><dt>分类</dt><dd>{{ item.category?.name ?? '—' }}</dd></div>
              <div><dt>更新</dt><dd>{{ formatTime(item.updatedAt) }}</dd></div>
              <div><dt>许可证</dt><dd>{{ item.license ?? '未标注' }}</dd></div>
            </dl>
          </section>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.scrim { position: fixed; inset: 0; z-index: 50; background: rgba(20, 20, 20, 0.28); backdrop-filter: blur(8px); display: flex; align-items: center; justify-content: center; padding: 32px 16px; }
.dialog { outline: none; position: relative; width: 100%; max-width: 760px; max-height: min(880px, calc(100vh - 64px)); display: flex; flex-direction: column; background: #fff; border-radius: 28px; box-shadow: 0 2px 4px rgba(20, 20, 20, 0.06), 0 32px 80px -20px rgba(20, 20, 20, 0.35); overflow: hidden; }
.close { position: absolute; top: 18px; right: 18px; z-index: 2; width: 44px; height: 44px; border: 0; border-radius: 12px; background: transparent; display: flex; align-items: center; justify-content: center; cursor: pointer; }
.close:hover { background: var(--wash); }
.state { padding: 72px 24px; text-align: center; color: var(--ink-3); display: flex; gap: 10px; justify-content: center; align-items: center; }
.head { flex: 0 0 auto; display: flex; flex-wrap: wrap; align-items: center; gap: 20px 28px; padding: 40px 72px 24px 44px; border-bottom: 1px solid var(--line); }
.head-main { display: flex; flex-direction: column; align-items: flex-start; gap: 14px; min-width: 0; }
h1 { font-size: 26px; font-weight: 600; letter-spacing: -0.01em; overflow-wrap: anywhere; }
.actions { display: flex; flex-wrap: wrap; gap: 10px; }
.local-note { font-size: 13px; color: var(--ink-3); }
.body { flex: 1 1 auto; min-height: 0; overflow-y: auto; scrollbar-width: thin; padding: 24px 44px 44px; display: flex; flex-direction: column; gap: 4px; }
.lead { font-size: 16px; line-height: 1.75; color: #2e2e2e; margin-bottom: 12px; }
.tags { display: flex; flex-wrap: wrap; gap: 8px; }
.stats { margin: 28px 0 8px; display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); }
.stats > div { padding: 18px 8px; text-align: center; display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.stats b { font-size: 18px; font-weight: 500; display: inline-flex; align-items: center; justify-content: center; gap: 6px; flex-wrap: wrap; }
.stats b a { display: inline-flex; align-items: center; gap: 4px; }
.stats span { font-size: 13px; color: var(--ink-3); }
.verified { color: #2f6b2a; }
.mono-num { font-variant-numeric: tabular-nums; }
section { margin-top: 28px; }
h2 { display: flex; align-items: center; gap: 10px; font-size: 18px; font-weight: 600; margin-bottom: 14px; }
h2.split { justify-content: space-between; }
h2.split > span { display: inline-flex; align-items: center; gap: 10px; }
h2 small { font-size: 13px; font-weight: 400; color: var(--ink-3); }
.notice { line-height: 1.75; }
.kv { margin: 12px 0 0; padding: 6px 22px; border-radius: 16px; background: #f7f7f6; }
.kv > div { display: flex; flex-wrap: wrap; gap: 4px 16px; padding: 12px 0; font-size: 14px; }
.kv > div + div { border-top: 1px solid var(--line); }
.kv dt { flex: 0 0 72px; color: var(--ink-3); }
.kv dd { margin: 0; flex: 1; min-width: 0; overflow-wrap: anywhere; color: #2e2e2e; }
.kv dd.mono { font-size: 12.5px; }
.prompts { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 12px; }
.prompts li { display: flex; align-items: center; gap: 16px; padding: 18px 22px; border-radius: 16px; background: #f7f7f6; color: #2e2e2e; line-height: 1.7; }
.prompts li span { flex: 1; min-width: 0; }
.prompts li svg { flex: 0 0 22px; color: var(--ink-2); }
.code, .text { margin: 0; padding: 18px 22px; border-radius: 16px; background: #f7f7f6; color: #2e2e2e; font-size: 13px; line-height: 1.75; overflow-x: auto; }
.code { font-family: var(--mono); }
.text { font-family: var(--font); white-space: pre-wrap; overflow-wrap: anywhere; max-height: 360px; overflow-y: auto; }
.deps { margin-top: 12px; display: flex; flex-wrap: wrap; gap: 8px; align-items: center; font-size: 14px; color: var(--ink-2); }
.files, .versions { list-style: none; margin: 0; padding: 0; border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
.files li, .versions li { display: flex; flex-wrap: wrap; align-items: center; gap: 4px 16px; padding: 12px 22px; font-size: 14px; }
.files li + li, .versions li + li { border-top: 1px solid var(--line); }
.files .path { flex: 1 1 220px; min-width: 0; font-size: 13px; overflow-wrap: anywhere; }
.files .size { flex: 0 0 64px; text-align: right; color: var(--ink-3); font-variant-numeric: tabular-nums; }
.files .sha { flex: 0 0 96px; text-align: right; color: var(--ink-3); font-size: 12.5px; }
.versions li.yanked { color: var(--ink-3); }
.versions .when { flex: 0 0 72px; color: var(--ink-3); font-size: 13px; }
.versions .note { flex: 1 1 200px; min-width: 0; }
.skill-md { margin-top: 12px; }
.skill-md summary { cursor: pointer; min-height: 32px; color: var(--ink-2); font-size: 14px; }
@media (max-width: 640px) {
  .head { padding: 32px 64px 20px 20px; }
  .body { padding: 20px 20px 32px; }
  .dialog { border-radius: 20px; }
}
</style>
