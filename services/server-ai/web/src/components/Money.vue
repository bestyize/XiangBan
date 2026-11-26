<script setup lang="ts">
import { computed } from 'vue';
import { moneyParts, xiaobanPoints } from '../format';

const props = withDefaults(
  defineProps<{
    points: number;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    muted?: boolean;
    signed?: boolean;
    hidePts?: boolean;
    compactPts?: boolean;
  }>(),
  { size: 'md', muted: false, signed: false, hidePts: false, compactPts: false },
);

const parts = computed(() => moneyParts(props.points));
const signText = computed(() => parts.value.sign || (props.signed && props.points > 0 ? '+' : ''));
const ptsText = computed(() => `${parts.value.sign}${xiaobanPoints(props.points)}`);
</script>

<template>
  <span class="money-val num" :class="[size, { muted, positive: signed && points > 0 }]">
    <span class="money-usd">
      <span v-if="signText" class="sign">{{ signText }}</span><span class="cur">$</span><span class="int">{{ parts.int }}</span><span class="dec">{{ parts.dec }}</span>
    </span>
    <span v-if="!hidePts" class="money-pts" :class="{ compact: compactPts }">{{ ptsText }}</span>
  </span>
</template>

<style scoped>
.money-val {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  flex-wrap: wrap;
  line-height: 1.15;
}
.money-usd {
  display: inline-flex;
  align-items: baseline;
  font-weight: 700;
  letter-spacing: -0.02em;
  white-space: nowrap;
}
.money-val.muted .money-usd {
  color: var(--ink-3);
  font-weight: 600;
}
.money-val.positive .money-usd {
  color: var(--ok);
}
.sign { margin-right: 1px; }
.cur { font-size: 0.72em; font-weight: 600; opacity: 0.6; margin-right: 1px; }
.dec { font-size: 0.76em; font-weight: 600; opacity: 0.65; }

.sm .money-usd { font-size: 13px; }
.md .money-usd { font-size: 14.5px; }
.lg .money-usd { font-size: 20px; }
.xl .money-usd { font-size: 24px; }

.money-pts {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--ink-3);
  white-space: nowrap;
}
.money-pts.compact { font-size: 11px; }
</style>
