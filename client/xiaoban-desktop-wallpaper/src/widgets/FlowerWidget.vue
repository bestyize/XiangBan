<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Droplets } from 'lucide-vue-next';
import { FLOWER_STAGES, FLOWER_MAX_WATER, emptyFlower, flowerDays, flowerProgress, flowerStage, waterUntilNext, validateFlower } from './flower.ts';
import type { FlowerData, WidgetContent } from './model.ts';
import stage1 from './assets/flower/stage-1.webp';
import stage2 from './assets/flower/stage-2.webp';
import stage3 from './assets/flower/stage-3.webp';
import stage4 from './assets/flower/stage-4.webp';
import stage5 from './assets/flower/stage-5.webp';
import stage6 from './assets/flower/stage-6.webp';

const props = defineProps<{ flower?: FlowerData; preview?: boolean; editing?: boolean; save?: (content: WidgetContent) => Promise<void> }>();
const STAGE_IMAGES = [stage1, stage2, stage3, stage4, stage5, stage6];

const water = ref(props.flower?.water ?? 0);
const since = ref(props.flower?.since ?? emptyFlower().since);
const dirty = ref(false), saving = ref(false), error = ref('');
const drops = ref(false), dropKey = ref(0), plusOne = ref(0), flash = ref(''), bloom = ref(false);
const stage = computed(() => flowerStage(water.value));
const progress = computed(() => flowerProgress(water.value));
const remaining = computed(() => waterUntilNext(water.value));
const days = computed(() => flowerDays(since.value));
const locked = computed(() => !!props.preview || !!props.editing);

watch(() => props.flower, value => {
  if (!value || saving.value || dirty.value) return;
  water.value = value.water;
  since.value = value.since;
});

let dropTimer: ReturnType<typeof setTimeout> | undefined, flashTimer: ReturnType<typeof setTimeout> | undefined, bloomTimer: ReturnType<typeof setTimeout> | undefined;
function animate(kind: 'water' | 'stage') {
  clearTimeout(dropTimer);
  // A fresh key remounts the drops, so every watering restarts the fall even mid-animation.
  dropKey.value += 1;
  drops.value = true;
  dropTimer = setTimeout(() => { drops.value = false; }, 900);
  if (kind === 'stage') {
    clearTimeout(flashTimer);
    clearTimeout(bloomTimer);
    bloom.value = true;
    flash.value = stage.value === FLOWER_STAGES.length - 1 ? '满庭花开' : '又长高了一些';
    flashTimer = setTimeout(() => { flash.value = ''; }, 1800);
    bloomTimer = setTimeout(() => { bloom.value = false; }, 1200);
  }
}

async function commit() {
  if (saving.value) return;
  saving.value = true;
  error.value = '';
  try {
    while (dirty.value) {
      dirty.value = false;
      validateFlower({ water: water.value, since: since.value });
      await props.save?.({ flower: { water: water.value, since: since.value } });
    }
  } catch (reason) {
    dirty.value = true;
    error.value = String(reason);
  } finally {
    saving.value = false;
  }
}

function tap() {
  if (locked.value || water.value >= 999) return;
  const before = stage.value;
  water.value += 1;
  plusOne.value = (plusOne.value + 1) % 1000;
  dirty.value = true;
  animate(stage.value > before ? 'stage' : 'water');
  void commit();
}
</script>
<template>
  <div class="flower-content" :class="{ preview: locked }">
    <div class="flower-status">
      <div><span>第 {{ stage + 1 }} 阶</span><b>{{ FLOWER_STAGES[stage].label }}</b></div>
      <span class="flower-days">已相伴 {{ days }} 天</span>
    </div>
    <div class="flower-scene">
      <span class="flower-halo" aria-hidden="true"></span>
      <div class="flower-art" :class="{ sway: !locked }" aria-hidden="true">
        <img v-for="(src, index) in STAGE_IMAGES" :key="src" :src="src" :class="{ on: index === stage }" alt="" draggable="false" />
      </div>
      <div v-if="drops" :key="dropKey" class="flower-rain" aria-hidden="true"><i v-for="n in 7" :key="n"></i><span class="flower-splash"></span></div>
      <span class="flower-plus" aria-hidden="true" :key="plusOne">+1</span>
      <transition name="flower-rise"><span v-if="flash" class="flower-flash" role="status">{{ flash }}</span></transition>
      <span v-if="bloom" class="flower-bloom" aria-hidden="true"><i v-for="n in 6" :key="n"></i></span>
    </div>
    <div class="flower-track" :aria-label="`已浇水 ${water} 次，${remaining ? `再浇 ${remaining} 次进入下一阶` : '已经满庭花开'}`">
      <div class="flower-bar" aria-hidden="true"><span :style="{ width: `${progress * 100}%` }"></span><i v-for="(item, index) in FLOWER_STAGES" :key="item.id" :class="{ on: index <= stage }" :style="{ left: `${(index / (FLOWER_STAGES.length - 1)) * 100}%` }"></i></div>
      <p>{{ remaining ? `再浇 ${remaining} 次，就进入「${FLOWER_STAGES[stage + 1].label}」` : '满庭花开，可以一直陪着你了' }}</p>
    </div>
    <div class="flower-actions">
      <button type="button" class="flower-water" :disabled="locked || water >= 999" :aria-label="`给小白花浇水，当前 ${water} 次`" @click="tap"><Droplets :size="14" /><span>浇水</span><small>{{ water }}</small></button>
      <span class="flower-note">{{ water >= FLOWER_MAX_WATER ? '开到最盛' : FLOWER_STAGES[stage].note }}</span>
    </div>
    <p v-if="error" class="flower-error" role="alert" @click="error = ''">{{ error }}，点此重试</p>
  </div>
