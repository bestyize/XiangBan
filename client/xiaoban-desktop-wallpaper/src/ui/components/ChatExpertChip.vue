<script setup lang="ts">
/** 对话绑定的专家：头像、名字和职业。新对话开始前可以点 × 取消；已经开始的对话只显示。 */
import { onMounted, ref, watch } from 'vue';
import { X } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ExpertView } from '../chat/types.ts';

const props = defineProps<{ expert: ExpertView; removable?: boolean; desktop?: boolean; /** 输入框右上角用：只显示头像和名字，职业放进提示。 */ compact?: boolean }>();
const emit = defineEmits<{ clear: [] }>();

/** 头像读一次就记住；没有头像或读不出来就用名字的第一个字。 */
const avatars = new Map<string, string | null>();
const avatar = ref<string | null>(null);
async function load() {
  const { plugin, hasAvatar } = props.expert;
  if (!hasAvatar || !props.desktop) { avatar.value = null; return; }
  if (!avatars.has(plugin)) {
    try { avatars.set(plugin, await invoke<string | null>('ai_plugin_avatar', { name: plugin })); } catch { avatars.set(plugin, null); }
  }
  avatar.value = avatars.get(plugin) ?? null;
}
onMounted(load);
watch(() => props.expert.plugin, load);
</script>

<template>
  <div class="expert-chip" :class="{ compact }" data-testid="chat-expert" :title="expert.profession ? `${expert.name} · ${expert.profession}` : expert.name">
    <span class="expert-avatar" :class="{ team: expert.kind === 'team' }" aria-hidden="true"><img v-if="avatar" :src="avatar" alt="" /><template v-else>{{ expert.name.slice(0, 1) }}</template></span>
    <span class="expert-name">{{ expert.name }}</span>
    <span v-if="expert.profession && !compact" class="expert-profession">{{ expert.profession }}</span>
    <button v-if="removable" type="button" class="expert-clear" data-testid="chat-expert-clear" aria-label="取消召唤这位专家" @click="emit('clear')"><X :size="12" /></button>
  </div>
</template>

<style scoped>
.expert-chip{display:inline-flex;align-items:center;gap:7px;max-width:100%;min-width:0;height:28px;padding:0 6px 0 4px;border-radius:999px;background:var(--ui-soft);color:var(--ui-ink);font-size:12.5px}
.expert-avatar{flex:none;display:grid;place-items:center;width:20px;height:20px;border-radius:50%;overflow:hidden;background:var(--ui-surface);font-size:11px;font-weight:600}
.expert-avatar img{width:100%;height:100%;object-fit:cover;display:block}
.expert-avatar.team{border-radius:6px}
.expert-chip.compact{background:transparent;box-shadow:inset 0 0 0 1px var(--ui-border-strong);color:var(--ui-text-primary);max-width:200px}
.expert-chip.compact .expert-avatar{background:var(--ui-soft);color:var(--ui-ink)}
.expert-name{font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.expert-profession{color:var(--ui-text-secondary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.expert-clear{flex:none;display:grid;place-items:center;width:18px;height:18px;margin-left:2px;border:0;border-radius:50%;background:transparent;color:var(--ui-text-secondary);cursor:pointer}
.expert-clear:hover{background:var(--ui-control-hover);color:var(--ui-text-primary)}
</style>
