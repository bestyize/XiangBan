<script setup lang="ts">
const model = defineModel<'topup' | 'monthly'>({ required: true });

const options = [
  { value: 'topup', title: '加油包', desc: '长期有效（默认 365 天），按量消耗' },
  { value: 'monthly', title: '月卡', desc: '限时额度（默认 30 天），到期未用完作废' },
] as const;
</script>

<template>
  <div class="kind-grid" role="radiogroup" aria-label="额度类型">
    <button
      v-for="o in options"
      :key="o.value"
      type="button"
      role="radio"
      class="kind-opt"
      :class="{ on: model === o.value }"
      :aria-checked="model === o.value"
      @click="model = o.value"
    >
      <div class="top">
        <span class="radio" :class="{ on: model === o.value }"></span>
        <strong>{{ o.title }}</strong>
      </div>
      <small>{{ o.desc }}</small>
    </button>
  </div>
</template>

<style scoped>
.kind-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}
.kind-opt {
  padding: 11px 13px;
  border-radius: 11px;
  border: 1.5px solid var(--line-strong);
  background: var(--surface);
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 3px;
  transition: border-color 0.14s, background 0.14s, box-shadow 0.14s;
}
.kind-opt:hover {
  border-color: var(--line-focus);
}
.kind-opt.on {
  border-color: var(--ink);
  background: var(--surface-muted);
  box-shadow: var(--shadow-xs);
}
.top {
  display: flex;
  align-items: center;
  gap: 8px;
}
.top strong {
  font-size: 13.5px;
  font-weight: 650;
}
.radio {
  width: 15px;
  height: 15px;
  border-radius: 50%;
  border: 1.5px solid var(--line-strong);
  position: relative;
  flex: none;
}
.radio.on {
  border-color: var(--ink);
}
.radio.on::after {
  content: '';
  position: absolute;
  inset: 3px;
  border-radius: 50%;
  background: var(--ink);
}
small {
  color: var(--ink-3);
  font-size: 12px;
  line-height: 1.45;
  padding-left: 23px;
}
@media (max-width: 520px) {
  .kind-grid { grid-template-columns: 1fr; }
}
</style>
