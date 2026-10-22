<script setup lang="ts">
import StudioActions from '../components/StudioActions.vue';
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, onBeforeUnmount, onDeactivated, onMounted, ref, watch } from 'vue';
import { NAlert, NPopconfirm, NSlider, NSwitch } from 'naive-ui';
import { Ban, Sparkles, Trash2, Download } from 'lucide-vue-next';
import { normalizeChoice, paramsOf } from '../../effects/parse.ts';
import { defaultParams, limits, type EffectsChoice } from '../../effects/types.ts';
import type { ControlStore } from '../control-types.ts';
import EffectPreview from '../components/EffectPreview.vue';
const props = defineProps<{ store: ControlStore }>();

const choice = computed(() => normalizeChoice(props.store.settings.effects));
const category=ref<'builtin'|'custom'>('builtin');
const packs = computed(() => props.store.effectList.packs);
const counts=computed(()=>({builtin:packs.value.filter(pack=>pack.source!=='user').length,custom:packs.value.filter(pack=>pack.source==='user').length}));
const visiblePacks=computed(()=>packs.value.filter(pack=>category.value==='custom'?pack.source==='user':pack.source!=='user'));
const chosen = computed(() => packs.value.find(pack => pack.id === choice.value.selected) ?? null);
// A saved desktop selection does not autoplay when this page opens.
const previewId = ref('');
const hoveredId = ref('');
const playingId = computed(() => hoveredId.value || previewId.value);
// While the list is still loading nothing can be missing; afterwards a chosen effect that is gone is worth saying.
const missing = computed(() => choice.value.selected !== '' && packs.value.length > 0 && !chosen.value);
const message = (error: unknown) => error instanceof Error ? error.message : String(error);

// The sliders follow the chosen effect; each effect keeps its own density, speed and size.
const density = ref(1), speed = ref(1), size = ref(1);
let timer: ReturnType<typeof setTimeout> | null = null;
watch(() => [chosen.value?.id, chosen.value ? JSON.stringify(paramsOf(choice.value, chosen.value.id)) : ''], () => {
  if (timer || !chosen.value) return;
  const params = paramsOf(choice.value, chosen.value.id);
  density.value = params.density; speed.value = params.speed; size.value = params.size;
}, { immediate: true });

