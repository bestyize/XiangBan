<script setup lang="ts">
import { computed } from 'vue';
import { moneyParts } from '../format';

const props = withDefaults(defineProps<{ points: number; size?: 'sm' | 'md' | 'lg' | 'xl'; muted?: boolean; signed?: boolean }>(), { size: 'md', muted: false, signed: false });
const parts = computed(() => moneyParts(props.points));
</script>

<template>
  <span class="money num" :class="[size, { muted }]"><span v-if="parts.sign || (signed && points > 0)" class="sign">{{ parts.sign || '+' }}</span><span class="cur">$</span><span class="int">{{ parts.int }}</span><span class="dec">{{ parts.dec }}</span></span>
</template>

<style scoped>
.money { display: inline-flex; align-items: baseline; font-weight: 700; letter-spacing: -0.01em; line-height: 1.1; white-space: nowrap; }
.sign { margin-right: 0.06em; }
.money.muted { color: var(--faint); }
.sm { font-size: 13px; }
.md { font-size: 15px; }
/* 只有大号金额才把 $ 和小数位缩小，小号时缩小只会让数字看起来破碎。 */
.lg, .xl { letter-spacing: -0.02em; }
.lg { font-size: 26px; }
.xl { font-size: 40px; letter-spacing: -0.03em; }
.lg .cur, .xl .cur { font-weight: 600; font-size: 0.62em; margin-right: 0.08em; opacity: 0.6; transform: translateY(-0.06em); }
.lg .dec, .xl .dec { font-weight: 600; font-size: 0.66em; opacity: 0.62; margin-left: 0.02em; }
</style>
