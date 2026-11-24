<script lang="ts">
let brandMarkSeq = 0
</script>

<script setup lang="ts">
const uid = `xb-mark-${++brandMarkSeq}`
</script>

<template>
  <svg class="mark" viewBox="0 0 120 120" aria-hidden="true">
    <defs>
      <linearGradient :id="`${uid}-bg-light`" x1="12%" y1="6%" x2="88%" y2="96%">
        <stop offset="0%" stop-color="#FF6584" />
        <stop offset="100%" stop-color="#F9744B" />
      </linearGradient>
      <radialGradient :id="`${uid}-aura-light`" cx="50%" cy="44%" r="48%">
        <stop offset="0%" stop-color="#FFE0A3" stop-opacity="0.24" />
        <stop offset="100%" stop-color="#FFE0A3" stop-opacity="0" />
      </radialGradient>
      <linearGradient :id="`${uid}-bg-dark`" x1="12%" y1="6%" x2="88%" y2="96%">
        <stop offset="0%" stop-color="#262023" />
        <stop offset="100%" stop-color="#121012" />
      </linearGradient>
      <radialGradient :id="`${uid}-aura-dark`" cx="50%" cy="44%" r="48%">
        <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.09" />
        <stop offset="100%" stop-color="#FFFFFF" stop-opacity="0" />
      </radialGradient>
      <linearGradient :id="`${uid}-sheen`" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" stop-color="#FFFFFF" stop-opacity="0.16" />
        <stop offset="45%" stop-color="#FFFFFF" stop-opacity="0" />
      </linearGradient>
    </defs>
    <g class="mark-bg-light">
      <rect x="0" y="0" width="120" height="120" rx="27" :fill="`url(#${uid}-bg-light)`" />
      <rect x="0" y="0" width="120" height="120" rx="27" :fill="`url(#${uid}-aura-light)`" />
      <rect x="0" y="0" width="120" height="120" rx="27" :fill="`url(#${uid}-sheen)`" />
      <rect x="0.75" y="0.75" width="118.5" height="118.5" rx="26.25" fill="none" stroke="#FFFFFF" stroke-opacity="0.28" stroke-width="1.5" />
    </g>
    <g class="mark-bg-dark">
      <rect x="0" y="0" width="120" height="120" rx="27" :fill="`url(#${uid}-bg-dark)`" />
      <rect x="0" y="0" width="120" height="120" rx="27" :fill="`url(#${uid}-aura-dark)`" />
      <rect x="0" y="0" width="120" height="120" rx="27" :fill="`url(#${uid}-sheen)`" />
      <rect x="0.75" y="0.75" width="118.5" height="118.5" rx="26.25" fill="none" stroke="#FFFFFF" stroke-opacity="0.13" stroke-width="1.5" />
    </g>
    <g transform="translate(55.5 62.5) scale(0.96)">
      <path fill="#FFFFFF" d="M -3.2 -30.2 Q 0 -34 3.2 -30.2 C 7.6 -15.6 15.6 -7.6 30.2 -3.2 Q 34 0 30.2 3.2 C 15.6 7.6 7.6 15.6 3.2 30.2 Q 0 34 -3.2 30.2 C -7.6 15.6 -15.6 7.6 -30.2 3.2 Q -34 0 -30.2 -3.2 C -15.6 -7.6 -7.6 -15.6 -3.2 -30.2 Z" />
    </g>
    <g transform="translate(82.5 35.5) scale(0.47)">
      <path fill="#FFFFFF" d="M -3.8 -29.8 Q 0 -34 3.8 -29.8 C 8.5 -15 15 -8.5 29.8 -3.8 Q 34 0 29.8 3.8 C 15 8.5 8.5 15 3.8 29.8 Q 0 34 -3.8 29.8 C -8.5 15 -15 8.5 -29.8 3.8 Q -34 0 -29.8 -3.8 C -15 -8.5 -8.5 -15 -3.8 -29.8 Z" />
    </g>
  </svg>
</template>

<style scoped>
.mark {
  width: 28px;
  height: 28px;
  flex: none;
}
.mark-bg-dark {
  display: none;
}
:global([data-theme='dark']) .mark-bg-light {
  display: none;
}
:global([data-theme='dark']) .mark-bg-dark {
  display: inline;
}
</style>
