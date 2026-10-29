<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { NButton, NModal, NSpin } from 'naive-ui';
import { invoke, listen } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import { applyFlow, closeApply, confirmApply, KIND_LABEL, scheduleText, schemeFlow, schemeProgress, startApply, stopScheme, TRUST_LABEL } from './apply.ts';

const props = defineProps<{ store: ControlStore }>();

interface CommunityLink { action: 'apply' | 'open'; packId?: number; version?: number | null; sha256?: string | null; target?: string; id?: string }

const state = computed(() => applyFlow.state);
const show = computed(() => state.value.phase !== 'idle');
const progress = computed(() => schemeProgress());
const preview = computed(() => ('preview' in state.value ? state.value.preview : null));
const taskLike = computed(() => preview.value?.kind === 'routine');

/** A link that opened the app: apply links go straight into the check; open links just show the community page. */
async function takeLink() {
  if (!props.store.desktop) return;
  try {
    const link = await invoke<CommunityLink | null>('community_take_link');
    if (!link) return;
    if (link.action === 'apply' && link.packId) void startApply({ packId: link.packId, version: link.version, sha256: link.sha256 });
    else props.store.navigate('community');
  } catch { /* 没有待处理的链接 */ }
}

async function apply() {
  const message = await confirmApply({
    // 任务模板：放进新对话的输入框，不自动发送。
    async task(prompt) { await props.store.chat.select(null); props.store.creationDraft = prompt; props.store.navigate('chat'); },
    // 目标模板：打开长期目标表单并填好，由用户自己保存。
    async goal(goal) { await props.store.chat.select(null); props.store.goalTemplate = goal; props.store.navigate('chat'); },
  });
  if (message) props.store.notify(message);
}

let unlisten: (() => void) | undefined;
onMounted(() => {
  void takeLink();
  void listen('community-link', () => void takeLink()).then(stop => { unlisten = stop; });
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <NModal :show="show" preset="card" :title="preview ? `${KIND_LABEL[preview.kind] ?? preview.kind}：${preview.title}` : '社区内容'" style="width: min(560px, 92vw)" :mask-closable="state.phase !== 'applying'" @update:show="(v: boolean) => { if (!v) closeApply(); }">
<p v-if="progress" class="community-meta">{{ progress }}<template v-if="schemeFlow.skipped">，另有 {{ schemeFlow.skipped }} 项桌面端暂不支持，已跳过</template></p>
    <div v-if="state.phase === 'loading'" class="community-state"><NSpin size="small" /> 正在下载并校验…</div>

    <template v-else-if="state.phase === 'confirm' || state.phase === 'applying'">
      <p class="community-meta">
        <span>{{ TRUST_LABEL[state.preview.trustLevel] ?? '未审核' }}</span>
        <span>版本 {{ state.preview.version }}</span>
        <span>{{ (state.preview.size / 1024).toFixed(1) }} KB</span>
        <span title="下载内容已通过 SHA-256 校验">已校验</span>
      </p>
      <p v-if="taskLike" class="community-warn">这是会按时间自动运行的例行事项（只读取、汇总、提醒，不写文件、不运行命令）。下面是它的完整说明，请先读完；添加后默认不启用，要到「提醒」页确认后再打开。</p>
      <p v-else-if="!state.preview.canApply" class="community-warn">这个内容还没有通过审核，暂时不能一键应用。</p>
      <ul class="community-items">
        <li v-for="(item, i) in state.preview.items" :key="i">
          <strong>{{ item.title }}</strong>
          <span v-if="item.schedule" class="community-when">{{ scheduleText(item.schedule) }}</span>
          <p v-if="item.detail">{{ item.detail }}</p>
        </li>
      </ul>
    </template>

    <p v-else-if="state.phase === 'done'" class="community-state">{{ state.message }}</p>
    <p v-else-if="state.phase === 'error'" class="community-error">{{ state.message }}</p>

    <template #footer>
      <div class="community-actions">
        <NButton v-if="state.phase === 'confirm' || state.phase === 'applying'" type="primary" :disabled="!state.preview.canApply" :loading="state.phase === 'applying'" @click="apply">{{ taskLike ? '添加（默认不启用）' : preview?.kind === 'task' ? '放进对话' : preview?.kind === 'goal' ? '填入目标表单' : '应用' }}</NButton>
        <NButton v-if="progress" :disabled="state.phase === 'applying'" @click="stopScheme">停止复刻</NButton>
        <NButton :disabled="state.phase === 'applying'" @click="closeApply">{{ progress ? (state.phase === 'done' ? '下一项' : '跳过这项') : state.phase === 'done' || state.phase === 'error' ? '关闭' : '取消' }}</NButton>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.community-state, .community-error { margin: 0; color: var(--xb-text-2, #666); display: flex; align-items: center; gap: 8px; }
.community-error { color: var(--xb-danger, #d03050); }
.community-meta { margin: 0 0 8px; display: flex; flex-wrap: wrap; gap: 4px 12px; color: var(--xb-text-3, #888); font-size: 12px; }
.community-warn { margin: 0 0 8px; padding: 8px 10px; border-radius: 6px; background: var(--xb-surface-2, #f5f5f5); font-size: 13px; }
.community-items { list-style: none; margin: 0; padding: 0; max-height: 280px; overflow: auto; }
.community-items li { padding: 8px 0; border-bottom: 1px solid var(--xb-line, #eee); }
.community-items p { margin: 4px 0 0; white-space: pre-wrap; font-size: 13px; }
.community-when { margin-left: 8px; color: var(--xb-text-3, #888); font-size: 12px; }
.community-actions { display: flex; justify-content: flex-end; gap: 8px; }
</style>
