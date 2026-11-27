<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '../bridge.ts';
import CompanionReplyCard from '../companion-dialogue/CompanionReplyCard.vue';
import type { CompanionReply } from '../companion-dialogue/dialogue.ts';
import { accents, DEFAULT_ACCENT, type AccentId, type AppearanceMode } from '../ui/appearance.ts';
import { applyUiTokens } from '../ui/design-tokens.ts';

interface Greeting {
  id: number;
  petName: string;
  text: string;
  appearance?: AppearanceMode;
  accent?: AccentId;
  tail?: 'left' | 'right';
  dialogue?: CompanionReply;
}

const greeting = ref<Greeting | null>(null);
const error = ref('');
const card = ref<HTMLElement | null>(null);
let timeout: ReturnType<typeof setTimeout> | undefined;
let revision = 0;
let hovered = false;
let focused = false;
let mediaQuery: MediaQueryList | null = null;

function resolveAccent(value?: string): AccentId {
  return accents.some(item => item.id === value) ? (value as AccentId) : DEFAULT_ACCENT;
}

function applyTheme(appearance: AppearanceMode = 'system', accent?: string) {
  const prefersDark = !!mediaQuery?.matches;
  const dark = appearance === 'dark' || (appearance === 'system' && prefersDark);
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
  applyUiTokens(dark, resolveAccent(accent));
}

function onSchemeChange() {
  if (greeting.value) applyTheme(greeting.value.appearance ?? 'system', greeting.value.accent);
}

function arm() {
  clearTimeout(timeout);
  timeout = undefined;
  if (!greeting.value || greeting.value.dialogue || hovered || focused) return;
  timeout = setTimeout(() => void finish('ignored'), 8000);
}

function display(value: Greeting | null) {
  ++revision;
  clearTimeout(timeout);
  timeout = undefined;
  hovered = false;
  focused = false;
  error.value = '';
  greeting.value = value;
  if (value) {
    applyTheme(value.appearance ?? 'system', value.accent);
    arm();
  }
}

function changed(event: Event) {
  display((event as CustomEvent<Greeting | null>).detail);
}

function pointerIn() { hovered = true; clearTimeout(timeout); timeout = undefined; }
function pointerOut() { hovered = false; arm(); }
function focusIn() { focused = true; clearTimeout(timeout); timeout = undefined; }
function focusOut() {
  queueMicrotask(() => {
    focused = !!card.value?.contains(document.activeElement);
    arm();
  });
}

async function finish(outcome: 'chat' | 'poke' | 'dismissed' | 'ignored' | 'snoozed') {
  const active = greeting.value;
  if (!active) return;
  display(null);
  const submitted = revision;
  try { await invoke('respond_companion_greeting', { id: active.id, outcome }); }
  catch (e) { if (submitted === revision) error.value = String(e); }
}

onMounted(async () => {
  document.documentElement.style.background = 'transparent';
  document.body.className = 'pet-greeting-scene';
  mediaQuery = typeof window.matchMedia === 'function' ? window.matchMedia('(prefers-color-scheme: dark)') : null;
  mediaQuery?.addEventListener?.('change', onSchemeChange);
  applyTheme('system', DEFAULT_ACCENT);
  window.addEventListener('companion-greeting-changed', changed);
  const initial = revision;
  try {
    const first = await invoke<Greeting>('get_companion_greeting');
    if (initial === revision) display(first);
  } catch (e) {
    if (initial === revision) error.value = String(e);
  }
});

onBeforeUnmount(() => {
  ++revision;
  clearTimeout(timeout);
  window.removeEventListener('companion-greeting-changed', changed);
  mediaQuery?.removeEventListener?.('change', onSchemeChange);
});
</script>

<template>
  <div class="greeting-shell" :data-appearance="greeting?.appearance || 'system'">
    <CompanionReplyCard
      v-if="greeting?.dialogue"
      :key="greeting.id"
      class="greeting-dialogue"
      :initial="greeting.dialogue"
      :tail="greeting.tail || 'left'"
      @close="display(null)"
    />
    <section
      v-else-if="greeting"
      ref="card"
      class="greeting-card"
      :data-tail="greeting.tail || 'left'"
      role="status"
      aria-live="polite"
      @pointerenter="pointerIn"
      @pointerleave="pointerOut"
      @focusin="focusIn"
      @focusout="focusOut"
      @keydown.esc.stop="finish('dismissed')"
    >
      <header class="greeting-head">
        <span class="greeting-dot" aria-hidden="true"></span>
        <span class="greeting-name">{{ greeting.petName }}</span>
        <span class="greeting-badge">轻声问候</span>
        <button type="button" class="greeting-close" title="关闭问候" aria-label="关闭问候" @click="finish('dismissed')">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" aria-hidden="true">
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </header>
      <p class="greeting-text">{{ greeting.text }}</p>
      <footer class="greeting-foot">
        <button type="button" class="greeting-action is-primary" @click="finish('chat')">聊两句</button>
        <button type="button" class="greeting-action" @click="finish('poke')">摸摸头</button>
        <button type="button" class="greeting-later" title="短时间内不再主动弹出问候" @click="finish('snoozed')">暂不打扰</button>
      </footer>
    </section>
    <span v-else-if="error" class="greeting-error">问候已结束</span>
  </div>
</template>

