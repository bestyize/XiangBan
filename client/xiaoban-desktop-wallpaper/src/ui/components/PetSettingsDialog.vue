<script setup lang="ts">
/**
 * 伙伴设置: a centred dialog for one pet on the desktop. Left, a big live preview (the size follows the slider);
 * right, name / look / size / mood / layer. Changes stay in a draft until 完成, 取消 throws them away.
 */
import { computed, reactive, ref, watch } from 'vue';
import { NModal, NPopconfirm, NSelect, NSwitch } from 'naive-ui';
import { Check, ChevronsUpDown, Crosshair, Sparkles, X } from 'lucide-vue-next';
import PetActor from '../../pets/PetActor.vue';
import { EXPRESSIONS, EXPRESSION_NAMES, type ExpressionName } from '../../companion/expressions.ts';
import type { PetInstance, PetTheme } from '../../pets/model.ts';

type Theme = PetTheme & { source: 'builtin' | 'custom' };
export interface PetSettingsPatch extends Partial<Pick<PetInstance, 'name' | 'size' | 'expression' | 'layer' | 'links' | 'themeId' | 'petId'>> {}
const props = defineProps<{ show: boolean; instance?: PetInstance; themes: Theme[]; busy?: boolean; paused?: boolean }>();
const emit = defineEmits<{ 'update:show': [boolean]; save: [PetSettingsPatch]; remove: []; center: []; create: [] }>();

/** The 25 expressions, in the three groups of the design. Anything the table gains later lands in 状态. */
const MOOD_GROUPS: { name: string; items: ExpressionName[] }[] = [
  { name: '日常', items: ['idle', 'happy', 'excited', 'laughing', 'proud', 'curious', 'thinking', 'shy', 'love'] },
  { name: '情绪', items: ['confused', 'surprised', 'sad', 'worried', 'angry', 'annoyed', 'crying', 'embarrassed'] },
  { name: '状态', items: ['sleepy', 'sleeping', 'listening', 'speaking', 'greeting', 'waiting', 'offline', 'charging'] },
];
const groups = computed(() => {
  const known = new Set(MOOD_GROUPS.flatMap(group => group.items));
  const extra = EXPRESSION_NAMES.filter(name => !known.has(name));
  return MOOD_GROUPS.map(group => ({ name: group.name, items: (group.name === '状态' ? [...group.items, ...extra] : group.items).filter(name => name in EXPRESSIONS) }));
});
const groupOf = (name: ExpressionName) => groups.value.find(group => group.items.includes(name))?.name ?? '';
const layers = [
  { value: 'desktop' as const, label: '贴在桌面', hint: '在桌面图标下方，不挡住窗口' },
  { value: 'topmost' as const, label: '始终置顶', hint: '浮在所有窗口上方，随时能戳到' },
];

const draft = reactive({ name: '', size: 210, expression: 'idle' as ExpressionName, layer: 'topmost' as PetInstance['layer'], links: true, plant: '' });
const moodOpen = ref(false);
function reset() {
  const item = props.instance;
  if (!item) return;
  Object.assign(draft, { name: item.name, size: item.size, expression: item.expression, layer: item.layer, links: item.links, plant: `${item.themeId}/${item.petId}` });
  moodOpen.value = false;
}
watch(() => [props.show, props.instance?.id], () => { if (props.show) reset(); }, { immediate: true });

const plants = computed(() => props.themes.flatMap(theme => theme.pets.map(pet => ({ label: `${theme.name} · ${pet.name}`, value: `${theme.id}/${pet.id}` }))));
const definition = computed(() => { const [themeId, petId] = draft.plant.split('/'); return props.themes.find(theme => theme.id === themeId)?.pets.find(pet => pet.id === petId); });
const theme = computed(() => props.themes.find(item => item.id === draft.plant.split('/')[0]));
/** Sizes run 80–420 on the desktop; the stage here is 200 wide, so the preview maps them onto 72–184. */
const previewSize = computed(() => Math.round(72 + (Math.min(420, Math.max(80, draft.size)) - 80) / 340 * 112));

