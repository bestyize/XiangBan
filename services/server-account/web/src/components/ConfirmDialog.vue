<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import Icon from './Icon.vue';

const props = withDefaults(
  defineProps<{
    title: string;
    description?: string;
    confirmText?: string;
    cancelText?: string;
    danger?: boolean;
    busy?: boolean;
    error?: string;
    inputLabel?: string;
    inputPlaceholder?: string;
    inputHint?: string;
    initialValue?: string;
    maxLength?: number;
    over?: boolean;
  }>(),
  {
    confirmText: '确认',
    cancelText: '取消',
    danger: false,
    busy: false,
    over: false,
  },
);

const emit = defineEmits<{
  close: [];
  confirm: [value: string];
}>();

const value = ref(props.initialValue ?? '');
watch(
  () => props.initialValue,
  (v) => {
    value.value = v ?? '';
  },
);

function submit() {
  if (props.busy) return;
  if (props.inputLabel && !value.value.trim()) return;
  emit('confirm', value.value.trim());
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && !props.busy) emit('close');
}
onMounted(() => window.addEventListener('keydown', onKey));
onUnmounted(() => window.removeEventListener('keydown', onKey));
</script>

<template>
  <div class="scrim" :class="{ over }" @click="!busy && emit('close')" />
  <form
    class="dialog"
    :class="{ over }"
    role="dialog"
    aria-modal="true"
    :aria-label="title"
    @submit.prevent="submit"
  >
    <div class="dialog-head">
      <div>
        <h3>{{ title }}</h3>
        <p v-if="description">{{ description }}</p>
      </div>
      <button type="button" class="btn ghost icon sm" :disabled="busy" aria-label="关闭" @click="emit('close')">
        <Icon name="x" :size="16" />
      </button>
    </div>

    <div v-if="inputLabel || error || $slots.default" class="dialog-body">
      <slot />
      <label v-if="inputLabel" class="field">
        <span class="field-label">{{ inputLabel }}</span>
        <input
          v-model="value"
          class="input"
          :placeholder="inputPlaceholder"
          :maxlength="maxLength"
          required
          autofocus
        />
        <span v-if="inputHint" class="hint">{{ inputHint }}</span>
      </label>
      <div v-if="error" class="banner danger" role="alert">
        <Icon name="alert" :size="15" class="banner-ico" />
        <span>{{ error }}</span>
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" :disabled="busy" @click="emit('close')">{{ cancelText }}</button>
      <button
        type="submit"
        class="btn"
        :class="danger ? 'danger-solid' : 'primary'"
        :disabled="busy || (!!inputLabel && !value.trim())"
      >
        {{ busy ? '处理中…' : confirmText }}
      </button>
    </div>
  </form>
</template>
