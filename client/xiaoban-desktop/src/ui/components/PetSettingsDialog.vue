<script setup lang="ts">
/**
 * 伙伴管理: focused configuration view for one desktop pet instance.
 * Left: frameless pure-white preview stage with inline rename & quick actions.
 * Right: two-tab workbench (常规 / 心情) with live hover preview for all 25 expressions.
 */
import { computed, reactive, ref, watch } from 'vue';
import { NModal, NPopconfirm, NSwitch } from 'naive-ui';
import { Check, ChevronRight, Crosshair, Monitor, Pencil, Pin, Sparkles, X } from 'lucide-vue-next';
import PetActor from '../../pets/PetActor.vue';
import { EXPRESSIONS, EXPRESSION_NAMES, type ExpressionName } from '../../companion/expressions.ts';
import type { PetInstance, PetTheme } from '../../pets/model.ts';
import WindowChrome from '../configuration/WindowChrome.vue';
import { dragConfigurationWindow, zoomConfigurationWindow } from '../configuration/windows.ts';

type Theme = PetTheme & { source: 'builtin' | 'custom' };
export interface PetSettingsPatch extends Partial<Pick<PetInstance, 'name' | 'size' | 'expression' | 'layer' | 'links' | 'enabled'>> {}

const props = defineProps<{
  show: boolean;
  detached?: boolean;
  platform?: string;
  instance?: PetInstance;
  themes: Theme[];
  busy?: boolean;
  paused?: boolean;
}>();
const emit = defineEmits<{
  'update:show': [boolean];
  save: [PetSettingsPatch];
  remove: [];
  center: [];
  create: [];
  close: [];
}>();

const MOOD_GROUPS: { name: string; items: ExpressionName[] }[] = [
  { name: '日常', items: ['idle', 'happy', 'excited', 'laughing', 'proud', 'curious', 'thinking', 'shy', 'love'] },
  { name: '情绪', items: ['confused', 'surprised', 'sad', 'worried', 'angry', 'annoyed', 'crying', 'embarrassed'] },
  { name: '状态', items: ['sleepy', 'sleeping', 'listening', 'speaking', 'greeting', 'waiting', 'offline', 'charging'] },
];
const groups = computed(() => {
  const known = new Set(MOOD_GROUPS.flatMap(group => group.items));
  const extra = EXPRESSION_NAMES.filter(name => !known.has(name));
  return MOOD_GROUPS.map(group => ({
    name: group.name,
    items: (group.name === '状态' ? [...group.items, ...extra] : group.items).filter(name => name in EXPRESSIONS),
  }));
});
const groupOf = (name: ExpressionName) => groups.value.find(group => group.items.includes(name))?.name ?? '日常';

const layers = [
  { value: 'topmost' as const, label: '始终置顶', hint: '浮在所有窗口上方，随时能戳到', icon: Pin },
  { value: 'desktop' as const, label: '贴在桌面', hint: '贴在桌面图标下方，不遮挡其他窗口', icon: Monitor },
];
const sizePresets = [
  { label: '小', value: 140 },
  { label: '标准', value: 210 },
  { label: '大', value: 300 },
];

const draft = reactive({
  name: '',
  size: 210,
  expression: 'idle' as ExpressionName,
  layer: 'topmost' as PetInstance['layer'],
  links: true,
  enabled: true,
});

const tab = ref<'general' | 'mood'>('general');
const moodFilter = ref<'all' | '日常' | '情绪' | '状态'>('all');
const hoverMood = ref<ExpressionName | null>(null);
const nameInputRef = ref<HTMLInputElement | null>(null);

function reset() {
  const item = props.instance;
  if (!item) return;
  Object.assign(draft, {
    name: item.name,
    size: item.size,
    expression: item.expression,
    layer: item.layer,
    links: item.links,
    enabled: item.enabled,
  });
  tab.value = 'general';
  moodFilter.value = 'all';
  hoverMood.value = null;
}
watch(() => [props.show, props.instance?.id, props.detached], () => { if (props.show || props.detached) reset(); }, { immediate: true });