function done() {
  const item = props.instance;
  if (!item) { emit('update:show', false); return; }
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
  emit('update:show', false);
}
function close() { emit('update:show', false); }
function pick(name: ExpressionName) { draft.expression = name; moodOpen.value = false; }
function onPointerDown(event: PointerEvent) { if (moodOpen.value && !(event.target as Element).closest('.mood-wrap')) moodOpen.value = false; }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape' && moodOpen.value) { event.stopPropagation(); moodOpen.value = false; } }
</script>
<template>
  <NModal :show="show && !!instance" :auto-focus="false" transform-origin="center" :mask-closable="true" @update:show="value => emit('update:show', value)">
    <section v-if="instance" class="pet-dialog" role="dialog" aria-modal="true" aria-label="伙伴设置" data-testid="pet-settings-dialog" @pointerdown="onPointerDown" @keydown="onKeydown">
      <header class="pd-head">
        <h2>{{ draft.name || instance.name }}</h2>
        <span class="pd-sub">{{ definition?.name ?? '作品暂不可用' }} · {{ theme?.name ?? '' }}</span>
        <button type="button" class="xb-icon-btn" aria-label="关闭" data-testid="pet-dialog-close" @click="close"><X :size="16" /></button>
      </header>
      <div class="pd-body">
        <div class="pd-left">
          <div class="pd-stage"><PetActor v-if="definition" :definition="definition" :size="previewSize" :expression="draft.expression" :links="draft.links" :paused="paused" /></div>
          <span class="pd-hint">戳一戳，会回应你</span>
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-center" @click="emit('center')"><Crosshair :size="13" />移到屏幕中间</button>
          <button type="button" class="xb-btn-ghost" data-testid="pet-dialog-create" @click="emit('create')"><Sparkles :size="13" />让 AI 换个形象</button>
        </div>
        <div class="pd-right">
          <label class="fl"><span>名字</span><input v-model="draft.name" type="text" maxlength="40" class="pd-input" data-testid="pet-dialog-name" /></label>
          <div class="fl"><span>形象</span><NSelect v-model:value="draft.plant" :options="plants" filterable data-testid="pet-change-plant" /></div>
          <label class="fl"><span class="split">大小<b data-testid="pet-dialog-size">{{ draft.size }} px</b></span><input v-model.number="draft.size" type="range" min="80" max="420" step="10" class="pd-range" aria-label="大小" /></label>
          <div class="fl"><span>心情</span>
            <div class="mood-wrap">
              <button type="button" class="mood-btn" :class="{ open: moodOpen }" aria-haspopup="listbox" :aria-expanded="moodOpen" :aria-label="`心情：${EXPRESSIONS[draft.expression].label}`" data-testid="pet-dialog-mood" @click="moodOpen = !moodOpen">
                <i class="xb-dot"></i>{{ EXPRESSIONS[draft.expression].label }}<span class="grp">{{ groupOf(draft.expression) }}</span><ChevronsUpDown :size="14" class="chev" />
              </button>
              <div v-if="moodOpen" class="mood-list xb-menu" role="listbox" aria-label="选择心情">
                <template v-for="group in groups" :key="group.name">
                  <div class="xb-menu-label">{{ group.name }}</div>
                  <div class="mood-grid"><button v-for="name in group.items" :key="name" type="button" role="option" class="mood-opt" :class="{ on: draft.expression === name }" :aria-selected="draft.expression === name" :data-testid="`pet-mood-${name}`" @click="pick(name)"><Check v-if="draft.expression === name" :size="12" :stroke-width="3" />{{ EXPRESSIONS[name].label }}</button></div>
                </template>
              </div>
            </div>
          </div>
          <div class="fl"><span>层级</span>
            <div class="xb-seg" role="radiogroup" aria-label="层级"><button v-for="option in layers" :key="option.value" type="button" role="radio" :aria-checked="draft.layer === option.value" :data-testid="`pet-layer-${option.value}`" @click="draft.layer = option.value">{{ option.label }}</button></div>
            <small>{{ layers.find(option => option.value === draft.layer)?.hint }}</small>
          </div>
          <div class="fl switch-row"><div class="grow"><span>回应桌面状态</span><small>特效、问候和离开后的重逢</small></div><NSwitch v-model:value="draft.links" size="small" data-testid="pet-dialog-links" /></div>
          <slot name="extra" />
        </div>
      </div>
      <footer class="pd-foot">
        <NPopconfirm positive-text="移除" negative-text="取消" @positive-click="emit('remove')"><template #trigger><button type="button" class="remove" :disabled="busy" data-testid="pet-dialog-remove">从桌面移除</button></template>移除这只宠物？主题作品仍会保留。</NPopconfirm>
        <button type="button" class="xb-btn" data-testid="pet-dialog-cancel" @click="close">取消</button>
        <button type="button" class="xb-btn-primary" :disabled="busy" data-testid="pet-dialog-done" @click="done">完成</button>
      </footer>
    </section>
  </NModal>
