<script setup lang="ts">
import StudioActions from '../components/StudioActions.vue';
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, onBeforeUnmount, onDeactivated, onMounted, ref, shallowRef, watch } from 'vue';
import { NAlert, NPopconfirm, NSlider, NSwitch } from 'naive-ui';
import { Ban, Sparkles, Trash2, Download } from 'lucide-vue-next';
import { normalizeChoice, paramsOf } from '../../effects/parse.ts';
import { defaultParams, isArowana, isHtmlEffect, limits, type EffectsChoice, type EffectParams } from '../../effects/types.ts';
import { aquariumLightControls, aquariumModes, defaultAquarium, lightValue, normalizeAquarium, type AquariumLight, type AquariumMode } from '../../effects/arowana/settings.ts';
import type { ControlStore } from '../control-types.ts';
import EffectPreview from '../components/EffectPreview.vue';
import { createWidgetClient } from '../../widgets/client.ts';
import { useLockScreenPicture } from '../wallpaper/useLockScreenPicture.ts';
import { wallpaperFrame } from '../wallpaper/lock-screen-frame.ts';
import LockScreenPicture from '../wallpaper/LockScreenPicture.vue';
const props = defineProps<{ store: ControlStore }>();

const displays = createWidgetClient();
const monitors = computed(() => displays.layout.value.monitors ?? []);
const missingMonitor = computed(() => displays.ready.value && !!choice.value.monitor && choice.value.monitor !== "*" && !monitors.value.some(m => m.id === choice.value.monitor));
const pendingChoice = shallowRef<EffectsChoice>();
const choice = computed(() => pendingChoice.value ?? normalizeChoice(props.store.settings.effects));
const filter=ref<'all'|'desk'|'mine'>('all');
const packs = computed(() => props.store.effectList.packs);
const visiblePacks=computed(()=>packs.value.filter(pack=>filter.value==='mine'?pack.source==='user':filter.value==='desk'?pack.id===choice.value.selected:true));
const chosen = computed(() => packs.value.find(pack => pack.id === choice.value.selected) ?? null);
const aquariumSelected = computed(() => isArowana(chosen.value));
const htmlSelected = computed(() => isHtmlEffect(chosen.value));
const importing = ref(false);
// A saved desktop selection does not autoplay when this page opens.
const previewId = ref('');
const hoveredId = ref('');
const playingId = computed(() => hoveredId.value || previewId.value);
watch(filter,()=>{hoveredId.value='';previewId.value='';});
// While the list is still loading nothing can be missing; afterwards a chosen effect that is gone is worth saying.
const missing = computed(() => choice.value.selected !== '' && packs.value.length > 0 && !chosen.value);
const message = (error: unknown) => error instanceof Error ? error.message : String(error);

// Preview the sliders immediately, then save them together after the drag settles.
const density = ref(1), speed = ref(1), size = ref(1);
const aquarium = ref(normalizeAquarium(undefined));
const tuning = computed<EffectParams>(() => ({
  density: density.value, speed: speed.value, size: size.value,
  ...(aquariumSelected.value ? { aquarium: aquarium.value } : {}),
}));
const lockScreen = useLockScreenPicture(props.store);
const canRenderEffectPicture = computed(() => !!chosen.value && !aquariumSelected.value && !htmlSelected.value);
watch(chosen, lockScreen.close);
function prepareLockScreen() {
  const effect = chosen.value;
  if (!effect || !canRenderEffectPicture.value) return;
  const size = monitors.value.find(monitor => monitor.primary) ?? { width: 1920, height: 1080 };
  const params = tuning.value;
  const { wallpaperId, wallpaperOptions } = props.store.settings;
  void lockScreen.show(() => wallpaperFrame(wallpaperId, wallpaperOptions, { size, effect, params }));
}
let timer: ReturnType<typeof setTimeout> | null = null;
watch(() => [chosen.value?.id, chosen.value ? JSON.stringify(paramsOf(choice.value, chosen.value.id)) : ''] as const, ([id], previous) => {
  if ((timer && previous?.[0] === id) || !chosen.value) return;
  if (timer) { clearTimeout(timer); timer = null; }
  const params = paramsOf(choice.value, chosen.value.id);
  density.value = params.density; speed.value = params.speed; size.value = params.size;
  aquarium.value = normalizeAquarium(params.aquarium);
}, { immediate: true });