const theme = computed(() => props.themes.find(item => item.id === props.instance?.themeId));
const definition = computed(() => theme.value?.pets.find(pet => pet.id === props.instance?.petId));
const visibleGroups = computed(() => moodFilter.value === 'all' ? groups.value : groups.value.filter(g => g.name === moodFilter.value));
const activeExpression = computed<ExpressionName>(() => hoverMood.value ?? draft.expression);
const previewSize = computed(() => Math.round(136 + (Math.min(420, Math.max(80, draft.size)) - 80) / 340 * 68));
const sizeProgress = computed(() => `${((Math.min(420, Math.max(80, draft.size)) - 80) / 340) * 100}%`);
const nameInvalid = computed(() => !draft.name.trim());
const metaLine = computed(() => [theme.value?.name, definition.value?.name ?? '作品暂不可用', props.instance?.monitor].filter(Boolean).join(' · '));

function finishRename() {
  if (!draft.name.trim() && props.instance) draft.name = props.instance.name;
  nameInputRef.value?.blur();
}

function pickMood(name: ExpressionName) {
  draft.expression = name;
  hoverMood.value = null;
}
function clearHover(name: ExpressionName) {
  if (hoverMood.value === name) hoverMood.value = null;
}

function pickRandomMood() {
  const pool = EXPRESSION_NAMES.filter(n => n !== draft.expression);
  const chosen = pool[Math.floor(Math.random() * pool.length)] ?? draft.expression;
  pickMood(chosen);
}

function onMoodKeydown(event: KeyboardEvent, current: ExpressionName) {
  const flat = visibleGroups.value.flatMap(g => g.items);
  const idx = flat.indexOf(current);
  if (idx < 0) return;
  let nextIdx = -1;
  if (event.key === 'ArrowRight') nextIdx = (idx + 1) % flat.length;
  else if (event.key === 'ArrowLeft') nextIdx = (idx - 1 + flat.length) % flat.length;
  else if (event.key === 'ArrowDown') nextIdx = Math.min(flat.length - 1, idx + 4);
  else if (event.key === 'ArrowUp') nextIdx = Math.max(0, idx - 4);
  else if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault();
    pickMood(current);
    return;
  } else return;
  event.preventDefault();
  const targetName = flat[nextIdx];
  hoverMood.value = targetName;
  const root = (event.currentTarget as HTMLElement | null)?.closest('.mood-board-wrap');
  root?.querySelector<HTMLButtonElement>(`[data-testid="pet-mood-${targetName}"]`)?.focus();
}

function done() {
  const item = props.instance;
  if (!item) { close(); return; }
  const patch: PetSettingsPatch = {};
  const name = draft.name.trim();
  if (name && name !== item.name) patch.name = name;
  if (draft.size !== item.size) patch.size = draft.size;
  if (draft.expression !== item.expression) patch.expression = draft.expression;
  if (draft.layer !== item.layer) patch.layer = draft.layer;
  if (draft.links !== item.links) patch.links = draft.links;
  if (draft.enabled !== item.enabled) patch.enabled = draft.enabled;
  if (Object.keys(patch).length) emit('save', patch);
  close();
}

function close() {
  hoverMood.value = null;
  emit('update:show', false);
  emit('close');
}
</script>

