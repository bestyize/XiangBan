<script setup lang="ts">
/**
 * 伙伴设置: focused configuration view for one desktop pet instance.
 * Runs inside a native detached configuration window on desktop, and falls back to an NModal in browser previews.
 */
import { computed, h, reactive, ref, watch, type VNodeChild } from 'vue';
import { NModal, NPopconfirm, NSelect, NSwitch, type SelectGroupOption, type SelectOption } from 'naive-ui';
import { Check, ChevronsUpDown, Crosshair, Sparkles, X } from 'lucide-vue-next';
import PetActor from '../../pets/PetActor.vue';
import { EXPRESSIONS, EXPRESSION_NAMES, stillFrame, type ExpressionName } from '../../companion/expressions.ts';
import type { PetDefinition, PetInstance, PetTheme } from '../../pets/model.ts';
import { petMarkup } from '../../pets/render.ts';
import WindowChrome from '../configuration/WindowChrome.vue';
import { dragConfigurationWindow, zoomConfigurationWindow } from '../configuration/windows.ts';

type Theme = PetTheme & { source: 'builtin' | 'custom' };
export interface PetSettingsPatch extends Partial<Pick<PetInstance, 'name' | 'size' | 'expression' | 'layer' | 'links' | 'themeId' | 'petId'>> {}

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
const groupOf = (name: ExpressionName) => groups.value.find(group => group.items.includes(name))?.name ?? '';

const layers = [
  { value: 'desktop' as const, label: '贴在桌面', hint: '在桌面图标下方，不挡住窗口' },
  { value: 'topmost' as const, label: '始终置顶', hint: '浮在所有窗口上方，随时能戳到' },
];
const sizePresets = [
  { label: '小', value: 140 },
  { label: '标准', value: 210 },
  { label: '大', value: 300 },
];

const draft = reactive({ name: '', size: 210, expression: 'idle' as ExpressionName, layer: 'topmost' as PetInstance['layer'], links: true, plant: '' });
const moodOpen = ref(false);
const thumbCache = new WeakMap<PetDefinition, string>();
function thumbOf(pet: PetDefinition) {
  let src = thumbCache.get(pet);
  if (!src) {
    src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(petMarkup(pet, stillFrame('happy'), 'thumb').replace(/viewBox="[^"]+"/, 'viewBox="0 -72 320 352"'))}`;
    thumbCache.set(pet, src);
  }
  return src;
}

function reset() {
  const item = props.instance;
  if (!item) return;
  Object.assign(draft, {
    name: item.name,
    size: item.size,
    expression: item.expression,
    layer: item.layer,
    links: item.links,
    plant: `${item.themeId}/${item.petId}`,
  });
  moodOpen.value = false;
}
watch(() => [props.show, props.instance?.id, props.detached], () => { if (props.show || props.detached) reset(); }, { immediate: true });

const plants = computed<(SelectGroupOption | SelectOption)[]>(() =>
  props.themes.map(theme => ({
    type: 'group',
    label: theme.name,
    key: theme.id,
    children: theme.pets.map(pet => ({
      label: `${theme.name} · ${pet.name}`,
      value: `${theme.id}/${pet.id}`,
      petName: pet.name,
      pet,
    })),
  })),
);

function renderPlantLabel(option: SelectOption | SelectGroupOption): VNodeChild {
  if (option.type === 'group') return String(option.label ?? '');
  const pet = option.pet as PetDefinition | undefined;
  const label = String(option.petName ?? option.label ?? '');
  if (!pet) return label;
  return h('div', { class: 'plant-opt-row' }, [
    h('span', { class: 'plant-opt-thumb' }, [h('img', { src: thumbOf(pet), alt: '' })]),
    h('span', { class: 'plant-opt-name' }, label),
  ]);
}

const definition = computed(() => {
  const [themeId, petId] = draft.plant.split('/');
  return props.themes.find(theme => theme.id === themeId)?.pets.find(pet => pet.id === petId);
});
const theme = computed(() => props.themes.find(item => item.id === draft.plant.split('/')[0]));
const previewSize = computed(() => Math.round(76 + (Math.min(420, Math.max(80, draft.size)) - 80) / 340 * 108));
const nameInvalid = computed(() => !draft.name.trim());
const metaLine = computed(() => {
  const parts = [theme.value?.name, definition.value?.name ?? '作品暂不可用', props.instance?.monitor].filter(Boolean);
  return parts.join(' · ');
});

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
  const [themeId, petId] = draft.plant.split('/');
  if (themeId !== item.themeId || petId !== item.petId) Object.assign(patch, { themeId, petId });
  if (Object.keys(patch).length) emit('save', patch);
  close();
}
function close() {
  emit('update:show', false);
  emit('close');
}
function pick(name: ExpressionName) { draft.expression = name; moodOpen.value = false; }
function onPointerDown(event: PointerEvent) { if (moodOpen.value && !(event.target as Element).closest('.mood-wrap')) moodOpen.value = false; }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape' && moodOpen.value) { event.stopPropagation(); event.preventDefault(); moodOpen.value = false; } }
</script>