async function apply(next: EffectsChoice) {
  // Preserve a tuning change alongside a selection or display change that arrives before the debounce.
  if (timer) {
    clearTimeout(timer); timer = null;
    next = withTuning(next);
  }
  pendingChoice.value = next;
  try { await props.store.setEffects(next); }
  catch (error) { props.store.notify(message(error), 'error'); }
  finally { if (pendingChoice.value === next) pendingChoice.value = undefined; }
}
function choose(id: string) { previewId.value = id; void apply({ ...choice.value, selected: id }); }
function withTuning(next: EffectsChoice): EffectsChoice {
  return chosen.value ? { ...next, params: { ...next.params, [chosen.value.id]: tuning.value } } : next;
}
function commit() {
  if (timer) clearTimeout(timer);
  timer = null;
  if (chosen.value) void apply(withTuning(choice.value));
}
function changeAquariumMode(mode: AquariumMode) {
  aquarium.value = { ...aquarium.value, mode };
  commit();
}
function changeAquascape(aquascape: boolean) {
  aquarium.value = { ...aquarium.value, aquascape };
  commit();
}
function changeLight(key: AquariumLight, value: number) {
  aquarium.value = { ...aquarium.value, lighting: { ...aquarium.value.lighting, [key]: value } };
  tune();
}
function resetLights() {
  aquarium.value = { ...aquarium.value, lighting: { ...defaultAquarium.lighting } };
  commit();
}
function restoreDefaults() {
  if (!chosen.value) return;
  density.value = defaultParams.density; speed.value = defaultParams.speed; size.value = defaultParams.size;
  aquarium.value = normalizeAquarium(undefined);
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
  if (importing.value) return;
  importing.value = true;
  try {
    const outcome = await props.store.importEffectPack();
    if (outcome.status === 'imported') {filter.value='mine';props.store.notify(`已导入特效「${outcome.name}」，可以选用了`, 'success');}
    else if (outcome.status === 'declined') props.store.notify(`特效「${outcome.id}」已经存在，没有替换`, 'info');
  } catch (error) { props.store.notify(message(error), 'error'); }
  finally { importing.value = false; }
}
async function removeEffect(id: string, name: string) {
  try {
    await props.store.deleteEffect(id);
    props.store.notify(`已删除特效「${name}」`, 'success');
  } catch (error) {
    props.store.notify(message(error), 'error');
  }
}
onMounted(() => { void reload(false); void displays.load(); });
onDeactivated(() => { hoveredId.value = ''; previewId.value = ''; });
onBeforeUnmount(() => { displays.dispose(); if (timer) { clearTimeout(timer); commit(); } });
</script>
<template>
  <section class="effects-page" data-testid="effects-page">
    <StudioActions>
      <button type="button" class="xb-btn" data-testid="effects-import" :disabled="importing" title="支持 ZIP、JSON 和单个 HTML 文件" aria-description="支持 ZIP、JSON 和单个 HTML 文件" @click="importPack"><Download :size="14" />导入</button>
      <button type="button" class="xb-btn-primary" :disabled="!store.desktop" data-testid="effects-create" @click="store.startCreation('effect')"><Sparkles :size="14" />让小伴创作</button>
    </StudioActions>
    <div class="fx-browser">
      <div class="fx-toolbar">
        <div class="xb-seg" role="group" aria-label="特效筛选"><button type="button" :aria-pressed="filter === 'all'" data-testid="effects-filter-all" @click="filter = 'all'">全部</button><button type="button" :aria-pressed="filter === 'desk'" data-testid="effects-filter-desk" @click="filter = 'desk'">在桌面上 <b class="cnt">{{ chosen ? 1 : 0 }}</b></button><button type="button" :aria-pressed="filter === 'mine'" data-testid="effects-filter-mine" @click="filter = 'mine'">我的</button></div>
        <span class="fx-hint">特效叠在任何壁纸之上，点一下立即生效</span>
      </div>
      <div class="fx-main">
      <NAlert v-if="missing" type="warning" :show-icon="false" data-testid="effect-missing">所选的特效「{{ choice.selected }}」已经找不到（可能被移出了特效文件夹），桌面上暂时没有特效。重新选一个即可。</NAlert>
      <NAlert v-if="store.effectList.errors.length" type="warning" :show-icon="false" data-testid="effect-errors">有 {{ store.effectList.errors.length }} 个自定义特效无法使用：{{ store.effectList.errors.join('；') }}</NAlert>
      <div v-if="filter === 'all' || visiblePacks.length" class="effect-grid" role="group" aria-label="特效">
        <button v-if="filter !== 'mine'" class="effect-card none" :class="{ on: !chosen }" :aria-pressed="!chosen" data-testid="effect-none" @click="choose('')">
          <span class="effect-stage"><Ban :size="26" :stroke-width="1.6" /><i v-if="!chosen" class="lb">使用中</i></span>
          <span class="effect-name"><b>无特效</b></span>
        </button>
        <div v-for="pack in visiblePacks" :key="pack.id" class="effect-card-wrapper" :class="{ user: pack.source === 'user' }">
          <button class="effect-card" :class="{ on: chosen?.id === pack.id }" :aria-pressed="chosen?.id === pack.id" :data-testid="`effect-${pack.id}`" :title="pack.description" @click="choose(pack.id)" @mouseenter="hoveredId = pack.id" @mouseleave="hoveredId = ''">
            <span class="effect-stage"><EffectPreview :pack="pack" :playing="playingId === pack.id && !((aquariumSelected || isHtmlEffect(pack)) && chosen?.id === pack.id)" :paused="store.settings.paused" :fps="store.settings.fps" /><i v-if="chosen?.id === pack.id" class="lb">使用中</i></span>
            <span class="effect-name"><b>{{ pack.name }}</b></span>
          </button>
          <div v-if="pack.source === 'user'" class="card-tools">
            <ResourceShareActions icons :store="store" kind="effect" :id="pack.id" :local-only="isHtmlEffect(pack)" />
            <NPopconfirm positive-text="删除" negative-text="取消" @positive-click="removeEffect(pack.id, pack.name)">
              <template #trigger><button class="effect-delete-btn" title="删除这个自定义特效" :aria-label="`删除特效 ${pack.name}`" :data-testid="`delete-effect-${pack.id}`" @click.stop><Trash2 :size="13" /></button></template>
              确定要删除自定义特效「{{ pack.name }}」吗？
            </NPopconfirm>
          </div>
        </div>
      </div>
      <div v-if="filter === 'mine' && !visiblePacks.length" class="xb-empty" data-testid="custom-effects-empty"><span class="xb-empty-icon"><Sparkles :size="24" /></span><h3>还没有自定义特效</h3><p>可以导入 ZIP、JSON 或单文件 HTML，也可以让小伴创作。</p></div>
      <div v-if="filter === 'desk' && !visiblePacks.length" class="xb-empty" data-testid="desktop-effects-empty"><span class="xb-empty-icon"><Sparkles :size="24" /></span><h3>当前没有使用特效</h3><p>挑选喜欢的特效，点一下即可应用到桌面。</p><button type="button" class="xb-link" @click="filter = 'all'">去挑选特效</button></div>
      <p class="fx-notes">特效显示在桌面图片之上、桌面图标和其他窗口之下，鼠标可以穿过它。{{ store.settings.paused ? '桌面动画目前已暂停。' : '' }}{{ store.game.active ? '游戏空间打开期间特效会先隐藏，退出后自动回来。' : '' }}</p>
    </div>
    </div>
    <aside class="fx-inspector" aria-label="特效设置" data-testid="effect-inspector">
      <div class="fx-preview" :class="{ none: !chosen }">
        <EffectPreview v-if="chosen" :pack="chosen" :playing="true" :paused="store.settings.paused" :fps="store.settings.fps" :tuning="aquariumSelected ? tuning : undefined" />
        <Ban v-else :size="30" :stroke-width="1.4" />
      </div>
      <div class="fx-title"><b>{{ chosen ? chosen.name : '无特效' }}</b><span>{{ chosen ? chosen.description : '桌面保持安静，不叠加任何特效。' }}</span></div>
      <label class="kv" v-if="store.desktop"><span>显示器</span>
        <select class="fx-display" :value="choice.monitor || ''" data-testid="effect-monitor" @change="apply({ ...choice, monitor: ($event.target as HTMLSelectElement).value })">
          <option value="">主显示器</option><option value="*">全部显示器</option>
          <option v-if="missingMonitor" :value="choice.monitor">原显示器已断开（暂用主屏）</option>
          <option v-for="monitor in monitors" :key="monitor.id" :value="monitor.id">{{ monitor.name }}{{ monitor.primary ? '（主屏）' : '' }}</option>
        </select>
      </label>
      <section v-if="chosen && !htmlSelected" class="fx-tuning" data-testid="effect-tuning">
        <label v-if="aquariumSelected" class="kv"><span>鱼类组合</span>
          <select class="fx-display" :value="aquarium.mode" data-testid="aquarium-mode" @change="changeAquariumMode(($event.target as HTMLSelectElement).value as AquariumMode)">
            <option v-for="mode in aquariumModes" :key="mode.value" :value="mode.value">{{ mode.label }}</option>
          </select>
        </label>
        <div v-else class="kv"><div class="h"><span>数量</span><span data-testid="effect-density-value">×{{ density.toFixed(1) }}</span></div><NSlider v-model:value="density" :min="limits.density[0]" :max="limits.density[1]" :step="0.1" :tooltip="false" @update:value="tune" /></div>
        <div v-if="aquariumSelected" class="kv row">
          <span class="grow">砂石水草</span>
          <NSwitch :value="aquarium.aquascape" aria-label="砂石水草" data-testid="aquarium-aquascape" @update:value="changeAquascape" />
        </div>
        <div class="kv"><div class="h"><span>速度</span><span data-testid="effect-speed-value">×{{ speed }}</span></div><NSlider v-model:value="speed" :min="limits.speed[0]" :max="limits.speed[1]" :step="0.25" :tooltip="false" @update:value="tune" /></div>
        <div class="kv"><div class="h"><span>大小</span><span data-testid="effect-size-value">×{{ size.toFixed(1) }}</span></div><NSlider v-model:value="size" :min="limits.size[0]" :max="limits.size[1]" :step="0.1" :tooltip="false" @update:value="tune" /></div>
        <details v-if="aquariumSelected" class="fx-lighting">
          <summary>灯光</summary>
          <div v-for="light in aquariumLightControls" :key="light.key" class="kv" :data-testid="`aquarium-light-${light.key}`">
            <div class="h"><span>{{ light.label }}</span><span>{{ lightValue(light.key, aquarium.lighting[light.key]) }}</span></div>
            <NSlider :value="aquarium.lighting[light.key]" :min="light.min" :max="light.max" :step="light.step" :aria-label="light.label" :tooltip="false" @update:value="value => changeLight(light.key, value)" />
          </div>
          <button type="button" class="xb-link fx-light-reset" data-testid="aquarium-light-reset" @click="resetLights">恢复默认灯光</button>
        </details>
      </section>
      <div class="kv row"><div class="grow"><span>伙伴跟着做表情</span><small>只换表情，不弹窗、不出声</small></div><NSwitch :value="store.settings.companionLinks" data-testid="effect-companion-links" @update:value="setLinks" /></div>
      <button v-if="lockScreen.picture.mode === 'lockScreen' && canRenderEffectPicture" type="button" class="xb-btn" :disabled="lockScreen.picture.busy" data-testid="effect-lock-screen-picture" @click="prepareLockScreen">将当前效果设为锁屏图片</button>
      <LockScreenPicture :frame="lockScreen.picture.frame" :busy="lockScreen.picture.busy" :manual="!!lockScreen.picture.manualId" @apply="lockScreen.apply" @save="lockScreen.save" @close="lockScreen.close" />
      <div v-if="!htmlSelected" class="fx-buttons"><button type="button" class="xb-btn" :disabled="!chosen" data-testid="effect-reset" @click="restoreDefaults">恢复默认</button><button v-if="!isArowana(chosen)" type="button" class="xb-btn" :disabled="!store.desktop" data-testid="effect-derive" @click="createFrom">基于它创作</button></div>
    </aside>
  </section>