<template>
  <component
    :is="detached ? 'div' : NModal"
    :show="detached ? undefined : (show && !!instance)"
    :auto-focus="detached ? undefined : false"
    :transform-origin="detached ? undefined : 'center'"
    :mask-closable="detached ? undefined : true"
    :class="detached ? 'pet-detached-host' : undefined"
    @update:show="(value: boolean) => { if (!detached) emit('update:show', value); }"
  >
    <section
      v-if="instance"
      class="pet-dialog"
      :class="detached ? 'is-detached' : 'is-modal'"
      :role="detached ? 'region' : 'dialog'"
      :aria-modal="detached ? undefined : true"
      aria-label="伙伴设置"
      data-testid="pet-settings-dialog"
    >
      <!-- 左栏：纯净无框白展台 + 居中大预览 + 内联改名 -->
      <aside class="pd-left">
        <WindowChrome v-if="detached" :platform="platform" />
        <div class="pd-hero">
          <div class="pd-stage" :class="{ 'is-off': !draft.enabled }">
            <span class="pd-preview-tag" :class="{ 'is-visible': !!hoverMood }">
              {{ hoverMood ? `试看 · ${EXPRESSIONS[hoverMood].label}` : '' }}
            </span>
            <PetActor
              v-if="definition"
              :definition="definition"
              :size="previewSize"
              :expression="activeExpression"
              :links="draft.links"
              :paused="paused"
            />
          </div>

          <div class="pd-name-wrap">
            <label class="pd-name-field" title="点击修改名字">
              <input
                ref="nameInputRef"
                v-model="draft.name"
                type="text"
                maxlength="40"
                placeholder="输入伙伴昵称"
                aria-label="伙伴名字"
                class="pd-name-input"
                :class="{ 'is-err': nameInvalid }"
                data-testid="pet-dialog-name"
                @keydown.enter.prevent="finishRename"
                @keydown.esc.stop.prevent="finishRename"
              />
              <Pencil :size="12" class="pd-name-pencil" aria-hidden="true" />
            </label>
            <span v-if="nameInvalid" class="cfg-err">请输入昵称</span>
          </div>
          <span class="pd-meta" :title="metaLine">{{ metaLine }}</span>
          <span class="pd-hint">戳一戳，会回应你</span>
        </div>

        <div class="pd-left-acts">
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-center" @click="emit('center')">
            <Crosshair :size="13" />移到屏幕中间
          </button>
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-create" @click="emit('create')">
            <Sparkles :size="13" />基于此创作新伙伴
          </button>
        </div>
      </aside>

      <!-- 右栏：轻灰画布白卡 · 常规 / 心情 双段直达 -->
      <div class="pd-right">
        <header
          class="pd-head"
          @mousedown="detached && dragConfigurationWindow($event)"
          @dblclick="detached && zoomConfigurationWindow($event)"
        >
          <div class="pd-nav-seg" role="tablist" aria-label="伙伴管理分区">
            <button type="button" role="tab" :aria-selected="tab === 'general'" data-testid="pet-tab-general" @click="tab = 'general'; hoverMood = null">常规</button>
            <button type="button" role="tab" :aria-selected="tab === 'mood'" data-testid="pet-tab-mood" @click="tab = 'mood'">心情</button>
          </div>
          <span class="pd-status" :class="{ off: !draft.enabled }"><i />{{ draft.enabled ? '在桌面上' : '已隐藏' }}</span>
          <button v-if="!detached" type="button" class="xb-icon-btn pd-close" aria-label="关闭" data-testid="pet-dialog-close" @click="close">
            <X :size="16" />
          </button>
        </header>

        <div class="pd-scroll">
          <!-- 常规分区 -->
          <template v-if="tab === 'general'">
            <div class="cfg-card">
              <div class="cfg-row" title="控制这只伙伴当前是否在桌面上显示">
                <span class="cfg-lbl grow">在桌面上显示</span>
                <NSwitch v-model:value="draft.enabled" size="small" data-testid="pet-dialog-enabled" />
              </div>

              <div class="cfg-row col">
                <div class="size-head">
                  <span class="cfg-lbl">大小</span>
                  <div class="size-presets" role="group" aria-label="大小预设">
                    <button v-for="preset in sizePresets" :key="preset.value" type="button" :class="{ active: draft.size === preset.value }" @click="draft.size = preset.value">{{ preset.label }}</button>
                  </div>
                  <b class="size-val" data-testid="pet-dialog-size">{{ draft.size }} px</b>
                </div>
                <input v-model.number="draft.size" type="range" min="80" max="420" step="10" class="pd-range" :style="{ '--range-pct': sizeProgress }" aria-label="大小" />
              </div>
            </div>

            <div class="cfg-card">
              <div class="cfg-row col">
                <span class="cfg-lbl">显示层级</span>
                <div class="layer-cards" role="radiogroup" aria-label="显示层级">
                  <button
                    v-for="option in layers"
                    :key="option.value"
                    type="button"
                    role="radio"
                    class="layer-card"
                    :aria-checked="draft.layer === option.value"
                    :title="option.hint"
                    :aria-description="option.hint"
                    :data-testid="`pet-layer-${option.value}`"
                    @click="draft.layer = option.value"
                  >
                    <component :is="option.icon" :size="14" />
                    <span>{{ option.label }}</span>
                    <Check :size="13" :stroke-width="2.6" class="chk" />
                  </button>
                </div>
              </div>

              <div class="cfg-row" title="开启后，伙伴会对桌面特效、问候和离开后的重逢做出回应" aria-description="开启后，伙伴会对桌面特效、问候和离开后的重逢做出回应">
                <span class="cfg-lbl grow">回应桌面状态</span>
                <NSwitch v-model:value="draft.links" size="small" data-testid="pet-dialog-links" />
              </div>
            </div>

            <div class="cfg-card">
              <button type="button" class="jump-row" data-testid="pet-dialog-mood" @click="tab = 'mood'">
                <span class="cfg-lbl">心情</span>
                <span class="jump-val">{{ EXPRESSIONS[draft.expression].label }} · {{ groupOf(draft.expression) }}<ChevronRight :size="14" /></span>
              </button>
            </div>

            <slot name="extra" />
          </template>

          <!-- 心情分区：4 列全览 + 悬停实时预览 -->
          <div v-else class="mood-board-wrap">
            <div class="mood-toolbar">
              <div class="mood-filter-pills" role="group" aria-label="心情分组">
                <button
                  v-for="opt in (['all', '日常', '情绪', '状态'] as const)"
                  :key="opt"
                  type="button"
                  :aria-pressed="moodFilter === opt"
                  @click="moodFilter = opt"
                >
                  {{ opt === 'all' ? `全部 ${EXPRESSION_NAMES.length}` : opt }}
                </button>
              </div>
              <button type="button" class="xb-btn-ghost mood-random" title="随机切换一个心情" data-testid="pet-mood-random" @click="pickRandomMood">随机试试</button>
            </div>

            <div class="mood-groups">
              <template v-for="group in visibleGroups" :key="group.name">
                <div v-if="moodFilter === 'all'" class="mood-section-label">{{ group.name }}</div>
                <div class="mood-board-grid" role="listbox" :aria-label="group.name">
                  <button
                    v-for="name in group.items"
                    :key="name"
                    type="button"
                    role="option"
                    class="mood-tile"
                    :class="{ on: draft.expression === name }"
                    :aria-selected="draft.expression === name"
                    :data-testid="`pet-mood-${name}`"
                    @pointerenter="hoverMood = name"
                    @focus="hoverMood = name"
                    @pointerleave="clearHover(name)"
                    @blur="clearHover(name)"
                    @keydown="onMoodKeydown($event, name)"
                    @click="pickMood(name)"
                  >
                    <span>{{ EXPRESSIONS[name].label }}</span>
                    <Check v-if="draft.expression === name" :size="12" :stroke-width="2.8" />
                  </button>
                </div>
              </template>
            </div>

            <div class="mood-hover-note">
              <span>悬停或键盘聚焦即可在左侧实时试看表情</span>
              <span>点击确认</span>
            </div>
          </div>
        </div>

        <footer class="pd-foot">
          <NPopconfirm positive-text="移除" negative-text="取消" @positive-click="emit('remove')">
            <template #trigger><button type="button" class="remove" :disabled="busy" data-testid="pet-dialog-remove">从桌面移除</button></template>
            移除这只宠物？主题作品仍会保留。
          </NPopconfirm>
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-cancel" @click="close">取消</button>
          <button type="button" class="xb-btn-primary is-sm" :disabled="busy || nameInvalid" data-testid="pet-dialog-done" @click="done">完成</button>
        </footer>
      </div>
    </section>
  </component>