<style>
html:has(body.pet-greeting-scene),
body.pet-greeting-scene,
body.pet-greeting-scene #app {
  width: 100%;
  height: 100%;
  margin: 0;
  background: transparent !important;
  overflow: hidden;
  -webkit-font-smoothing: antialiased;
}
.greeting-shell {
  box-sizing: border-box;
  width: 100%;
  height: 100%;
  padding: 13px 14px 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--ui-font-family, -apple-system, 'SF Pro Text', 'PingFang SC', 'Microsoft YaHei UI', sans-serif);
}
.greeting-card {
  position: relative;
  box-sizing: border-box;
  width: 100%;
  padding: 14px 15px 13px;
  border-radius: 22px;
  background: #FFFDFB;
  box-shadow:
    0 0 0 1px rgba(140, 108, 86, .13),
    0 14px 30px rgba(72, 48, 34, .12),
    0 3px 8px rgba(72, 48, 34, .05);
  color: var(--ui-text-primary, #1D1D1F);
  display: flex;
  flex-direction: column;
  gap: 9px;
  animation: greeting-bubble-in .2s cubic-bezier(.22, 1, .36, 1);
}
.greeting-card[data-tail]::after {
  content: "";
  position: absolute;
  width: 10px;
  height: 10px;
  background: inherit;
  transform: rotate(45deg);
  pointer-events: none;
}
.greeting-card[data-tail="left"]::after {
  left: -5px;
  top: 28px;
  border-radius: 0 0 0 2.5px;
  box-shadow: -1px 1px 0 rgba(140, 108, 86, .13);
}
.greeting-card[data-tail="right"]::after {
  right: -5px;
  top: 28px;
  border-radius: 0 2.5px 0 0;
  box-shadow: 1px -1px 0 rgba(140, 108, 86, .13);
}
.greeting-head {
  display: flex;
  align-items: center;
  gap: 6px;
  min-height: 20px;
}
.greeting-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex: none;
  background: var(--ui-dot, #E5484D);
  box-shadow: 0 0 0 3px var(--ui-soft, #FCEBEC);
}
.greeting-name {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--ui-text-secondary, #68686E);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.greeting-badge {
  flex: none;
  font-size: 10.5px;
  color: var(--ui-text-tertiary, #8E8E95);
  padding: 1px 7px;
  border-radius: 999px;
  background: rgba(164, 132, 110, .10);
  white-space: nowrap;
}
.greeting-close {
  margin-left: auto;
  margin-right: -4px;
  margin-top: -2px;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--ui-text-tertiary, #8E8E95);
  display: grid;
  place-items: center;
  cursor: pointer;
  flex: none;
  transition: background .15s ease, color .15s ease;
}
.greeting-close:hover {
  background: rgba(164, 132, 110, .12);
  color: var(--ui-text-primary, #1D1D1F);
}
.greeting-text {
  margin: 0;
  font-size: 13px;
  line-height: 1.56;
  min-height: 20px;
  color: var(--ui-text-primary, #1D1D1F);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  overflow-wrap: anywhere;
}
.greeting-foot {
  display: flex;
  gap: 6px;
  align-items: center;
  padding-top: 2px;
}
.greeting-action {
  height: 26px;
  padding: 0 11px;
  border: 0;
  border-radius: 999px;
  background: rgba(164, 132, 110, .10);
  color: var(--ui-text-secondary, #68686E);
  font: inherit;
  font-size: 11.5px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: background .15s ease, color .15s ease;
}
.greeting-action:hover {
  background: rgba(164, 132, 110, .17);
  color: var(--ui-text-primary, #1D1D1F);
}
.greeting-action.is-primary {
  background: var(--ui-soft, #FCEBEC);
  color: var(--ui-ink, #B42A33);
  font-weight: 600;
}
.greeting-action.is-primary:hover {
  background: var(--ui-btn, #D93A44);
  color: #fff;
}
.greeting-later {
  margin-left: auto;
  height: 26px;
  padding: 0 4px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--ui-text-tertiary, #8E8E95);
  font: inherit;
  font-size: 11.5px;
  cursor: pointer;
  white-space: nowrap;
  transition: color .15s ease;
}
.greeting-later:hover {
  color: var(--ui-text-primary, #1D1D1F);
}
.greeting-card button:focus-visible {
  outline: 2px solid var(--ui-btn, #D93A44);
  outline-offset: 2px;
}
.greeting-error {
  font-size: 11px;
  color: var(--ui-text-tertiary, #8E8E95);
}
.greeting-dialogue {
  width: 100%;
  max-height: 100%;
}

html[data-theme="dark"] .greeting-card {
  background: #292624;
  box-shadow:
    inset 0 1px 0 rgba(255, 236, 220, .08),
    0 0 0 1px rgba(255, 224, 198, .11),
    0 18px 40px rgba(0, 0, 0, .54);
}
html[data-theme="dark"] .greeting-card[data-tail="left"]::after {
  box-shadow: -1px 1px 0 rgba(255, 224, 198, .11);
}
html[data-theme="dark"] .greeting-card[data-tail="right"]::after {
  box-shadow: 1px -1px 0 rgba(255, 224, 198, .11);
}
html[data-theme="dark"] .greeting-badge,
html[data-theme="dark"] .greeting-action {
  background: rgba(255, 224, 198, .08);
}
html[data-theme="dark"] .greeting-action:hover,
html[data-theme="dark"] .greeting-close:hover {
  background: rgba(255, 224, 198, .14);
}
html[data-theme="dark"] .greeting-action.is-primary {
  background: var(--ui-soft, #3A2225);
  color: var(--ui-ink, #FF8C91);
}
html[data-theme="dark"] .greeting-action.is-primary:hover {
  background: var(--ui-btn, #E5484D);
  color: #fff;
}

@keyframes greeting-bubble-in {
  from { opacity: 0; transform: translateY(5px) scale(.97); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}
@media (prefers-reduced-motion: reduce) {
  .greeting-card,
  .greeting-card button { animation: none; transition: none; }
}
</style>