</template>
<style scoped>
.effects-page{height:100%;min-height:0;min-width:0;display:flex;margin-top:18px;overflow:hidden}
.fx-browser{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column}
.fx-main{flex:1;min-width:0;min-height:0;display:flex;flex-direction:column;gap:12px;padding:0 24px 28px 28px;overflow-y:auto;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.fx-main>*{flex-shrink:0}
.fx-toolbar{flex:none;display:flex;align-items:center;flex-wrap:wrap;gap:8px;padding:0 24px 12px 28px;background:var(--ui-surface)}
.fx-toolbar .cnt{font-weight:600;color:var(--ui-ink)}
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
.fx-display{width:100%;height:34px;border:1px solid var(--ui-border);border-radius:8px;background:var(--ui-control);color:var(--ui-text-primary);padding:0 8px}
.fx-lighting{margin-top:16px;border-top:1px solid var(--ui-border);border-bottom:1px solid var(--ui-border);padding:10px 0;font-size:13px}
.fx-lighting summary{cursor:pointer;color:var(--ui-text-primary)}
.fx-light-reset{margin-top:10px}
.fx-buttons{margin-top:auto;padding-top:20px;display:flex;gap:8px}
.fx-buttons .xb-btn{flex:1}
@media(max-width:1000px){.effect-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
</style>