<template>
  <template v-if="detached">
    <section v-if="instance" class="pet-dialog is-detached" role="region" aria-label="伙伴设置" data-testid="pet-settings-dialog" @pointerdown="onPointerDown" @keydown="onKeydown">
      <aside class="pd-left">
        <WindowChrome :platform="platform" title="伙伴设置" />
        <div class="pd-stage">
          <PetActor v-if="definition" :definition="definition" :size="previewSize" :expression="draft.expression" :links="draft.links" :paused="paused" />
        </div>
        <span class="pd-meta" :title="metaLine">{{ metaLine }}</span>
        <span class="pd-hint">戳一戳，会回应你</span>
        <div class="pd-left-acts">
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-center" @click="emit('center')"><Crosshair :size="13" />移到屏幕中间</button>
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-create" @click="emit('create')"><Sparkles :size="13" />让 AI 换个形象</button>
        </div>
      </aside>

      <div class="pd-right">
        <header class="pd-head" @mousedown="dragConfigurationWindow" @dblclick="zoomConfigurationWindow">
          <h2>{{ draft.name.trim() || instance.name }}</h2>
          <span class="pd-status"><i :class="{ off: !instance.enabled }" />{{ instance.enabled ? '在桌面上' : '已隐藏' }}</span>
        </header>

        <div class="pd-scroll">
          <div class="cfg-card">
            <label class="cfg-row">
              <span class="cfg-lbl">名字</span>
              <div class="cfg-grow col">
                <input v-model="draft.name" type="text" maxlength="40" placeholder="输入伙伴昵称" class="pd-input" :class="{ 'is-err': nameInvalid }" data-testid="pet-dialog-name" />
                <span v-if="nameInvalid" class="cfg-err">请输入 1–40 个字符的昵称</span>
              </div>
            </label>

            <div class="cfg-row">
              <span class="cfg-lbl">形象</span>
              <div class="cfg-grow">
                <NSelect v-model:value="draft.plant" :options="plants" :render-label="renderPlantLabel" filterable class="pd-select" data-testid="pet-change-plant" />
              </div>
            </div>

            <div class="cfg-row col">
              <div class="size-head">
                <span class="cfg-lbl">大小</span>
                <div class="size-presets" role="group" aria-label="大小预设">
                  <button v-for="preset in sizePresets" :key="preset.value" type="button" :class="{ active: draft.size === preset.value }" @click="draft.size = preset.value">{{ preset.label }}</button>
                </div>
                <b class="size-val" data-testid="pet-dialog-size">{{ draft.size }} px</b>
              </div>
              <input v-model.number="draft.size" type="range" min="80" max="420" step="10" class="pd-range" aria-label="大小" />
            </div>
          </div>

          <div class="cfg-card">
            <div class="cfg-row">
              <span class="cfg-lbl">心情</span>
              <div class="cfg-grow mood-wrap">
                <button type="button" class="mood-btn" :class="{ open: moodOpen }" aria-haspopup="listbox" :aria-expanded="moodOpen" :aria-label="`心情：${EXPRESSIONS[draft.expression].label}`" data-testid="pet-dialog-mood" @click="moodOpen = !moodOpen">
                  <span>{{ EXPRESSIONS[draft.expression].label }}</span>
                  <span class="grp">{{ groupOf(draft.expression) }}</span>
                  <ChevronsUpDown :size="14" class="chev" />
                </button>
                <div v-if="moodOpen" class="mood-list is-up xb-menu" role="listbox" aria-label="选择心情">
                  <template v-for="group in groups" :key="group.name">
                    <div class="xb-menu-label">{{ group.name }}</div>
                    <div class="mood-grid">
                      <button v-for="name in group.items" :key="name" type="button" role="option" class="mood-opt" :class="{ on: draft.expression === name }" :aria-selected="draft.expression === name" :data-testid="`pet-mood-${name}`" @click="pick(name)">
                        <Check v-if="draft.expression === name" :size="12" :stroke-width="2.5" />{{ EXPRESSIONS[name].label }}
                      </button>
                    </div>
                  </template>
                </div>
              </div>
            </div>

            <div class="cfg-row" title="决定伙伴浮在所有窗口上方，还是贴在桌面图标下方">
              <span class="cfg-lbl">层级</span>
              <div class="cfg-grow end">
                <div class="xb-seg" role="radiogroup" aria-label="层级">
                  <button v-for="option in layers" :key="option.value" type="button" role="radio" :aria-checked="draft.layer === option.value" :title="option.hint" :aria-description="option.hint" :data-testid="`pet-layer-${option.value}`" @click="draft.layer = option.value">{{ option.label }}</button>
                </div>
              </div>
            </div>

            <div class="cfg-row" title="开启后，伙伴会对桌面特效、问候和离开后的重逢做出回应" aria-description="开启后，伙伴会对桌面特效、问候和离开后的重逢做出回应">
              <span class="cfg-lbl grow">回应桌面状态</span>
              <NSwitch v-model:value="draft.links" size="small" data-testid="pet-dialog-links" />
            </div>
          </div>

          <slot name="extra" />
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
  </template>

  <NModal v-else :show="show && !!instance" :auto-focus="false" transform-origin="center" :mask-closable="true" @update:show="value => emit('update:show', value)">
    <section v-if="instance" class="pet-dialog is-modal" role="dialog" aria-modal="true" aria-label="伙伴设置" data-testid="pet-settings-dialog" @pointerdown="onPointerDown" @keydown="onKeydown">
      <aside class="pd-left">
        <div class="pd-stage">
          <PetActor v-if="definition" :definition="definition" :size="previewSize" :expression="draft.expression" :links="draft.links" :paused="paused" />
        </div>
        <span class="pd-meta" :title="metaLine">{{ metaLine }}</span>
        <span class="pd-hint">戳一戳，会回应你</span>
        <div class="pd-left-acts">
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-center" @click="emit('center')"><Crosshair :size="13" />移到屏幕中间</button>
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-create" @click="emit('create')"><Sparkles :size="13" />让 AI 换个形象</button>
        </div>
      </aside>

      <div class="pd-right">
        <header class="pd-head">
          <h2>{{ draft.name.trim() || instance.name }}</h2>
          <span class="pd-status"><i :class="{ off: !instance.enabled }" />{{ instance.enabled ? '在桌面上' : '已隐藏' }}</span>
          <button type="button" class="xb-icon-btn pd-close" aria-label="关闭" data-testid="pet-dialog-close" @click="close"><X :size="16" /></button>
        </header>

        <div class="pd-scroll">
          <div class="cfg-card">
            <label class="cfg-row">
              <span class="cfg-lbl">名字</span>
              <div class="cfg-grow col">
                <input v-model="draft.name" type="text" maxlength="40" placeholder="输入伙伴昵称" class="pd-input" :class="{ 'is-err': nameInvalid }" data-testid="pet-dialog-name" />
                <span v-if="nameInvalid" class="cfg-err">请输入 1–40 个字符的昵称</span>
              </div>
            </label>

            <div class="cfg-row">
              <span class="cfg-lbl">形象</span>
              <div class="cfg-grow">
                <NSelect v-model:value="draft.plant" :options="plants" :render-label="renderPlantLabel" filterable class="pd-select" data-testid="pet-change-plant" />
              </div>
            </div>

            <div class="cfg-row col">
              <div class="size-head">
                <span class="cfg-lbl">大小</span>
                <div class="size-presets" role="group" aria-label="大小预设">
                  <button v-for="preset in sizePresets" :key="preset.value" type="button" :class="{ active: draft.size === preset.value }" @click="draft.size = preset.value">{{ preset.label }}</button>
                </div>
                <b class="size-val" data-testid="pet-dialog-size">{{ draft.size }} px</b>
              </div>
              <input v-model.number="draft.size" type="range" min="80" max="420" step="10" class="pd-range" aria-label="大小" />
            </div>
          </div>

          <div class="cfg-card">
            <div class="cfg-row">
              <span class="cfg-lbl">心情</span>
              <div class="cfg-grow mood-wrap">
                <button type="button" class="mood-btn" :class="{ open: moodOpen }" aria-haspopup="listbox" :aria-expanded="moodOpen" :aria-label="`心情：${EXPRESSIONS[draft.expression].label}`" data-testid="pet-dialog-mood" @click="moodOpen = !moodOpen">
                  <span>{{ EXPRESSIONS[draft.expression].label }}</span>
                  <span class="grp">{{ groupOf(draft.expression) }}</span>
                  <ChevronsUpDown :size="14" class="chev" />
                </button>
                <div v-if="moodOpen" class="mood-list is-up xb-menu" role="listbox" aria-label="选择心情">
                  <template v-for="group in groups" :key="group.name">
                    <div class="xb-menu-label">{{ group.name }}</div>
                    <div class="mood-grid">
                      <button v-for="name in group.items" :key="name" type="button" role="option" class="mood-opt" :class="{ on: draft.expression === name }" :aria-selected="draft.expression === name" :data-testid="`pet-mood-${name}`" @click="pick(name)">
                        <Check v-if="draft.expression === name" :size="12" :stroke-width="2.5" />{{ EXPRESSIONS[name].label }}
                      </button>
                    </div>
                  </template>
                </div>
              </div>
            </div>

            <div class="cfg-row" title="决定伙伴浮在所有窗口上方，还是贴在桌面图标下方">
              <span class="cfg-lbl">层级</span>
              <div class="cfg-grow end">
                <div class="xb-seg" role="radiogroup" aria-label="层级">
                  <button v-for="option in layers" :key="option.value" type="button" role="radio" :aria-checked="draft.layer === option.value" :title="option.hint" :aria-description="option.hint" :data-testid="`pet-layer-${option.value}`" @click="draft.layer = option.value">{{ option.label }}</button>
                </div>
              </div>
            </div>

            <div class="cfg-row" title="开启后，伙伴会对桌面特效、问候和离开后的重逢做出回应" aria-description="开启后，伙伴会对桌面特效、问候和离开后的重逢做出回应">
              <span class="cfg-lbl grow">回应桌面状态</span>
              <NSwitch v-model:value="draft.links" size="small" data-testid="pet-dialog-links" />
            </div>
          </div>

          <slot name="extra" />
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
  </NModal>
