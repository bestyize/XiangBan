<script setup lang="ts">
import { computed, ref } from 'vue';
import { toDollars } from '../format';
import type { DayPoint } from '../types';
import Money from './Money.vue';

const props = defineProps<{ series: DayPoint[] }>();

const active = ref<number | null>(null);

const STEPS = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000, 50000, 100000];

/**
 * 纵轴只按“用掉”来定。发放是一次几美元的大块进账，用掉是每天几毛的细水长流，
 * 画在同一个纵轴上会把管理员最关心的成本压成一条细线，所以发放只在柱顶用小标记表示。
 * 纵轴取“好看的整数刻度”：分成四格，每格是 1/2/5 的整数倍。
 */
const scale = computed(() => {
  const peak = Math.max(0, ...props.series.map((d) => toDollars(d.consumed)));
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
  <figure class="chart" aria-label="近 14 天每天用掉的额度">
    <div class="legend">
      <span class="key"><i class="sw used"></i>用户用掉</span>
      <span class="key"><i class="sw gave"></i>当天有发放（悬停看金额）</span>
    </div>

    <div class="plot">
      <div class="axis" aria-hidden="true">
        <span v-for="g in grid" :key="g.at" class="tick num" :style="{ bottom: `${g.at}%` }">{{ g.label }}</span>
      </div>

      <div class="field">
        <div v-for="g in grid" :key="g.at" class="line" :style="{ bottom: `${g.at}%` }" aria-hidden="true"></div>

        <div class="cols" @mouseleave="active = null">
          <div
            v-for="(d, i) in series"
            :key="d.day"
            class="col"
            :class="{ on: active === i, today: i === series.length - 1 }"
            tabindex="0"
            :aria-label="`${d.day}：用掉 ${toDollars(d.consumed).toFixed(2)} 美元，发放 ${toDollars(d.granted).toFixed(2)} 美元`"
            @mouseenter="active = i"
            @focus="active = i"
            @blur="active = null"
          >
            <span v-if="d.granted > 0" class="mark" aria-hidden="true"></span>
            <div class="bars"><span class="bar" :style="{ height: height(d.consumed) }"></span></div>
            <div v-if="active === i" class="tip" :class="tipSide(i)" role="tooltip">
              <strong>{{ d.day.slice(5).replace('-', ' 月 ') }} 日</strong>
              <span class="trow"><i class="sw used"></i>用掉<Money :points="d.consumed" size="sm" /></span>
              <span class="trow"><i class="sw gave"></i>发放<Money :points="d.granted" size="sm" /></span>
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
.chart { display: flex; flex-direction: column; gap: 14px; }
.legend { display: flex; gap: 18px; justify-content: flex-end; color: var(--muted); font-size: 12.5px; font-weight: 600; flex-wrap: wrap; }
.key { display: inline-flex; align-items: center; gap: 7px; }
.sw { display: inline-block; width: 10px; height: 10px; border-radius: 3px; }
.sw.used { background: var(--tide); }
.sw.gave { width: 9px; height: 9px; border-radius: 50%; background: #3a86b4; }

.plot { display: grid; grid-template-columns: 48px 1fr; height: 236px; }
.axis { position: relative; height: calc(100% - 28px); }
.tick { position: absolute; right: 12px; transform: translateY(50%); font-size: 11.5px; color: var(--faint); font-weight: 600; }
.field { position: relative; }
.line { position: absolute; left: 0; right: 0; height: 0; border-top: 1px solid var(--line); }
.line:first-of-type { border-top-color: var(--line-strong); }
.cols { position: absolute; left: 0; right: 0; top: 0; bottom: 28px; display: flex; }
.col { position: relative; flex: 1; display: flex; align-items: flex-end; justify-content: center; cursor: default; border-radius: 6px 6px 0 0; outline-offset: -2px; }
.col.on { background: rgba(10, 107, 94, 0.06); }
.mark { position: absolute; top: 6px; left: 50%; width: 9px; height: 9px; margin-left: -4.5px; border-radius: 50%; background: #3a86b4; box-shadow: 0 0 0 3px rgba(58, 134, 180, 0.18); }
.bars { display: flex; align-items: flex-end; height: 100%; width: 62%; max-width: 34px; }
.bar { flex: 1; border-radius: 5px 5px 0 0; background: linear-gradient(180deg, #14907e, var(--tide)); transform-origin: bottom; animation: rise 0.55s cubic-bezier(0.2, 0.7, 0.2, 1) both; }
@keyframes rise { from { transform: scaleY(0); } }

.xlabels { position: absolute; left: 0; right: 0; bottom: 0; height: 28px; display: flex; align-items: flex-end; }
.xl { flex: 1; text-align: center; font-size: 11.5px; color: var(--faint); font-weight: 600; white-space: nowrap; overflow: visible; }
.xl.today { color: var(--tide); }

.tip { position: absolute; bottom: calc(100% + 8px); z-index: 5; min-width: 150px; padding: 10px 12px; border-radius: 10px; background: var(--ink); color: #eaf6f2; box-shadow: var(--shadow-2); display: flex; flex-direction: column; gap: 5px; font-size: 12.5px; pointer-events: none; }
.tip.mid { left: 50%; transform: translateX(-50%); }
.tip.start { left: 0; }
.tip.end { right: 0; }
.tip strong { font-size: 12.5px; color: #fff; }
.trow { display: flex; align-items: center; gap: 7px; color: #b9cfcb; }
.trow :deep(.money) { margin-left: auto; color: #fff; }
.trow .sw.gave { background: #6fa3c4; }
.none { position: absolute; inset: 0 0 28px; display: grid; place-items: center; text-align: center; color: var(--muted); padding: 0 24px; pointer-events: none; }

@media (max-width: 560px) {
  .plot { grid-template-columns: 40px 1fr; height: 200px; }
  .bars { width: 70%; }
}
</style>
