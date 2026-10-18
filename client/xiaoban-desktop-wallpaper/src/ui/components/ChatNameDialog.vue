<script setup lang="ts">
import { ref, watch } from 'vue';
import { NButton, NInput, NModal } from 'naive-ui';

const props = defineProps<{ show: boolean; title: string; placeholder: string; initial: string; maxLength: number; confirmText: string; busy: boolean }>();
const emit = defineEmits<{ close: []; submit: [value: string] }>();

const text = ref('');
const input = ref<InstanceType<typeof NInput> | null>(null);

watch(() => props.show, show => { if (show) text.value = props.initial; }, { immediate: true });

function submit() {
  if (props.busy || !text.value.trim()) return;
  emit('submit', text.value);
}
</script>

<template>
  <NModal :show="show" preset="card" :title="title" :mask-closable="!busy" style="max-width: 420px; width: 92vw;" data-testid="chat-name-dialog" @update:show="value => { if (!value && !busy) emit('close'); }" @after-enter="input?.select()">
    <form class="name-form" @submit.prevent="submit">
      <NInput ref="input" v-model:value="text" :placeholder="placeholder" :maxlength="maxLength" :disabled="busy" clearable data-testid="chat-name-input" />
      <div class="name-actions">
        <NButton :disabled="busy" data-testid="chat-name-cancel" @click="emit('close')">取消</NButton>
        <NButton type="primary" attr-type="submit" :loading="busy" :disabled="!text.trim()" data-testid="chat-name-ok">{{ confirmText }}</NButton>
      </div>
    </form>
  </NModal>
</template>

<style scoped>
.name-form { display: grid; gap: 16px; }
.name-actions { display: flex; justify-content: flex-end; gap: 10px; }
</style>