</template>

<style scoped>
.pet-dialog{display:grid;grid-template-columns:224px minmax(0,1fr);background:var(--ui-configuration-canvas);color:var(--ui-text-primary);font-family:var(--ui-font-family);overflow:hidden}
.pet-dialog.is-detached{width:100%;height:100%}
.pet-dialog.is-modal{width:656px;max-width:calc(100vw - 32px);max-height:min(580px,calc(100vh - 48px));border-radius:var(--ui-radius-dialog);box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog)}
.pd-left{background:var(--ui-configuration-sidebar);border-right:1px solid var(--ui-line);padding:14px 14px 16px;display:flex;flex-direction:column;align-items:center;gap:8px;min-height:0}
.is-detached .pd-left{padding-top:4px}
.pd-left>:deep(.configuration-chrome){align-self:stretch;margin:0 -6px 2px}
.pd-stage{width:100%;flex:1;min-height:212px;border-radius:14px;box-sizing:border-box;padding:10px;display:flex;align-items:flex-end;justify-content:center;background:radial-gradient(ellipse 72% 56% at 50% 42%,#fff 0%,#f4f4f6 68%,#ececef 100%);overflow:hidden}
html[data-theme=dark] .pd-stage{background:radial-gradient(ellipse 72% 56% at 50% 42%,#2b2c30 0%,#212226 68%,#1b1c20 100%)}
.pd-meta{font-size:11.5px;color:var(--ui-text-secondary);max-width:100%;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pd-hint{font-size:11px;color:var(--ui-text-tertiary);margin-top:-4px}
.pd-left-acts{width:100%;display:flex;flex-direction:column;gap:2px;margin-top:2px}
.pd-left-acts .xb-btn-ghost{width:100%;justify-content:flex-start;height:30px;font-size:12px}

.pd-right{display:flex;flex-direction:column;min-width:0;min-height:0}
.pd-head{flex:none;display:flex;align-items:center;gap:10px;padding:16px 20px 10px}
.pd-head h2{margin:0;font-size:17px;font-weight:600;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pd-status{height:21px;padding:0 8px;border-radius:999px;background:var(--ui-configuration-control);color:var(--ui-text-secondary);font-size:11px;font-weight:500;display:inline-flex;align-items:center;gap:5px;flex:none}
.pd-status i{width:5.5px;height:5.5px;border-radius:50%;background:var(--ui-success)}
.pd-status i.off{background:var(--ui-text-disabled)}
.pd-close{margin-left:auto}

.pd-scroll{flex:1;min-height:0;overflow-y:auto;padding:2px 20px 16px;display:flex;flex-direction:column;gap:12px;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent}
.cfg-card{background:var(--ui-configuration-group);border-radius:12px;padding:2px 14px}
.cfg-row{display:flex;align-items:center;gap:12px;min-height:46px;padding:8px 0;border-top:1px solid var(--ui-line)}
.cfg-row:first-child{border-top:0}
.cfg-row.col{flex-direction:column;align-items:stretch;gap:8px;padding:11px 0}
.cfg-lbl{font-size:13px;font-weight:500;color:var(--ui-text-primary);min-width:60px;flex:none}
.cfg-lbl.grow{flex:1}
.cfg-grow{flex:1;min-width:0;display:flex;align-items:center;gap:8px}
.cfg-grow.col{flex-direction:column;align-items:stretch;gap:3px}
.cfg-grow.end{justify-content:flex-end}
.pd-input{width:100%;height:32px;padding:0 10px;border-radius:8px;border:0;background:var(--ui-configuration-control);color:var(--ui-text-primary);font:inherit;font-size:13px;outline:none}
.pd-input:focus{box-shadow:0 0 0 2px var(--ui-btn);background:var(--ui-configuration-surface)}
.pd-input.is-err{box-shadow:0 0 0 1.5px var(--ui-danger)}
.cfg-err{font-size:11px;color:var(--ui-danger)}

.pd-select{width:100%}
.pd-select :deep(.n-base-selection){background:var(--ui-configuration-control);border-radius:8px}
.pd-select :deep(.n-base-selection__border),.pd-select :deep(.n-base-selection__state-border){border:0;box-shadow:none}
.size-head{display:flex;align-items:center;gap:8px}
.size-presets{display:flex;gap:4px}
.size-presets button{height:22px;padding:0 8px;border:0;border-radius:6px;background:var(--ui-configuration-control);color:var(--ui-text-secondary);font-size:11px;cursor:pointer}
.size-presets button.active{background:var(--ui-surface);color:var(--ui-text-primary);font-weight:600;box-shadow:0 1px 2px rgba(0,0,0,.1)}
html[data-theme=dark] .size-presets button.active{background:#48484c}
.size-val{margin-left:auto;font-size:12.5px;font-weight:500;color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.pd-range{width:100%;margin:0;accent-color:var(--ui-btn)}

.mood-wrap{position:relative}
.mood-btn{width:100%;height:32px;display:flex;align-items:center;gap:8px;padding:0 10px;border:0;border-radius:8px;background:var(--ui-configuration-control);color:var(--ui-text-primary);font-size:13px;text-align:left;cursor:pointer}
.mood-btn:hover{background:var(--ui-control-hover)}
.mood-btn.open{box-shadow:0 0 0 2px var(--ui-btn)}
.mood-btn .grp{font-size:11.5px;color:var(--ui-text-tertiary)}
.mood-btn .chev{margin-left:auto;color:var(--ui-text-tertiary)}
.mood-list{position:absolute;left:0;right:0;top:calc(100% + 6px);z-index:20;max-height:248px;overflow-y:auto;padding:5px 6px 6px;border-radius:12px}
.mood-list.is-up{top:auto;bottom:calc(100% + 6px)}
.mood-list .xb-menu-label{padding:6px 8px 3px;font-size:11px;font-weight:600;color:var(--ui-text-tertiary)}
.mood-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:2px}
.mood-opt{height:28px;border:0;border-radius:7px;background:transparent;color:var(--ui-text-primary);font-size:12.5px;display:flex;align-items:center;gap:5px;padding:0 8px;cursor:pointer;text-align:left;white-space:nowrap}
.mood-opt:hover{background:var(--ui-surface-hover)}
.mood-opt.on{background:var(--ui-control);color:var(--ui-text-primary);font-weight:600}

.pd-foot{flex:none;display:flex;align-items:center;gap:8px;padding:11px 20px 13px;border-top:1px solid var(--ui-line);background:var(--ui-configuration-canvas)}
.pd-foot .remove{margin-right:auto;height:30px;padding:0 6px;border:0;border-radius:7px;background:transparent;color:var(--ui-danger);font-size:12.5px;cursor:pointer}
.pd-foot .remove:hover{background:rgba(217,58,68,.08)}
.pd-foot .xb-btn-primary{border:0;background:var(--ui-btn);box-shadow:none}

@media(max-width:600px){
  .pet-dialog{grid-template-columns:1fr}
  .pd-left{border-right:0;border-bottom:1px solid var(--ui-line)}
  .pd-stage{min-height:150px}
}
</style>
<style>
.plant-opt-row{display:flex;align-items:center;gap:8px}
.plant-opt-thumb{width:22px;height:22px;border-radius:5px;background:var(--ui-control);display:grid;place-items:center;overflow:hidden;flex:none}
.plant-opt-thumb img{width:20px;height:20px;object-fit:contain;display:block}
.plant-opt-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
</style>
