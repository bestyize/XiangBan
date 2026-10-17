<script setup lang="ts">
import ResourceShareActions from "../components/ResourceShareActions.vue";
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { NAlert, NButton, NPopconfirm, NSlider } from 'naive-ui';
import { Ban, Check, FileUp, Sparkles, Trash2 } from 'lucide-vue-next';
import { normalizeChoice, paramsOf } from '../../effects/parse.ts';
import { limits, type EffectsChoice } from '../../effects/types.ts';
import type { ControlStore } from '../control-types.ts';
import EffectPreview from '../components/EffectPreview.vue';
const props = defineProps<{ store: ControlStore }>();

const choice = computed(() => normalizeChoice(props.store.settings.effects));
const packs = computed(() => props.store.effectList.packs);
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
function tune() { if (timer) clearTimeout(timer); timer = setTimeout(commit, 150); }
async function reload(announce: boolean) {
  try { await props.store.loadEffects(); if (announce) props.store.notify('已重新载入特效', 'success'); }
  catch (error) { props.store.notify(message(error), 'error'); }
}
async function importPack() {
  try {
    const outcome = await props.store.importEffectPack();
    if (outcome.status === 'imported') props.store.notify(`已导入特效「${outcome.name}」，可以选用了`, 'success');
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
onBeforeUnmount(() => { if (timer) { clearTimeout(timer); commit(); } });
</script>
<template>
  <section class="ui-page effects-page" data-testid="effects-page">
    <header class="page-header">
      <h1>桌面特效</h1>
      <div class="header-actions"><NButton round data-testid="effects-import" @click="importPack"><template #icon><FileUp :size="15" /></template>导入特效</NButton><NButton type="primary" round :disabled="!store.desktop" data-testid="effects-create" @click="store.startCreation('effect')"><template #icon><Sparkles :size="15" /></template>创作特效</NButton></div>
    </header>
    <div class="effects-body">
    <NAlert v-if="missing" type="warning" :show-icon="false" data-testid="effect-missing">所选的特效「{{ choice.selected }}」已经找不到（可能被移出了特效文件夹），桌面上暂时没有特效。重新选一个即可。</NAlert>
    <NAlert v-if="store.effectList.errors.length" type="warning" :show-icon="false" data-testid="effect-errors">有 {{ store.effectList.errors.length }} 个自定义特效无法使用：{{ store.effectList.errors.join('；') }}</NAlert>
    <div class="effect-grid" role="group" aria-label="特效">
      <button class="effect-card none" :class="{ on: !chosen }" :aria-pressed="!chosen" data-testid="effect-none" @click="choose('')">
        <span class="effect-stage"><Ban :size="30" :stroke-width="1.5" /></span>
        <span class="effect-name"><b>无特效</b><small>桌面保持安静</small></span>
        <Check v-if="!chosen" :size="16" class="effect-check" />
      </button>
      <div v-for="pack in packs" :key="pack.id" class="effect-card-wrapper" :class="{ user: pack.source === 'user' }">
        <button class="effect-card" :class="{ on: chosen?.id === pack.id }" :aria-pressed="chosen?.id === pack.id" :data-testid="`effect-${pack.id}`" @click="choose(pack.id)" @mouseenter="hoveredId = pack.id" @mouseleave="hoveredId = ''">
          <span class="effect-stage"><EffectPreview :pack="pack" :playing="playingId === pack.id" :paused="store.settings.paused" :fps="store.settings.fps" /></span>
          <span class="effect-name"><b>{{ pack.name }}<i v-if="pack.source === 'user'" class="mine">我的</i></b><small>{{ pack.description }}</small></span>
          <Check v-if="chosen?.id === pack.id" :size="16" class="effect-check" />
        </button>
        <ResourceShareActions v-if="pack.source === 'user'" icons :store="store" kind="effect" :id="pack.id"/>
        <NPopconfirm v-if="pack.source === 'user'" positive-text="删除" negative-text="取消" @positive-click="removeEffect(pack.id, pack.name)">
          <template #trigger>
            <button class="effect-delete-btn" title="删除这个自定义特效" :aria-label="`删除特效 ${pack.name}`" :data-testid="`delete-effect-${pack.id}`" @click.stop>
              <Trash2 :size="13" />
            </button>
          </template>
          确定要删除自定义特效「{{ pack.name }}」吗？
        </NPopconfirm>
      </div>
    </div>
    <section v-if="chosen" class="surface tuning" data-testid="effect-tuning">
      <div class="section-heading"><div><h2>{{ chosen.name }}</h2><p>调整后立即生效，每种特效各自记住自己的设置</p></div></div>
      <div class="field"><label>密度 <span data-testid="effect-density-value">×{{ density.toFixed(1) }}</span></label><NSlider v-model:value="density" :min="limits.density[0]" :max="limits.density[1]" :step="0.1" @update:value="tune" /></div>
      <div class="field"><label>速度 <span data-testid="effect-speed-value">×{{ speed }}</span></label><NSlider v-model:value="speed" :min="limits.speed[0]" :max="limits.speed[1]" :step="0.25" @update:value="tune" /></div>
      <div class="field"><label>大小 <span data-testid="effect-size-value">×{{ size.toFixed(1) }}</span></label><NSlider v-model:value="size" :min="limits.size[0]" :max="limits.size[1]" :step="0.1" @update:value="tune" /></div>
    </section>
    <p class="muted notes">特效显示在桌面图片之上、桌面图标和其他窗口之下，鼠标可以穿过它。想要自己的特效？点“导入特效”选一个特效包文件（JSON），也可以直接在聊天里让小伴为你写一个。{{ store.settings.paused ? '桌面动画目前已暂停，特效会静止，可在“设置”里恢复。' : '' }}{{ store.game.active ? '游戏空间打开期间特效会先隐藏，退出后自动回来。' : '' }}</p>
    </div>
  </section>
</template>
<style scoped>
.effects-page{display:flex;flex-direction:column;flex:1;min-height:0;width:100%}
.effects-page>.page-header{flex:none;background:var(--ui-app-bg);padding:var(--ui-page-top) var(--ui-page-inset) 16px;margin-bottom:0;border-bottom:1px solid var(--ui-border)}
.effects-body{flex:1;min-height:0;overflow-y:auto;padding:var(--ui-section-gap) var(--ui-page-inset) 32px;display:grid;gap:var(--ui-section-gap);align-content:start}
.header-actions{display:flex;gap:10px;flex:none;flex-wrap:wrap;justify-content:flex-end}
.effect-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(210px,1fr));gap:var(--ui-card-gap)}
.effect-card-wrapper{position:relative}
.effect-card-wrapper .effect-card{width:100%}
.effect-card-wrapper .resource-share-actions.icons{position:absolute;right:12px;bottom:12px;z-index:2}
.effect-card-wrapper.user .effect-name{padding-right:82px}
.effect-delete-btn{position:absolute;top:8px;right:8px;z-index:3;width:26px;height:26px;border-radius:50%;border:0;background:rgba(0,0,0,0.5);color:#fff;display:grid;place-items:center;cursor:pointer;opacity:0.7;transition:all .2s}
.effect-delete-btn:hover{opacity:1;background:#dc2626}
.effect-card{position:relative;display:grid;grid-template-rows:auto auto;padding:0;border:0;background:var(--ui-surface);border-radius:var(--ui-radius-card);overflow:hidden;cursor:pointer;text-align:left;color:var(--ui-text-primary);transition:transform 200ms cubic-bezier(.2,.8,.2,1),box-shadow 200ms}
.effect-card:hover{transform:translateY(-3px);box-shadow:0 14px 30px var(--ui-shadow-hover)}
.effect-card.on{outline:3px solid var(--ui-primary);outline-offset:2px}
.effect-stage{display:block;aspect-ratio:16/9;overflow:hidden;position:relative}
.none .effect-stage{display:grid;place-items:center;background:var(--ui-surface-alt);color:var(--ui-text-tertiary)}
.effect-name{display:grid;gap:2px;padding:12px 16px 14px}
.effect-name b{font-size:15px;font-weight:650}
.effect-name small{font-size:12px;color:var(--ui-text-secondary)}
.mine{font-style:normal;font-size:11px;font-weight:600;margin-left:8px;padding:1px 7px;border-radius:999px;background:var(--ui-primary-soft);color:var(--ui-primary-text)}
.effect-check{position:absolute;top:10px;left:10px;width:26px;height:26px;padding:5px;border-radius:50%;background:var(--ui-primary);color:#fff}
.tuning{padding:24px;max-width:640px}
.tuning .section-heading{margin-bottom:12px}
.field{margin:16px 0 4px}
.field>label{display:flex;justify-content:space-between;font-size:13px;font-weight:600;margin-bottom:10px}
.notes{font-size:13px;line-height:1.7;margin:0}
</style>


