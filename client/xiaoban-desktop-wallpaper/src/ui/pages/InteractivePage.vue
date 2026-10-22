<script setup lang="ts">
import { ref } from 'vue';
import { ArrowUpRight, Check, Monitor, Sparkles } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';
import { wallpaperDefaults } from '../../bridge.ts';
import { openRitual } from '../../rituals/journal.ts';
import FortuneVessel from '../../rituals/FortuneVessel.vue';
import PondScene from '../../rituals/PondScene.vue';
const props = defineProps<{ store: ControlStore }>();
const busy = ref('');
async function enter(scene: 'fortune' | 'pond') {
  busy.value = scene;
  try { await openRitual(scene); } catch (error) { props.store.notify(error, 'error'); }
  finally { busy.value = ''; }
}
async function applyPond() {
  busy.value = 'builtin-wish-pond';
  try {
    await props.store.applyWallpaper('builtin-wish-pond', { ...wallpaperDefaults }, false);
    props.store.notify('已设为桌面壁纸，可随时进入互动放一盏灯', 'success');
  } catch (error) { props.store.notify(error, 'error'); }
  finally { busy.value = ''; }
}
</script>
<template>
  <section class="ui-page interactive-page" data-testid="interactive-page">
    <header class="xb-page-head"><h1 class="xb-page-title">互动壁纸</h1><span class="collection-label"><Sparkles :size="14" /> 心愿集 · 壹</span></header>
    <div class="ritual-intro"><span>一方风景，一点心意</span><p>求一支好签，放一盏心灯。把愿望轻轻交给此刻。</p></div>
    <div class="ritual-grid">
      <article class="ritual-card fortune-card" data-testid="ritual-fortune">
        <div class="ritual-art paper-art"><span class="art-number">01 / GOOD WISHES</span><span class="vertical-poem">春风自有花期<br>心事终有回响</span><div class="paper-orbit"></div><FortuneVessel class="card-vessel"/><span class="art-seal">宜<br>祈愿</span></div>
        <div class="ritual-copy"><div class="ritual-kind">互动仪式 <span>约十秒 · 可收藏</span></div><h2>一签好意</h2><p>轻摇签筒，让一份温柔的祝福落在今天。</p><div class="ritual-actions"><button class="ritual-enter red" :disabled="!!busy || store.game.active" data-testid="enter-fortune" @click="enter('fortune')">{{ busy === 'fortune' ? '正在打开…' : '求一支好签' }}<ArrowUpRight :size="16"/></button><span>十二份好意，随心遇见</span></div></div>
      </article>
      <article class="ritual-card pond-card" data-testid="ritual-pond">
        <div class="ritual-art pond-art"><PondScene :paused="store.settings.paused" :fps="store.settings.fps"/><span class="art-number">02 / LANTERN WISHES</span><span class="pond-poem">一灯一愿<br><small>愿心事有归处</small></span><span v-if="store.settings.wallpaperId === 'builtin-wish-pond'" class="applied"><Check :size="12"/>正在桌面播放</span></div>
        <div class="ritual-copy"><div class="ritual-kind">互动场景 <span>可放灯 · 可设为壁纸</span></div><h2>一池心灯</h2><p>写下心愿，放一盏莲灯，看灯火与水波相逢。</p><div class="ritual-actions"><button class="ritual-enter jade" :disabled="!!busy || store.game.active" data-testid="enter-pond" @click="enter('pond')">{{ busy === 'pond' ? '正在打开…' : '进入灯池' }}<ArrowUpRight :size="16"/></button><button class="wallpaper-action" :disabled="!!busy" data-testid="apply-pond" @click="applyPond"><Monitor :size="14"/>{{ busy === 'builtin-wish-pond' ? '正在应用…' : '设为壁纸' }}</button></div></div>
      </article>
    </div>
    <p class="ritual-footnote">{{ store.game.active ? '先退出游戏空间，就可以进入互动。' : '主动进入时开启互动，退出即回到原来的桌面。灯池设为壁纸后，水景会安静播放。' }}</p>
  </section>