</template>
<style scoped>
.flower-content{position:relative;display:flex;flex-direction:column;height:100%;min-height:0;padding-top:4px;cursor:default;user-select:none}.flower-status{display:flex;align-items:baseline;justify-content:space-between;gap:8px}.flower-status>div{display:flex;align-items:baseline;gap:7px;min-width:0}.flower-status>div>span{font-size:10px;letter-spacing:1px;opacity:.55;white-space:nowrap}.flower-status b{font:18px/1.2 'KaiTi','STKaiti','Songti SC',serif;letter-spacing:2px;font-weight:500}.flower-days{font-size:10px;opacity:.5;white-space:nowrap}
.flower-content.preview .flower-water{opacity:.45}
.flower-scene{position:relative;flex:1;min-height:0;display:flex;align-items:flex-end;justify-content:center;overflow:visible}.flower-halo{position:absolute;left:50%;bottom:8%;width:74%;aspect-ratio:1;transform:translateX(-50%);border-radius:50%;background:radial-gradient(circle,rgba(255,246,196,.3),rgba(214,238,178,.12) 55%,transparent 72%);filter:blur(2px);pointer-events:none}.flower-art{position:relative;width:min(100%,232px);height:100%;transform-origin:50% 92%;will-change:transform}.flower-art.sway{animation:flower-sway 6.5s ease-in-out infinite alternate}
.flower-scene:hover .flower-art.sway{animation-play-state:paused}.flower-art img{position:absolute;inset:0;width:100%;height:100%;object-fit:contain;object-position:center bottom;opacity:0;transform:translateY(12px) scale(.9);transform-origin:50% 100%;transition:opacity .55s ease,transform .85s cubic-bezier(.22,.9,.3,1.15);filter:drop-shadow(0 6px 8px rgba(30,52,30,.16));pointer-events:none}.flower-art img.on{opacity:1;transform:none}
@keyframes flower-sway{from{transform:rotate(-1.1deg)}to{transform:rotate(1.1deg)}}
.flower-rain{position:absolute;inset:0;pointer-events:none;overflow:visible}.flower-rain i{position:absolute;top:-6%;width:4px;height:9px;border-radius:60% 60% 50% 50%;background:linear-gradient(#dff4ff,#9fd6f2);opacity:0;animation:flower-drop .85s cubic-bezier(.4,.1,.7,1) forwards}.flower-rain i:nth-child(1){left:44%;animation-delay:0s}.flower-rain i:nth-child(2){left:49%;animation-delay:.07s;height:7px}.flower-rain i:nth-child(3){left:54%;animation-delay:.03s}.flower-rain i:nth-child(4){left:39%;animation-delay:.16s;height:6px}.flower-rain i:nth-child(5){left:58%;animation-delay:.13s;height:8px}.flower-rain i:nth-child(6){left:47%;animation-delay:.2s;height:6px}.flower-rain i:nth-child(7){left:52%;animation-delay:.24s;height:8px}.flower-splash{position:absolute;left:50%;bottom:1%;width:64px;height:12px;transform:translateX(-50%);border:1px solid rgba(255,255,255,.55);border-radius:50%;opacity:0;animation:flower-splash .8s ease-out .18s forwards}
@keyframes flower-drop{0%{opacity:0;transform:translateY(0)}18%{opacity:.95}80%{opacity:.85}100%{opacity:0;transform:translateY(138px) scale(.7)}}@keyframes flower-splash{0%{opacity:0;transform:translateX(-50%) scale(.45)}40%{opacity:.6}100%{opacity:0;transform:translateX(-50%) scale(1.15)}}
.flower-plus{position:absolute;right:22%;bottom:33%;font-size:11px;color:#7fc9e8;text-shadow:0 1px 4px rgba(0,40,60,.35);animation:flower-plus 1s ease-out forwards;pointer-events:none}
@keyframes flower-plus{0%{opacity:0;transform:translateY(4px)}25%{opacity:1}100%{opacity:0;transform:translateY(-16px)}}
.flower-bloom{position:absolute;inset:0;pointer-events:none}.flower-bloom i{position:absolute;left:50%;top:46%;width:5px;height:5px;border-radius:50%;background:#fdf3bd;box-shadow:0 0 6px 2px rgba(253,243,189,.55);opacity:0;animation:flower-spark 1.1s ease-out forwards}.flower-bloom i:nth-child(1){--sx:-46px;--sy:-34px;animation-delay:.02s}.flower-bloom i:nth-child(2){--sx:38px;--sy:-42px;animation-delay:.08s}.flower-bloom i:nth-child(3){--sx:-24px;--sy:-58px;animation-delay:.14s}.flower-bloom i:nth-child(4){--sx:52px;--sy:-10px;animation-delay:.05s}.flower-bloom i:nth-child(5){--sx:-58px;--sy:-4px;animation-delay:.11s}.flower-bloom i:nth-child(6){--sx:12px;--sy:-64px;animation-delay:.18s}
@keyframes flower-spark{0%{opacity:0;transform:translate(-50%,0) scale(.4)}25%{opacity:.95}100%{opacity:0;transform:translate(calc(-50% + var(--sx)),var(--sy)) scale(.85)}}
.flower-flash{position:absolute;right:6%;top:44%;font:13px/1 'KaiTi','STKaiti',serif;letter-spacing:2px;color:#f6efd2;background:rgba(28,58,46,.62);border:1px solid rgba(246,239,210,.28);padding:5px 9px;border-radius:20px;backdrop-filter:blur(4px);pointer-events:none;white-space:nowrap}
.flower-rise-enter-active,.flower-rise-leave-active{transition:opacity .5s ease,transform .5s ease}.flower-rise-enter-from,.flower-rise-leave-to{opacity:0;transform:translateY(5px)}
.flower-track{display:grid;gap:7px;padding:2px 0 0}.flower-bar{position:relative;height:4px;margin:0 4px;border-radius:3px;background:color-mix(in srgb,currentColor 16%,transparent)}.flower-bar>span{position:absolute;left:0;top:0;bottom:0;border-radius:3px;background:linear-gradient(90deg,#9fd083,#f1e19b);transition:width .8s cubic-bezier(.3,.8,.3,1)}.flower-bar i{position:absolute;top:50%;width:6px;height:6px;margin-left:-3px;border-radius:50%;transform:translateY(-50%);background:color-mix(in srgb,currentColor 34%,transparent);transition:opacity .4s,box-shadow .4s}.flower-bar i.on{background:color-mix(in srgb,currentColor 78%,transparent);box-shadow:0 0 0 3px rgba(241,225,155,.16)}.flower-track p{margin:0;font-size:10px;line-height:1.5;opacity:.62;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.flower-actions{display:flex;align-items:center;justify-content:space-between;gap:10px;padding-top:9px}.flower-water{display:flex;align-items:center;gap:6px;border:1px solid rgba(160,214,235,.5);background:linear-gradient(135deg,rgba(116,190,220,.28),rgba(74,142,170,.18));color:inherit;border-radius:20px;padding:6px 12px 6px 10px;font:11px 'Microsoft YaHei','PingFang SC',sans-serif;letter-spacing:1px;cursor:pointer;transition:border-color .2s,transform .15s}.flower-water:hover:not(:disabled){border-color:rgba(190,232,247,.8)}.flower-water:active:not(:disabled){transform:scale(.96)}.flower-water small{font-size:9px;opacity:.6;letter-spacing:0}.flower-water:disabled{opacity:.5;cursor:default}.flower-note{font-size:10px;opacity:.5;line-height:1.5;text-align:right;overflow:hidden;display:-webkit-box;-webkit-line-clamp:1;-webkit-box-orient:vertical}.flower-error{position:absolute;left:0;right:0;bottom:-2px;margin:0;padding:5px 8px;border-radius:8px;background:rgba(38,14,12,.86);font-size:10px;line-height:1.5;color:#ffd8c9;cursor:pointer;z-index:3;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}.flower-content.preview .flower-water{cursor:default}
@media(prefers-reduced-motion:reduce){.flower-art.sway{animation:none}.flower-art img{transition:none}.flower-rain i,.flower-splash,.flower-plus,.flower-flash,.flower-bloom i{animation:none}.flower-rain,.flower-flash,.flower-bloom{display:none}}
</style>