async function apply(next: EffectsChoice) {
  try { await props.store.setEffects(next); } catch (error) { props.store.notify(message(error), 'error'); }
}
function choose(id: string) { previewId.value = id; void apply({ selected: id, params: choice.value.params }); }
function commit() {
  timer = null;
  if (!chosen.value) return;
  void apply({ selected: chosen.value.id, params: { ...choice.value.params, [chosen.value.id]: { density: density.value, speed: speed.value, size: size.value } } });
}
/** 恢复默认: this effect's density, speed and size go back to 1. */
function restoreDefaults() {
  if (!chosen.value) return;
  if (timer) { clearTimeout(timer); timer = null; }
  density.value = defaultParams.density; speed.value = defaultParams.speed; size.value = defaultParams.size;
  commit();
}
/** 基于它创作: opens AI creation with this effect as the starting point. */
function createFrom() {
  const pack = chosen.value;
  void props.store.startCreation('effect').then(() => {
    if (pack) props.store.creationDraft = `请以桌面特效「${pack.name}」（${pack.id}）为参考，创作我的自定义特效。先读取现有特效，按我的要求调整：`;
  }).catch(error => props.store.notify(message(error), 'error'));
}
function setLinks(enabled: boolean) { void props.store.setCompanionLinks(enabled).catch(error => props.store.notify(message(error), 'error')); }
function tune() { if (timer) clearTimeout(timer); timer = setTimeout(commit, 150); }
async function reload(announce: boolean) {
  try { await props.store.loadEffects(); if (announce) props.store.notify('已重新载入特效', 'success'); }
  catch (error) { props.store.notify(message(error), 'error'); }
}
async function importPack() {
  try {
    const outcome = await props.store.importEffectPack();
    if (outcome.status === 'imported') {category.value='custom';props.store.notify(`已导入特效「${outcome.name}」，可以选用了`, 'success');}
    else if (outcome.status === 'declined') props.store.notify(`特效「${outcome.id}」已经存在，没有替换`, 'info');
  } catch (error) { props.store.notify(message(error), 'error'); }
}
async function removeEffect(id: string, name: string) {
  try {
    await props.store.deleteEffect(id);
    props.store.notify(`已删除特效「${name}」`, 'success');
  } catch (error) {
    props.store.notify(message(error), 'error');
  }
}
onMounted(() => { void reload(false); });
onDeactivated(() => { hoveredId.value = ''; previewId.value = ''; });
onBeforeUnmount(() => { if (timer) { clearTimeout(timer); commit(); } });
</script>
<template>
  <section class="effects-page" data-testid="effects-page">
    <StudioActions>
      <button type="button" class="xb-btn" data-testid="effects-import" @click="importPack"><Download :size="14" />导入</button>
      <button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="effects-create" @click="store.startCreation('effect')"><Sparkles :size="14" />让小伴创作</button>
    </StudioActions>
    <div class="fx-browser">
      <div class="fx-toolbar">
        <div class="xb-seg" role="group" aria-label="特效来源"><button type="button" :aria-pressed="category === 'builtin'" data-testid="effects-scope-builtin" @click="category = 'builtin'">官方 {{ counts.builtin }}</button><button type="button" :aria-pressed="category === 'custom'" data-testid="effects-scope-custom" @click="category = 'custom'">我的 {{ counts.custom }}</button></div>
        <span class="fx-hint">特效叠在任何壁纸之上，点一下立即生效</span>
      </div>
      <div class="fx-main">
      <NAlert v-if="missing" type="warning" :show-icon="false" data-testid="effect-missing">所选的特效「{{ choice.selected }}」已经找不到（可能被移出了特效文件夹），桌面上暂时没有特效。重新选一个即可。</NAlert>
      <NAlert v-if="store.effectList.errors.length" type="warning" :show-icon="false" data-testid="effect-errors">有 {{ store.effectList.errors.length }} 个自定义特效无法使用：{{ store.effectList.errors.join('；') }}</NAlert>
      <div class="effect-grid" role="group" aria-label="特效">
        <button class="effect-card none" :class="{ on: !chosen }" :aria-pressed="!chosen" data-testid="effect-none" @click="choose('')">
          <span class="effect-stage"><Ban :size="26" :stroke-width="1.6" /><i v-if="!chosen" class="lb">使用中</i></span>
          <span class="effect-name"><b>无特效</b></span>
        </button>
        <div v-for="pack in visiblePacks" :key="pack.id" class="effect-card-wrapper" :class="{ user: pack.source === 'user' }">
          <button class="effect-card" :class="{ on: chosen?.id === pack.id }" :aria-pressed="chosen?.id === pack.id" :data-testid="`effect-${pack.id}`" :title="pack.description" @click="choose(pack.id)" @mouseenter="hoveredId = pack.id" @mouseleave="hoveredId = ''">
            <span class="effect-stage"><EffectPreview :pack="pack" :playing="playingId === pack.id" :paused="store.settings.paused" :fps="store.settings.fps" /><i v-if="chosen?.id === pack.id" class="lb">使用中</i></span>
            <span class="effect-name"><b>{{ pack.name }}</b></span>
          </button>
          <div v-if="pack.source === 'user'" class="card-tools">
            <ResourceShareActions icons :store="store" kind="effect" :id="pack.id" />
            <NPopconfirm positive-text="删除" negative-text="取消" @positive-click="removeEffect(pack.id, pack.name)">
              <template #trigger><button class="effect-delete-btn" title="删除这个自定义特效" :aria-label="`删除特效 ${pack.name}`" :data-testid="`delete-effect-${pack.id}`" @click.stop><Trash2 :size="13" /></button></template>
              确定要删除自定义特效「{{ pack.name }}」吗？
            </NPopconfirm>
          </div>
        </div>
      </div>
      <div v-if="category === 'custom' && !visiblePacks.length" class="xb-empty" data-testid="custom-effects-empty"><span class="xb-empty-icon"><Sparkles :size="24" /></span><h3>还没有自定义特效</h3><p>点右上角“导入”选一个特效包文件（JSON），也可以直接在聊天里让小伴为你写一个。</p></div>
      <p class="fx-notes">特效显示在桌面图片之上、桌面图标和其他窗口之下，鼠标可以穿过它。{{ store.settings.paused ? '桌面动画目前已暂停，特效会静止。' : '' }}{{ store.game.active ? '游戏空间打开期间特效会先隐藏，退出后自动回来。' : '' }}</p>
    </div>
    </div>
    <aside class="fx-inspector" aria-label="特效设置" data-testid="effect-inspector">
      <div class="fx-preview" :class="{ none: !chosen }">
        <EffectPreview v-if="chosen" :pack="chosen" :playing="true" :paused="store.settings.paused" :fps="store.settings.fps" />
        <Ban v-else :size="30" :stroke-width="1.4" />
      </div>
      <div class="fx-title"><b>{{ chosen ? chosen.name : '无特效' }}</b><span>{{ chosen ? chosen.description : '桌面保持安静，不叠加任何特效。' }}</span></div>
      <section v-if="chosen" class="fx-tuning" data-testid="effect-tuning">
        <div class="kv"><div class="h"><span>数量</span><span data-testid="effect-density-value">×{{ density.toFixed(1) }}</span></div><NSlider v-model:value="density" :min="limits.density[0]" :max="limits.density[1]" :step="0.1" :tooltip="false" @update:value="tune" /></div>
        <div class="kv"><div class="h"><span>速度</span><span data-testid="effect-speed-value">×{{ speed }}</span></div><NSlider v-model:value="speed" :min="limits.speed[0]" :max="limits.speed[1]" :step="0.25" :tooltip="false" @update:value="tune" /></div>
        <div class="kv"><div class="h"><span>大小</span><span data-testid="effect-size-value">×{{ size.toFixed(1) }}</span></div><NSlider v-model:value="size" :min="limits.size[0]" :max="limits.size[1]" :step="0.1" :tooltip="false" @update:value="tune" /></div>
      </section>
      <div class="kv row"><div class="grow"><span>伙伴跟着做表情</span><small>只换表情，不弹窗、不出声</small></div><NSwitch :value="store.settings.companionLinks" data-testid="effect-companion-links" @update:value="setLinks" /></div>
      <div class="fx-buttons"><button type="button" class="xb-btn" :disabled="!chosen" data-testid="effect-reset" @click="restoreDefaults">恢复默认</button><button type="button" class="xb-btn" :disabled="!store.desktop" data-testid="effect-derive" @click="createFrom">基于它创作</button></div>
    </aside>
  </section>