</template>
<style scoped>
.interactive-page{display:grid;gap:26px;max-width:1200px;margin:0 auto}.collection-label{margin-left:auto;display:flex;align-items:center;gap:7px;color:var(--ui-text-secondary);font-size:12px;letter-spacing:2px;white-space:nowrap}.ritual-intro{border-left:2px solid #bca57b;padding:3px 0 3px 16px}.ritual-intro>span{font-family:'KaiTi','STKaiti',serif;font-size:20px;color:var(--ui-text-primary);letter-spacing:2px}.ritual-intro p{font-size:12px;color:var(--ui-text-secondary);margin:5px 0 0}.ritual-grid{display:grid;grid-template-columns:1fr 1fr;gap:24px}.ritual-card{overflow:hidden;border-radius:16px;background:var(--ui-surface);box-shadow:0 0 0 1px var(--ui-line),var(--ui-shadow-card);transition:box-shadow .3s,transform .3s}.ritual-card:hover{transform:translateY(-3px);box-shadow:0 14px 36px #192a331a}.ritual-art{position:relative;height:290px;overflow:hidden}.paper-art{background:radial-gradient(ellipse at 50% 55%,#fff7e4 0,#efe4ce 62%,#e3d4bb 100%);color:#8a6550}.paper-art:after{content:'';position:absolute;inset:0;pointer-events:none;opacity:.18;background-image:url("data:image/svg+xml,%3Csvg viewBox='0 0 180 180' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='.85' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Cpath fill='%23000' opacity='.25' filter='url(%23n)' d='M0 0h180v180H0z'/%3E%3C/svg%3E")}.art-number{position:absolute;top:23px;left:24px;font-size:9px;letter-spacing:2px;z-index:1;opacity:.7}.vertical-poem{position:absolute;top:76px;right:27px;writing-mode:vertical-rl;line-height:2;letter-spacing:4px;font-size:13px;font-family:'KaiTi','STKaiti',serif}.card-vessel{position:absolute!important;left:50%;bottom:-10px;width:258px!important;height:286px!important;transform:translateX(-50%)}.paper-orbit{position:absolute;width:235px;height:235px;border:1px solid #b08d6033;border-radius:50%;top:40px;left:50%;transform:translateX(-50%)}.paper-orbit:after{content:'';position:absolute;inset:10px;border:1px solid #b08d601c;border-radius:50%}.art-seal{position:absolute;left:26px;bottom:25px;border:1px solid #9b5145;color:#9b5145;padding:4px;font:11px/1.2 'KaiTi','STKaiti',serif;letter-spacing:2px}.pond-art{color:#e2d9b9;background:#082329}.pond-poem{position:absolute;left:28px;top:74px;font:28px/1.7 'KaiTi','STKaiti',serif;letter-spacing:5px;pointer-events:none}.pond-poem small{font-size:12px;letter-spacing:3px;opacity:.7}.applied{position:absolute;right:16px;bottom:16px;background:#092a2d9e;border:1px solid #dcc59040;border-radius:30px;font-size:10px;display:flex;gap:5px;align-items:center;padding:5px 9px}.ritual-copy{padding:24px 25px 25px}.ritual-kind{display:flex;justify-content:space-between;color:var(--ui-text-tertiary);font-size:11px;letter-spacing:1px}.ritual-kind>span{letter-spacing:0}.ritual-copy h2{font:26px 'KaiTi','STKaiti',serif;letter-spacing:3px;color:var(--ui-text-primary);margin:14px 0 8px}.ritual-copy p{font-size:13px;line-height:1.7;color:var(--ui-text-secondary);margin:0}.ritual-actions{margin-top:22px;display:flex;align-items:center;justify-content:space-between;gap:10px}.ritual-actions>span{font-size:11px;color:var(--ui-text-tertiary)}.ritual-enter{display:flex;gap:22px;align-items:center;justify-content:space-between;border:0;border-radius:8px;padding:11px 15px;color:#fff6e6;font-family:inherit;font-size:13px;font-weight:500;cursor:pointer;box-shadow:0 4px 10px #00000010}.ritual-enter.red{background:linear-gradient(110deg,#9d4943,#803431)}.ritual-enter.jade{background:linear-gradient(110deg,#2c5550,#193c3a)}.wallpaper-action{display:flex;align-items:center;gap:6px;background:none;border:0;color:var(--ui-text-secondary);font-size:12px;cursor:pointer;padding:8px 0}.ritual-footnote{margin:0;color:var(--ui-text-tertiary);font-size:12px;line-height:1.8}button:disabled{opacity:.45;cursor:default}button:focus-visible{outline:2px solid #bf995c;outline-offset:4px}@media(max-width:850px){.ritual-grid{grid-template-columns:1fr}.ritual-art{height:280px}}@media(prefers-reduced-motion:reduce){.ritual-card{transition:none}.ritual-card:hover{transform:none}}
</style>
