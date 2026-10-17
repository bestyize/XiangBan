<script setup lang="ts">
const model = defineModel<'topup' | 'monthly'>({ required: true });

const options = [
  { value: 'topup', title: '加油包', desc: '长期有效，适合按量买' },
  { value: 'monthly', title: '月卡', desc: '限时额度，到期没用完就作废' },
] as const;
</script>

<template>
  <fieldset class="kinds">
    <legend class="sr">类型</legend>
    <label v-for="o in options" :key="o.value" class="kind" :class="{ on: model === o.value }">
      <input v-model="model" type="radio" :value="o.value" />
      <span><strong>{{ o.title }}</strong><small>{{ o.desc }}</small></span>
    </label>
  </fieldset>
</template>

<style scoped>
.kinds { border: none; margin: 0; padding: 0; display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.kind { display: flex; gap: 11px; align-items: flex-start; padding: 12px 14px; border: 1.5px solid var(--line); border-radius: 12px; cursor: pointer; transition: border-color 0.14s, background-color 0.14s, box-shadow 0.14s; }
.kind:hover { border-color: var(--faint); }
.kind.on { border-color: var(--tide); background: var(--tide-soft); box-shadow: 0 0 0 3px rgba(10, 107, 94, 0.1); }
.kind input { margin-top: 3px; accent-color: var(--tide); }
.kind span { display: flex; flex-direction: column; gap: 1px; }
.kind small { color: var(--muted); font-size: 12.5px; line-height: 1.45; }
</style>
