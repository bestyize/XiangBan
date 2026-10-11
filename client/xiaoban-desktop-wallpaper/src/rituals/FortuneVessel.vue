<script setup lang="ts">
import { useId } from 'vue';
defineProps<{ shaking?: boolean }>();
const id = useId().replace(/:/g, '');
</script>
<template>
  <svg class="fortune-vessel" :class="{ shaking }" viewBox="0 0 360 400" aria-hidden="true">
    <defs>
      <linearGradient :id="`${id}-wood`"><stop stop-color="#d5a65e"/><stop offset=".4" stop-color="#f7df9c"/><stop offset="1" stop-color="#ab763b"/></linearGradient>
      <linearGradient :id="`${id}-red`"><stop stop-color="#6c2326"/><stop offset=".25" stop-color="#aa4540"/><stop offset=".55" stop-color="#be5750"/><stop offset="1" stop-color="#6b2429"/></linearGradient>
      <linearGradient :id="`${id}-gold`"><stop stop-color="#9e6c3d"/><stop offset=".4" stop-color="#f4d298"/><stop offset="1" stop-color="#bb8852"/></linearGradient>
      <radialGradient :id="`${id}-shadow`"><stop stop-color="#472d23" stop-opacity=".3"/><stop offset="1" stop-color="#472d23" stop-opacity="0"/></radialGradient>
    </defs>
    <ellipse cx="180" cy="355" rx="130" ry="25" :fill="`url(#${id}-shadow)`" />
    <g class="vessel-body">
      <ellipse cx="180" cy="158" rx="77" ry="23" fill="#492d29"/>
      <g v-for="(angle,i) in [-18,-12,-6,1,8,14,20]" :key="angle" :transform="`rotate(${angle} 180 230)`">
        <rect :x="151+i*5" :y="40+i%3*12" width="14" height="225" rx="3" :fill="`url(#${id}-wood)`" stroke="#9d723a" stroke-width=".7"/>
        <path :d="`M${158+i*5} ${67+i%3*12} v45`" stroke="#79523c" stroke-width="1.5" stroke-dasharray="5 4 7 3"/>
        <circle :cx="158+i*5" :cy="54+i%3*12" r="2" fill="#aa4c3f"/>
      </g>
      <g class="chosen-stick" opacity="0"><rect x="173" y="64" width="14" height="180" rx="3" :fill="`url(#${id}-wood)`" stroke="#9d723a" stroke-width=".7"/><text x="180" y="91" text-anchor="middle" fill="#914437" font-family="KaiTi, STKaiti, serif" font-size="10">吉</text></g>
      <path d="M104 156 Q180 196 256 156 L245 321 Q180 351 115 321 Z" :fill="`url(#${id}-red)`" stroke="#6d302b" stroke-width="1.5"/>
      <path d="M112 174 Q180 203 248 174 M116 308 Q180 334 244 308" fill="none" :stroke="`url(#${id}-gold)`" stroke-width="4"/>
      <path d="M129 189 L136 305 M237 189 L231 305" stroke="#ecc699" stroke-opacity=".16" stroke-width="1"/>
      <path d="M180 211 l34 37 -34 39 -34 -39 Z" fill="#762a2b" :stroke="`url(#${id}-gold)`" stroke-width="1"/>
      <text x="180" y="262" text-anchor="middle" fill="#eed2a0" font-family="KaiTi, STKaiti, serif" font-size="32">福</text>
      <ellipse cx="180" cy="157" rx="76" ry="21" fill="none" :stroke="`url(#${id}-gold)`" stroke-width="5"/>
      <path d="M120 164 Q180 191 243 164" fill="none" stroke="#fae1b6" stroke-opacity=".32" stroke-width="1"/>
    </g>
  </svg>
</template>
<style scoped>
.fortune-vessel{width:100%;height:100%;overflow:visible}.vessel-body{transform-origin:180px 320px}.shaking .vessel-body{animation:shake 380ms ease-in-out 3}.shaking .chosen-stick{animation:draw-stick .65s cubic-bezier(.2,.8,.2,1) 1.1s forwards}@keyframes draw-stick{from{opacity:0;transform:translateY(0)}to{opacity:1;transform:translateY(-52px)}}@keyframes shake{0%,100%{transform:rotate(0)}25%{transform:rotate(-7deg) translateY(-2px)}75%{transform:rotate(7deg) translateY(-2px)}}@media(prefers-reduced-motion:reduce){.shaking .vessel-body,.shaking .chosen-stick{animation:none}}
</style>