</template>
<style scoped>
.effects-page{height:100%;min-height:0;min-width:0;display:flex;margin-top:18px;overflow:hidden}
.fx-browser{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column}
.fx-main{flex:1;min-width:0;min-height:0;display:flex;flex-direction:column;gap:12px;padding:0 24px 28px 28px;overflow-y:auto;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.fx-main>*{flex-shrink:0}
.fx-toolbar{flex:none;display:flex;align-items:center;gap:8px;padding:0 24px 12px 28px;background:var(--ui-surface)}
.fx-hint{margin-left:auto;font-size:12px;color:var(--ui-text-secondary)}
.effect-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:18px 14px;margin-top:2px}
.effect-card-wrapper{position:relative;min-width:0}
.effect-card{width:100%;display:flex;flex-direction:column;gap:8px;padding:0;border:0;background:transparent;text-align:left;cursor:pointer;color:var(--ui-text-primary)}
.effect-stage{position:relative;display:block;aspect-ratio:16/10;border-radius:12px;overflow:hidden;box-shadow:0 0 0 1px var(--ui-line),0 2px 6px var(--ui-shadow-hover);transition:box-shadow .15s}
.effect-card.on .effect-stage{box-shadow:0 0 0 2px var(--ui-surface),0 0 0 4px var(--ui-btn)}
.effect-card:not(.on):hover .effect-stage{box-shadow:0 0 0 1px var(--ui-border-strong),0 4px 12px var(--ui-shadow-hover)}
.none .effect-stage{display:grid;place-items:center;background:var(--ui-control);color:var(--ui-text-tertiary)}
.lb{position:absolute;top:8px;left:8px;z-index:2;height:20px;padding:0 7px;border-radius:6px;background:var(--ui-btn);color:#fff;font-size:11px;font-weight:500;font-style:normal;display:inline-flex;align-items:center}
.effect-name{display:block;padding:0 2px}
.effect-name b{font-size:13.5px;font-weight:500}
.card-tools{position:absolute;top:6px;right:6px;z-index:3;display:flex;gap:4px;opacity:0;transition:opacity .15s}
.effect-card-wrapper:hover .card-tools,.card-tools:focus-within{opacity:1}
.effect-delete-btn{width:26px;height:26px;border-radius:50%;border:0;background:rgba(0,0,0,.5);color:#fff;display:grid;place-items:center;cursor:pointer;transition:background .15s}
.effect-delete-btn:hover{background:var(--ui-danger)}
.fx-notes{margin:6px 0 0;font-size:12px;line-height:1.7;color:var(--ui-text-tertiary)}
.fx-inspector{width:300px;flex:none;min-height:0;overflow-y:auto;border-left:1px solid var(--ui-border);padding:2px 22px 20px;display:flex;flex-direction:column;scrollbar-width:thin}
.fx-preview{position:relative;aspect-ratio:16/10;border-radius:12px;overflow:hidden;box-shadow:0 0 0 1px var(--ui-line),0 8px 20px var(--ui-shadow-hover);flex:none}
.fx-preview.none{display:grid;place-items:center;background:var(--ui-control);color:var(--ui-text-tertiary)}
.fx-title{margin-top:14px;display:flex;flex-direction:column;gap:3px}
.fx-title b{font-size:16px;font-weight:600}
.fx-title span{font-size:12px;line-height:1.55;color:var(--ui-text-secondary)}
.fx-tuning{margin-top:14px}
.kv{display:flex;flex-direction:column;gap:6px;margin-top:14px;font-size:13px}
.kv .h{display:flex;justify-content:space-between}
.kv .h span:last-child{color:var(--ui-text-secondary);font-variant-numeric:tabular-nums}
.kv.row{flex-direction:row;align-items:center;gap:10px}
.kv .grow{flex:1;display:flex;flex-direction:column}
.kv small{font-size:11.5px;color:var(--ui-text-secondary)}
.fx-buttons{margin-top:auto;padding-top:20px;display:flex;gap:8px}
.fx-buttons .xb-btn{flex:1}
@media(max-width:1000px){.effect-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>

