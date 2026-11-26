<script setup lang="ts">
const model = defineModel<'topup' | 'monthly'>({ required: true });

const options = [
  { value: 'topup', title: '加油包', desc: '默认 365 天，按量消耗' },
  { value: 'monthly', title: '月卡', desc: '默认 30 天，到期未用完作废' },
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
      :title="o.desc"
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
  gap: 8px;
}
.kind-opt {
  padding: 10px 12px;
  border-radius: var(--r-md);
  border: 1.5px solid transparent;
  background: var(--subtle);
  text-align: left;
  display: flex;
  flex-direction: column;
  gap: 3px;
  transition: border-color 140ms, background 140ms;
}
.kind-opt:hover {
  background: var(--hover);
}
.kind-opt.on {
  border-color: var(--brand);
  background: var(--surface);
}
.top {
  display: flex;
  align-items: center;
  gap: 8px;
}
.top strong {
  font-size: 13px;
  font-weight: 600;
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
  border-color: var(--brand);
}
.radio.on::after {
  content: '';
  position: absolute;
  inset: 3px;
  border-radius: 50%;
  background: var(--brand);
}
small {
  color: var(--ink-3);
  font-size: 11.5px;
  line-height: 1.4;
  padding-left: 23px;
}
@media (max-width: 520px) {
  .kind-grid { grid-template-columns: 1fr; }
}
</style>