</template>
<style scoped>
.pet-dialog{width:580px;max-width:calc(100vw - 32px);border-radius:20px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-border),var(--ui-shadow-dialog);color:var(--ui-text-primary);font-family:var(--ui-font-family)}
.pd-head{display:flex;align-items:center;gap:10px;padding:16px 16px 0 22px}
.pd-head h2{margin:0;font-size:16px;font-weight:600;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pd-sub{flex:1;min-width:0;font-size:12px;color:var(--ui-text-secondary);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.pd-body{display:flex;gap:22px;padding:18px 22px 20px}
.pd-left{width:200px;flex:none;display:flex;flex-direction:column;align-items:center;gap:6px}
.pd-stage{width:200px;height:220px;border-radius:16px;box-sizing:border-box;padding-bottom:10px;display:flex;align-items:flex-end;justify-content:center;background:radial-gradient(ellipse 55% 10% at 50% 90%,rgba(0,0,0,.1),transparent 70%),radial-gradient(circle at 50% 40%,var(--ui-soft),var(--ui-surface-sunken) 70%);overflow:hidden}
.pd-hint{font-size:11.5px;color:var(--ui-text-secondary)}
.pd-right{flex:1;min-width:0;display:flex;flex-direction:column;gap:14px}
.fl{display:flex;flex-direction:column;gap:6px;font-size:12.5px;font-weight:500;color:var(--ui-text-secondary)}
.fl small{font-weight:400;font-size:11.5px;color:var(--ui-text-secondary)}
.split{display:flex;justify-content:space-between}
.split b{font-weight:500;color:var(--ui-text-primary);font-variant-numeric:tabular-nums}
.pd-input{height:34px;padding:0 11px;border-radius:9px;border:0;background:var(--ui-surface);color:var(--ui-text-primary);box-shadow:inset 0 0 0 1px var(--ui-border-strong);font:inherit;font-size:14px;font-weight:400;outline:none}
.pd-input:focus{box-shadow:inset 0 0 0 1.5px var(--ui-btn),0 0 0 3px var(--ui-soft)}
.fl .xb-seg{align-self:flex-start}
.pd-range{margin:0;accent-color:var(--ui-btn)}
.mood-wrap{position:relative}
.mood-btn{width:100%;height:34px;display:flex;align-items:center;gap:8px;padding:0 10px 0 12px;border:0;border-radius:9px;background:var(--ui-surface);box-shadow:inset 0 0 0 1px var(--ui-border-strong);color:var(--ui-text-primary);font-size:14px;font-weight:400;text-align:left;cursor:pointer}
.mood-btn.open{box-shadow:inset 0 0 0 1.5px var(--ui-btn),0 0 0 3px var(--ui-soft)}
.mood-btn .grp{font-size:12px;color:var(--ui-text-secondary)}
.mood-btn .chev{margin-left:auto;color:var(--ui-text-tertiary)}
.mood-list{position:absolute;left:0;right:0;top:40px;z-index:5;max-height:268px;overflow-y:auto;padding:4px 6px 6px;border-radius:12px}
.mood-list .xb-menu-label{padding:8px 8px 4px;font-size:11px;font-weight:600;color:var(--ui-text-tertiary)}
.mood-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:2px}
.mood-opt{height:30px;border:0;border-radius:7px;background:transparent;color:var(--ui-text-primary);font-size:13px;display:flex;align-items:center;gap:6px;padding:0 8px;cursor:pointer;text-align:left;white-space:nowrap}
.mood-opt:hover{background:var(--ui-surface-hover)}
.mood-opt.on{background:var(--ui-soft);color:var(--ui-ink);font-weight:500}
.switch-row{flex-direction:row;align-items:center;gap:10px}
.switch-row .grow{flex:1;display:flex;flex-direction:column;gap:2px}
.switch-row .grow span{color:var(--ui-text-primary);font-weight:400;font-size:13px}
.pd-foot{display:flex;align-items:center;gap:8px;padding:12px 16px 14px 22px;border-top:1px solid var(--ui-line);background:var(--ui-surface-sunken);border-radius:0 0 20px 20px}
.pd-foot .remove{margin-right:auto;height:30px;padding:0 4px;border:0;background:transparent;color:var(--ui-danger);font-size:13px;cursor:pointer}
</style>
