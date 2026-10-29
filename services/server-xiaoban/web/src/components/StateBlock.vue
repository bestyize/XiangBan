<script setup lang="ts">
defineProps<{ kind: 'empty' | 'error'; title: string; hint?: string; action?: string }>()
const emit = defineEmits<{ action: [] }>()
</script>

<template>
  <div class="state" role="status">
    <h2 class="display">{{ title }}</h2>
    <p v-if="hint">{{ hint }}</p>
    <button v-if="action" class="btn" :class="{ 'btn-primary': kind === 'error' }" type="button" @click="emit('action')">
      {{ action }}
    </button>
  </div>
</template>

<style scoped>
.state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 72px 24px;
  text-align: center;
}

h2 {
  font-size: 34px;
}

p {
  max-width: 34ch;
  color: var(--text-2);
}

.btn {
  margin-top: 12px;
}
</style>
