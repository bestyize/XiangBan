<script setup lang="ts">
import { computed, ref } from 'vue';
import { money, toDollars, xiaobanPoints } from '../format';
import type { DayPoint } from '../types';

const props = withDefaults(defineProps<{ series: DayPoint[]; mode?: 'consumed' | 'compare' }>(), { mode: 'consumed' });

const active = ref<number | null>(null);

const STEPS = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000];

const scale = computed(() => {
  const peak = Math.max(
    0,
    ...props.series.map((d) => toDollars(props.mode === 'compare' ? Math.max(d.consumed, d.granted) : d.consumed)),
  );
  const step = STEPS.find((s) => s * 4 >= peak) ?? STEPS[STEPS.length - 1];
  return { step, top: step * 4 };
});

const grid = computed(() =>
  [0, 1, 2, 3, 4].map((i) => {
    const value = scale.value.step * i;
    return { at: i * 25, label: scale.value.step < 1 ? `$${value.toFixed(2)}` : `$${value.toLocaleString('en-US')}` };
  }),
);

const empty = computed(() => props.series.every((d) => d.consumed === 0 && d.granted === 0));

function height(points: number): string {
  if (points <= 0) return '0';
  return `max(3px, ${(toDollars(points) / scale.value.top) * 100}%)`;
}

function label(day: string, index: number): string {
  if (index === props.series.length - 1) return '今天';
  return (props.series.length - 1 - index) % 3 === 0 ? day.slice(5).replace('-', '/') : '';
}

function tipSide(index: number): string {
  if (index <= 1) return 'start';
  if (index >= props.series.length - 2) return 'end';
  return 'mid';
}
</script>

<template>
  <figure class="chart" aria-label="近 14 天额度流水">
    <div class="plot">
      <div class="axis" aria-hidden="true">
        <span v-for="g in grid" :key="g.at" class="tick num" :style="{ bottom: `${g.at}%` }">{{ g.label }}</span>
      </div>

      <div class="field">
        <div
          v-for="(g, gi) in grid"
          :key="g.at"
          class="line"
          :class="{ base: gi === 0 }"
          :style="{ bottom: `${g.at}%` }"
          aria-hidden="true"
        ></div>

        <div class="cols" @mouseleave="active = null">
          <div
            v-for="(d, i) in series"
            :key="d.day"
            class="col"
            :class="{ on: active === i, today: i === series.length - 1 }"
            tabindex="0"
            :aria-label="`${d.day}：消耗 ${money(d.consumed)}，发放 ${money(d.granted)}`"
            @mouseenter="active = i"
            @focus="active = i"
            @blur="active = null"
          >
            <span v-if="mode === 'consumed' && d.granted > 0" class="mark" aria-hidden="true"></span>
            <div class="bars">
              <span class="bar used" :style="{ height: height(d.consumed) }"></span>
              <span v-if="mode === 'compare' && d.granted > 0" class="bar gave" :style="{ height: height(d.granted) }"></span>
            </div>
            <div v-if="active === i" class="tip" :class="tipSide(i)" role="tooltip">
              <strong class="num">{{ d.day.slice(5).replace('-', ' 月 ') }} 日{{ i === series.length - 1 ? '（今天）' : '' }}</strong>
              <span class="trow">
                <span><i class="sw used" :class="{ today: i === series.length - 1 }"></i>消耗</span>
                <strong class="num">{{ money(d.consumed) }} <em class="money-pts-inline">· {{ xiaobanPoints(d.consumed) }}</em></strong>
              </span>
              <span class="trow">
                <span><i class="sw gave"></i>发放</span>
                <strong class="num">{{ money(d.granted) }} <em class="money-pts-inline">· {{ xiaobanPoints(d.granted) }}</em></strong>
              </span>
            </div>
          </div>
        </div>

        <div class="xlabels" aria-hidden="true">
          <span v-for="(d, i) in series" :key="d.day" class="xl num" :class="{ today: i === series.length - 1 }">{{ label(d.day, i) }}</span>
        </div>

        <p v-if="empty" class="none">这两周还没有额度流水，用户开始使用后会出现在这里。</p>
      </div>
    </div>
  </figure>
</template>

<style scoped>
.chart { display: flex; flex-direction: column; }
.plot { display: grid; grid-template-columns: 46px 1fr; height: 224px; }
.axis { position: relative; height: calc(100% - 26px); }
.tick { position: absolute; right: 10px; transform: translateY(50%); font-size: 11px; color: var(--ink-3); font-weight: 550; }
.field { position: relative; }
.line { position: absolute; left: 0; right: 0; height: 0; border-top: 1px dashed var(--line); }
.line.base { border-top-style: solid; border-top-color: var(--line-strong); }
.cols { position: absolute; left: 0; right: 0; top: 0; bottom: 26px; display: flex; }
.col {
  position: relative;
  flex: 1;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  cursor: default;
  border-radius: 6px 6px 0 0;
  outline-offset: -2px;
}
.col.on { background: var(--canvas-soft); }
.mark {
  position: absolute;
  top: 6px;
  left: 50%;
  width: 6px;
  height: 6px;
  margin-left: -3px;
  border-radius: 50%;
  background: var(--info);
  box-shadow: 0 0 0 3px var(--info-soft);
}
.bars { display: flex; align-items: flex-end; justify-content: center; gap: 3px; height: 100%; width: 70%; max-width: 28px; }
.bar {
  flex: 1;
  border-radius: 4px 4px 0 0;
  transform-origin: bottom;
  animation: rise 0.45s var(--ease) both;
}
.bar.used { background: var(--ink); }
.col.today .bar.used { background: var(--coral-grad); }
.bar.gave { background: var(--info); opacity: 0.45; }
@keyframes rise { from { transform: scaleY(0); } }

.xlabels { position: absolute; left: 0; right: 0; bottom: 0; height: 26px; display: flex; align-items: flex-end; }
.xl { flex: 1; text-align: center; font-size: 11px; color: var(--ink-3); font-weight: 550; white-space: nowrap; }
.xl.today { color: var(--coral); font-weight: 700; }

.tip {
  position: absolute;
  bottom: calc(100% + 8px);
  z-index: 20;
  min-width: 178px;
  padding: 10px 12px;
  border-radius: 10px;
  background: #19171a;
  color: #f3f1f5;
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: var(--shadow-md);
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 12px;
  pointer-events: none;
}
.tip.mid { left: 50%; transform: translateX(-50%); }
.tip.start { left: 0; }
.tip.end { right: 0; }
.tip strong { color: #fff; font-weight: 650; }
.trow { display: flex; align-items: center; justify-content: space-between; gap: 10px; color: rgba(243, 241, 245, 0.78); }
.trow em { font-style: normal; opacity: 0.75; font-weight: 500; }
:global([data-unit='usd']) .money-pts-inline { display: none; }
.sw { display: inline-block; width: 7px; height: 7px; border-radius: 2px; margin-right: 6px; }
.sw.used { background: #e5e2e8; }
.sw.used.today { background: var(--coral); }
.sw.gave { background: #60a5fa; }

.none {
  position: absolute;
  inset: 0 0 26px;
  display: grid;
  place-items: center;
  text-align: center;
  color: var(--ink-3);
  padding: 0 24px;
  pointer-events: none;
}

@media (max-width: 560px) {
  .plot { grid-template-columns: 40px 1fr; height: 196px; }
}
</style>