</template>

<style scoped>
.pet-detached-host{width:100%;height:100%}
.pet-dialog{display:grid;grid-template-columns:260px minmax(0,1fr);background:#f4f4f6;color:var(--ui-text-primary);font-family:var(--ui-font-family);overflow:hidden}
html[data-theme=dark] .pet-dialog{background:#1b1b1f}
.pet-dialog.is-detached{width:100%;height:100%}
.pet-dialog.is-modal{width:704px;max-width:calc(100vw - 32px);height:504px;max-height:calc(100vh - 48px);border-radius:var(--ui-radius-dialog);box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog)}

/* 左栏：纯净无框白展台 */
.pd-left{background:radial-gradient(ellipse 90% 48% at 50% 48%,#fff 0%,rgba(255,255,255,0) 70%),linear-gradient(180deg,#fff 0%,#fff 64%,#f9f8f6 100%);border-right:1px solid var(--ui-line);padding:14px 18px 14px;display:flex;flex-direction:column;align-items:center;min-height:0}
html[data-theme=dark] .pd-left{background:radial-gradient(ellipse 86% 48% at 50% 46%,rgba(255,255,255,.045) 0%,transparent 72%),linear-gradient(180deg,#26262a 0%,#232327 68%,#1f1f23 100%);border-right-color:rgba(255,255,255,.07)}
.is-detached .pd-left{padding-top:4px}
.pd-left>:deep(.configuration-chrome){align-self:stretch;margin:0 -8px 0}
.pd-hero{flex:1;width:100%;min-height:0;display:flex;flex-direction:column;align-items:center;justify-content:center;gap:6px}
.pd-stage{width:224px;height:224px;flex:none;position:relative;display:flex;align-items:center;justify-content:center;background:transparent}
.pd-stage::after{content:'';position:absolute;left:50%;bottom:14px;transform:translateX(-50%);width:132px;height:20px;border-radius:50%;pointer-events:none;background:radial-gradient(ellipse at center,rgba(60,48,36,.1) 0%,transparent 72%)}
html[data-theme=dark] .pd-stage::after{background:radial-gradient(ellipse at center,rgba(0,0,0,.42) 0%,transparent 74%)}
.pd-stage.is-off{opacity:.44;filter:grayscale(.55)}
.pd-preview-tag{position:absolute;top:2px;left:50%;transform:translateX(-50%);height:22px;padding:0 9px;border-radius:999px;background:var(--ui-soft);color:var(--ui-ink);font-size:11px;font-weight:500;display:inline-flex;align-items:center;opacity:0;transition:opacity .15s ease;pointer-events:none;white-space:nowrap;z-index:2}
.pd-preview-tag.is-visible{opacity:1}

.pd-name-wrap{width:100%;display:flex;flex-direction:column;align-items:center;gap:2px}
.pd-name-field{position:relative;width:100%;display:flex;align-items:center;justify-content:center}
.pd-name-input{width:100%;height:30px;padding:0 24px;border:0;border-radius:8px;text-align:center;background:transparent;color:var(--ui-text-primary);font:inherit;font-size:15px;font-weight:600;outline:none;transition:background .14s,box-shadow .14s}
.pd-name-input:hover{background:var(--ui-surface-hover)}
.pd-name-input:focus{background:var(--ui-configuration-surface);box-shadow:0 0 0 2px var(--ui-btn)}
.pd-name-input.is-err{box-shadow:0 0 0 1.5px var(--ui-danger)}
.pd-name-pencil{position:absolute;right:8px;color:var(--ui-text-tertiary);opacity:0;pointer-events:none;transition:opacity .14s}
.pd-name-field:hover .pd-name-pencil{opacity:1}
.pd-name-field:focus-within .pd-name-pencil{opacity:0}
.cfg-err{font-size:11px;color:var(--ui-danger)}
.pd-meta{font-size:11.5px;color:var(--ui-text-secondary);max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pd-hint{font-size:11px;color:var(--ui-text-tertiary);margin-top:-2px}
.pd-left-acts{width:100%;display:flex;flex-direction:column;gap:2px;padding-top:6px;flex:none}
.pd-left-acts .xb-btn-ghost{width:100%;justify-content:flex-start;height:30px;font-size:12px}

/* 右栏：轻灰画布 + 白卡 */
.pd-right{display:flex;flex-direction:column;min-width:0;min-height:0}
.pd-head{flex:none;display:flex;align-items:center;gap:10px;padding:14px 20px 10px}
.pd-nav-seg{padding:3px;border-radius:9px;background:var(--ui-configuration-control);display:inline-flex;gap:2px}
.pd-nav-seg button{height:26px;padding:0 13px;border:0;border-radius:7px;background:transparent;font-size:12.5px;color:var(--ui-text-secondary);cursor:pointer;transition:background .14s,color .14s,box-shadow .14s}
.pd-nav-seg button:hover{color:var(--ui-text-primary)}
.pd-nav-seg button[aria-selected=true]{background:var(--ui-configuration-surface);color:var(--ui-text-primary);font-weight:600;box-shadow:0 1px 2px rgba(0,0,0,.09)}
html[data-theme=dark] .pd-nav-seg button[aria-selected=true]{background:#48484c}
.pd-status{margin-left:auto;height:22px;padding:0 8px;border-radius:999px;background:var(--ui-configuration-control);color:var(--ui-text-secondary);font-size:11px;font-weight:500;display:inline-flex;align-items:center;gap:5px;flex:none}
.pd-status i{width:5.5px;height:5.5px;border-radius:50%;background:var(--ui-success)}
.pd-status.off i{background:var(--ui-text-disabled)}
.pd-close{margin-left:4px}

.pd-scroll{flex:1;min-height:0;overflow-y:auto;padding:2px 20px 16px;display:flex;flex-direction:column;gap:10px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent}
.cfg-card{background:var(--ui-configuration-group);border-radius:12px;padding:2px 14px;box-shadow:0 1px 2px rgba(32,26,20,.035),0 0 0 1px rgba(32,26,20,.055)}
html[data-theme=dark] .cfg-card{background:#28272b;box-shadow:0 0 0 1px rgba(255,255,255,.065)}
.cfg-row{display:flex;align-items:center;gap:12px;min-height:44px;padding:8px 0;border-top:1px solid var(--ui-line)}
.cfg-row:first-child{border-top:0}
.cfg-row.col{flex-direction:column;align-items:stretch;gap:8px;padding:11px 0}
.cfg-lbl{font-size:13px;font-weight:500;color:var(--ui-text-primary);min-width:60px;flex:none}
.cfg-lbl.grow{flex:1}

.size-head{display:flex;align-items:center;gap:8px}
.size-presets{display:flex;gap:4px}
.size-presets button{height:22px;padding:0 8px;border:0;border-radius:6px;background:var(--ui-configuration-control);color:var(--ui-text-secondary);font-size:11px;cursor:pointer}
.size-presets button.active{background:var(--ui-surface);color:var(--ui-text-primary);font-weight:600;box-shadow:0 1px 2px rgba(0,0,0,.1)}
html[data-theme=dark] .size-presets button.active{background:#48484c}
.size-val{margin-left:auto;font-size:12.5px;font-weight:500;color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.pd-range{-webkit-appearance:none;appearance:none;width:100%;height:5px;margin:4px 0 2px;border-radius:999px;background:linear-gradient(to right,var(--ui-btn) 0%,var(--ui-btn) var(--range-pct,38%),var(--ui-configuration-control) var(--range-pct,38%),var(--ui-configuration-control) 100%);outline:none;cursor:pointer}
.pd-range::-webkit-slider-thumb{-webkit-appearance:none;appearance:none;width:16px;height:16px;border-radius:50%;background:#fff;box-shadow:0 1px 4px rgba(0,0,0,.22),0 0 0 1px rgba(0,0,0,.08);transition:transform .12s}
.pd-range::-webkit-slider-thumb:hover{transform:scale(1.08)}
.pd-range::-moz-range-thumb{width:16px;height:16px;border:0;border-radius:50%;background:#fff;box-shadow:0 1px 4px rgba(0,0,0,.22),0 0 0 1px rgba(0,0,0,.08)}

.layer-cards{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}
.layer-card{height:36px;padding:0 11px;border:0;border-radius:9px;background:#f4f2ee;color:var(--ui-text-secondary);display:flex;align-items:center;gap:7px;font-size:12.5px;font-weight:500;cursor:pointer;transition:background .14s,color .14s,box-shadow .14s}
html[data-theme=dark] .layer-card{background:#313036}
.layer-card:hover{color:var(--ui-text-primary);background:#eeeae4}
html[data-theme=dark] .layer-card:hover{background:#38373e}
.layer-card[aria-checked=true]{background:var(--ui-configuration-surface);color:var(--ui-text-primary);font-weight:600;box-shadow:0 0 0 1.5px color-mix(in srgb,var(--ui-btn) 42%,var(--ui-border-strong)),0 2px 6px rgba(32,26,20,.06)}
.layer-card .chk{margin-left:auto;opacity:0;color:var(--ui-btn)}
.layer-card[aria-checked=true] .chk{opacity:1}

.jump-row{width:100%;min-height:44px;padding:6px 0;border:0;background:transparent;display:flex;align-items:center;gap:10px;text-align:left;cursor:pointer}
.jump-val{margin-left:auto;display:inline-flex;align-items:center;gap:5px;font-size:12.5px;color:var(--ui-text-secondary)}
.jump-row:hover .jump-val{color:var(--ui-text-primary)}

/* 心情 4 列免滚动面板 */
.mood-board-wrap{display:flex;flex-direction:column;gap:8px}
.mood-toolbar{display:flex;align-items:center;justify-content:space-between;gap:8px}
.mood-filter-pills{display:inline-flex;gap:4px}
.mood-filter-pills button{height:24px;padding:0 9px;border:0;border-radius:7px;background:transparent;font-size:11.5px;color:var(--ui-text-secondary);cursor:pointer}
.mood-filter-pills button:hover{background:var(--ui-surface-hover);color:var(--ui-text-primary)}
.mood-filter-pills button[aria-pressed=true]{background:var(--ui-configuration-group);color:var(--ui-text-primary);font-weight:600;box-shadow:0 1px 2px rgba(0,0,0,.07)}
.mood-random{height:24px;font-size:11.5px}
.mood-section-label{font-size:11px;font-weight:600;color:var(--ui-text-tertiary);padding:6px 2px 4px}
.mood-section-label:first-child{padding-top:2px}
.mood-board-grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:6px}
.mood-tile{height:32px;padding:0 8px;border:0;border-radius:8px;background:var(--ui-configuration-group);color:var(--ui-text-primary);font-size:12.5px;display:flex;align-items:center;justify-content:space-between;gap:4px;cursor:pointer;box-shadow:0 1px 1.5px rgba(32,26,20,.03),inset 0 0 0 1px rgba(32,26,20,.065);transition:background .12s,box-shadow .12s,color .12s}
html[data-theme=dark] .mood-tile{background:#28272b;box-shadow:inset 0 0 0 1px rgba(255,255,255,.07)}
.mood-tile:hover,.mood-tile:focus-visible{background:#f6f3ee;box-shadow:inset 0 0 0 1px rgba(32,26,20,.14);outline:none}
html[data-theme=dark] .mood-tile:hover,html[data-theme=dark] .mood-tile:focus-visible{background:#333238;box-shadow:inset 0 0 0 1px rgba(255,255,255,.16)}
.mood-tile.on{background:var(--ui-soft);color:var(--ui-ink);font-weight:600;box-shadow:inset 0 0 0 1.25px color-mix(in srgb,var(--ui-btn) 38%,transparent)}
.mood-hover-note{display:flex;align-items:center;justify-content:space-between;padding:2px 2px 0;font-size:11px;color:var(--ui-text-tertiary)}

.pd-foot{flex:none;display:flex;align-items:center;gap:8px;padding:11px 20px 13px;border-top:1px solid var(--ui-line);background:#f4f4f6}
html[data-theme=dark] .pd-foot{background:#1b1b1f}
.pd-foot .remove{margin-right:auto;height:30px;padding:0 6px;border:0;border-radius:7px;background:transparent;color:var(--ui-danger);font-size:12.5px;cursor:pointer}
.pd-foot .remove:hover{background:rgba(217,58,68,.08)}
.pd-foot .xb-btn-primary{border:0;background:var(--ui-btn);box-shadow:none}

@media(max-width:600px){
  .pet-dialog{grid-template-columns:1fr}
  .pd-left{border-right:0;border-bottom:1px solid var(--ui-line)}
  .pd-stage{width:170px;height:170px}
}
</style>
