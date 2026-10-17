<script setup lang="ts">
defineProps<{ title: string; desc?: string }>();
</script>

<template>
  <div class="page-head">
    <div>
      <h1>{{ title }}</h1>
      <p v-if="desc">{{ desc }}</p>
    </div>
    <div v-if="$slots.default" class="actions"><slot /></div>
  </div>
</template>
