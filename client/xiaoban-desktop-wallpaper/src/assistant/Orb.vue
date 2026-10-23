<script setup lang="ts">
import { computed } from 'vue';
import type { Phase } from './assistant-store.ts';

// The brand mark (the Sparkles before 小伴 AI) in white on a theme-colour disc. It turns into sound bars while
// listening or reading aloud, and its small sparkles twinkle while thinking. The disc fills `size`.
const props = withDefaults(defineProps<{ phase: Phase; level?: number; active?: boolean; muted?: boolean; size?: number }>(), { level: 0, active: true, muted: false, size: 34 });
const voice = computed(() => props.phase === 'listening' || props.phase === 'speaking');
const loud = computed(() => props.phase === 'listening' ? Math.max(0, Math.min(1, props.level)) : 0.6);
</script>

<template>
  <span class="orb-frame" :class="{ paused: !active, muted }" :data-phase="phase" :style="{ width: `${size}px`, height: `${size}px`, '--loud': loud }" aria-hidden="true">
    <span v-if="voice" class="bars"><i></i><i></i><i></i><i></i></span>
    <svg v-else class="mark" viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path fill="#fff" d="M11.017 2.814a1 1 0 0 1 1.966 0l1.051 5.558a2 2 0 0 0 1.594 1.594l5.558 1.051a1 1 0 0 1 0 1.966l-5.558 1.051a2 2 0 0 0-1.594 1.594l-1.051 5.558a1 1 0 0 1-1.966 0l-1.051-5.558a2 2 0 0 0-1.594-1.594l-5.558-1.051a1 1 0 0 1 0-1.966l5.558-1.051a2 2 0 0 0 1.594-1.594z" />
      <path class="twinkle" d="M20 2v4M22 4h-4" />
      <circle class="twinkle late" cx="4" cy="20" r="2" />
    </svg>
  </span>
</template>

<style scoped>
.orb-frame{display:grid;place-items:center;flex:none;pointer-events:none;border-radius:50%;background:var(--accent,#D93A44);box-shadow:inset 0 0 0 .5px rgba(255,255,255,.18),inset 0 1px 0 rgba(255,255,255,.28),inset 0 -2px 6px rgba(0,0,0,.14);transition:background .2s}
.muted{background:#8E8E93}
.mark{display:block;width:50%;height:50%;filter:drop-shadow(0 1px 1px rgba(0,0,0,.18))}
[data-phase=idle]{animation:breathe 4s ease-in-out infinite}
[data-phase=thinking] .mark,[data-phase=answering] .mark{animation:sway 2.4s ease-in-out infinite}
[data-phase=thinking] .twinkle,[data-phase=answering] .twinkle{animation:twinkle 1.2s ease-in-out infinite}
.twinkle.late{animation-delay:.4s!important}
.bars{display:flex;align-items:center;gap:8%;height:46%;transform:scaleY(calc(.55 + var(--loud) * .45));transition:transform .12s}
.bars i{display:block;width:3px;height:55%;border-radius:2px;background:#fff;animation:bar 1s ease-in-out infinite}
.bars i:nth-child(2){height:80%;animation-delay:.15s}.bars i:nth-child(3){height:100%;animation-delay:.3s}.bars i:nth-child(4){height:62%;animation-delay:.45s}
[data-phase=speaking] .bars i{animation-duration:1.4s}
.paused,.paused *{animation-play-state:paused!important}
@keyframes breathe{50%{transform:scale(1.035)}}
@keyframes sway{50%{transform:rotate(12deg)}}
@keyframes twinkle{50%{opacity:.15}}
@keyframes bar{0%,100%{transform:scaleY(.45)}50%{transform:scaleY(1)}}
@media(prefers-reduced-motion:reduce){.orb-frame,.orb-frame *{animation:none!important}}
</style>
