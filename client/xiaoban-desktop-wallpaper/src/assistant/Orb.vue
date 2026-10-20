<script setup lang="ts">
import type { Phase } from './assistant-store.ts';

// Keep the shared orb API; CSS animation replaces continuous canvas redraws.
withDefaults(defineProps<{ phase: Phase; level?: number; active?: boolean; flowing?: boolean; size?: number; dark?: boolean; fps?: number }>(), { level: 0, active: true, flowing: false, size: 34, dark: true, fps: 60 });
</script>

<template>
  <span class="orb-frame" :class="{ paused: !active, flowing }" :data-phase="phase" :style="{ width: `${size}px`, height: `${size}px` }" aria-hidden="true">
    <span class="orb-body"><i v-if="flowing" class="orb-colours"></i><i class="mist"></i></span>
  </span>
</template>

<style scoped>
.orb-frame{display:grid;place-items:center;flex:none;pointer-events:none}
.orb-body{display:block;position:relative;isolation:isolate;overflow:hidden;width:85.3%;height:85.3%;border-radius:50%;background:radial-gradient(ellipse at 25% 20%,#FFE47A,transparent 56%),radial-gradient(ellipse at 78% 28%,#F26BAA,transparent 60%),radial-gradient(ellipse at 80% 85%,#55BEEB,transparent 58%),radial-gradient(ellipse at 18% 78%,#43D6A2,transparent 62%),#AF8DE2;box-shadow:inset 0 0 0 1px #ffffff40,inset 1px 2px 3px #ffffff35,inset -2px -2px 4px #17404c22;animation:breathe 5s ease-in-out infinite}
.orb-body:before{content:'';position:absolute;inset:-15%;border-radius:43%;background:radial-gradient(ellipse at 28% 25%,#FFECA47D,transparent 60%);filter:blur(2px);animation:flow 12s linear infinite}
.orb-body:after{content:'';position:absolute;inset:0;border-radius:50%;background:radial-gradient(ellipse at 35% 17%,#ffffff9c,transparent 28%),radial-gradient(ellipse at 70% 85%,#ffffff24,transparent 40%);box-shadow:inset 0 -1px 2px #ffffff35}
.mist{position:absolute;inset:12%;border-radius:45%;border:1px solid #ffffff35;filter:blur(2px);animation:flow 16s linear infinite reverse}
[data-phase=listening] .orb-body{animation:listen 1.35s ease-in-out infinite}
[data-phase=listening] .mist{opacity:.9;filter:blur(1px);animation-duration:2.8s}
[data-phase=listening] .orb-body:before{animation-duration:4s}
[data-phase=thinking] .orb-body{animation:ponder 2.5s linear infinite}
[data-phase=thinking] .orb-body:before{animation-duration:1.4s}
[data-phase=thinking] .mist{filter:blur(.5px);animation-duration:1.2s}
[data-phase=answering] .orb-body,[data-phase=speaking] .orb-body{animation:speak 3s ease-in-out infinite}
[data-phase=answering] .orb-body:before,[data-phase=speaking] .orb-body:before{animation-duration:2.3s}
[data-phase=answering] .mist,[data-phase=speaking] .mist{animation-duration:2s}
.orb-colours{position:absolute;inset:-22%;border-radius:43%;background:radial-gradient(ellipse at 25% 20%,#FFE47A,transparent 56%),radial-gradient(ellipse at 78% 28%,#F26BAA,transparent 60%),radial-gradient(ellipse at 80% 85%,#55BEEB,transparent 58%),radial-gradient(ellipse at 18% 78%,#43D6A2,transparent 62%),#AF8DE2;animation:colour-flow 12s linear infinite}.flowing .orb-body:before{opacity:.45}@keyframes colour-flow{0%{transform:rotate(0deg) scale(1.05)}50%{transform:rotate(180deg) scale(1.18)}100%{transform:rotate(360deg) scale(1.05)}}
.paused .orb-colours,.paused .orb-body,.paused .orb-body:before,.paused .mist{animation-play-state:paused}
@keyframes breathe{50%{transform:scale(.965) translateY(-1px)}}
@keyframes flow{to{transform:rotate(360deg)}}
@keyframes listen{0%,100%{transform:scale(.88)}50%{transform:scale(1.08)}}
@keyframes ponder{0%{transform:rotate(0) scale(.97)}50%{transform:rotate(180deg) scale(1.03)}100%{transform:rotate(360deg) scale(.97)}}
@keyframes speak{0%,100%{transform:scale(1,.958)}18%{transform:scale(.909,1.098)}36%{transform:scale(1.084,.916)}55%{transform:scale(.958,1.049)}72%{transform:scale(1.049,.951)}88%{transform:scale(.944,1.077)}}
@media(prefers-reduced-motion:reduce){.orb-colours,.orb-body,.orb-body:before,.mist{animation:none}}
</style>
