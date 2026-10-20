<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { Check, ChevronRight, LoaderCircle, X } from 'lucide-vue-next';
import { clip, isNearBottom, prettyArguments, toolLabel } from './format.ts';
import { describeCall } from './view.ts';
import type { ToolCall } from './replies.ts';

const props = defineProps<{ calls: ToolCall[]; active: boolean; waiting?: boolean; status?: string }>();
const expanded = ref(false);
const history = ref<HTMLElement | null>(null);
const follow = ref(true);
const latest = computed(() => [...props.calls].reverse().find(call => call.status === 'running') ?? props.calls.at(-1));
const errors = computed(() => props.calls.filter(call => call.status === 'error').length);
const label = computed(() => !props.active ? '执行过程' : props.waiting ? '等待确认' : props.calls.some(call => call.status === 'running') ? '执行中' : props.status || '执行过程');
const changes = computed(() => [expanded.value, ...props.calls.map(call => `${call.id}:${call.status}:${call.output.length}`)].join('|'));

async function followLatest() {
  await nextTick();
  const el = history.value;
  // Keep a reader's position while inspecting an expanded command/result.
  if (el && follow.value && !el.querySelector('details[open]')) el.scrollTop = el.scrollHeight;
}
watch(changes, followLatest);
function toggle() { expanded.value = !expanded.value; follow.value = true; }
function onScroll() { if (history.value) follow.value = isNearBottom(history.value); }
</script>

<template>
  <section class="activity" :class="{ expanded, active }" data-testid="chat-activity" aria-label="本轮执行过程">
    <button type="button" class="activity-toggle" data-testid="activity-toggle" :aria-expanded="expanded" @click="toggle">
      <ChevronRight :size="13" class="chevron" :class="{ rotated: expanded }" />
      <span class="activity-label">{{ label }}</span>
      <span class="activity-count">{{ calls.length }} 步</span>
      <span v-if="errors" class="activity-errors">{{ errors }} 次异常</span>
    </button>

    <div v-if="active && !expanded && latest" class="activity-live" role="status" aria-live="polite" aria-atomic="true" data-testid="activity-live">
      <Transition name="activity-step" mode="out-in">
        <div :key="latest.id" class="step" :class="latest.status" :data-testid="`tool-${latest.id}`">
          <LoaderCircle v-if="latest.status === 'running'" :size="13" class="spin" /><Check v-else-if="latest.status === 'done'" :size="13" /><X v-else :size="13" />
          <span class="step-name">{{ toolLabel(latest.name) }}</span>
          <span class="step-description" :title="describeCall(latest.name, latest.arguments)">{{ describeCall(latest.name, latest.arguments) }}</span>
        </div>
      </Transition>
    </div>

    <div v-if="expanded" ref="history" class="activity-history" data-testid="activity-history" role="region" aria-label="完整执行记录" tabindex="0" @scroll.passive="onScroll">
      <details v-for="call in calls" :key="call.id" class="tool" :class="call.status" :data-testid="`tool-${call.id}`">
        <summary>
          <LoaderCircle v-if="call.status === 'running'" :size="13" class="spin" /><Check v-else-if="call.status === 'done'" :size="13" /><X v-else :size="13" />
          <b>{{ toolLabel(call.name) }}</b><span class="step-description">{{ describeCall(call.name, call.arguments) }}</span>
          <ChevronRight :size="12" class="call-chevron" />
        </summary>
        <div class="tool-body">
          <div v-if="call.arguments" class="tool-part"><small>内容</small><pre>{{ prettyArguments(call.arguments) }}</pre></div>
          <div v-if="call.output" class="tool-part"><small>结果</small><pre>{{ clip(call.output) }}</pre></div>
        </div>
      </details>
    </div>
  </section>
</template>

<style scoped>
.activity{margin-left:40px;min-width:0;max-width:100%;color:var(--ui-text-tertiary);font-size:12px}
.activity-toggle{display:flex;align-items:center;gap:7px;max-width:100%;padding:4px 0;border:0;background:transparent;color:inherit;font:inherit;cursor:pointer;text-align:left;border-radius:5px}
.activity-toggle:hover{color:var(--ui-text-secondary)}
.activity-toggle:focus-visible,.activity-history:focus-visible{outline:2px solid var(--ui-primary);outline-offset:3px}
.chevron{flex:none;transition:transform .15s ease}.chevron.rotated{transform:rotate(90deg)}
.activity-label{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.activity-count{white-space:nowrap;font-variant-numeric:tabular-nums;opacity:.75}
.activity-errors{color:var(--ui-primary-pressed);white-space:nowrap}
.activity-live{height:28px;overflow:hidden;margin-left:20px}
.step{height:28px;display:flex;align-items:center;gap:8px;min-width:0;color:var(--ui-text-secondary)}
.step svg,.tool summary>svg{flex:none;color:var(--ui-text-tertiary)}
.step.done>svg,.tool.done summary>svg:first-child{color:#1fb871}
.step.error>svg,.tool.error summary>svg:first-child{color:var(--ui-primary-pressed)}
.step-name{flex:none}.step-description{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--ui-text-tertiary);font:11.5px ui-monospace,Menlo,Consolas,monospace}
.activity-history{max-height:260px;overflow:auto;overscroll-behavior:contain;scrollbar-width:thin;overflow-anchor:none;margin:6px 0 2px 6px;border-left:1px solid var(--ui-line);padding-left:12px}
.tool{font-size:12px;min-width:0}
.tool summary{display:flex;align-items:center;gap:8px;min-width:0;padding:7px 2px;list-style:none;cursor:pointer;color:var(--ui-text-secondary)}
.tool summary::-webkit-details-marker{display:none}.tool summary:hover{color:var(--ui-text-primary)}
.tool summary b{font-weight:500;white-space:nowrap;flex:none}.call-chevron{margin-left:auto}.tool[open] .call-chevron{transform:rotate(90deg)}
.tool-body{display:grid;gap:8px;padding:3px 3px 10px 21px}.tool-part small{color:var(--ui-text-tertiary)}
.tool-part pre{margin:4px 0 0;padding:9px 11px;background:var(--ui-surface-alt);border-radius:8px;overflow:auto;max-height:180px;font-size:12px;line-height:1.6;white-space:pre-wrap;overflow-wrap:anywhere;word-break:break-word;font-family:ui-monospace,Menlo,Consolas,monospace}
.spin{animation:activity-spin 1s linear infinite}@keyframes activity-spin{to{transform:rotate(360deg)}}
.activity-step-enter-active,.activity-step-leave-active{transition:opacity .12s ease,transform .12s ease}
.activity-step-enter-from{opacity:0;transform:translateY(8px)}.activity-step-leave-to{opacity:0;transform:translateY(-8px)}
@media(prefers-reduced-motion:reduce){.spin{animation:none}.chevron,.activity-step-enter-active,.activity-step-leave-active{transition:none}}
@media(max-width:900px){.activity{margin-left:0}}
</style>
