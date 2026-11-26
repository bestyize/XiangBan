<script setup lang="ts">
import { ref } from 'vue';
import { useModal } from '../composables/useModal';
import Icon from './Icon.vue';

const props = withDefaults(
  defineProps<{
    title: string;
    desc: string;
    confirmText?: string;
    busy?: boolean;
    over?: boolean;
    withNote?: boolean;
    noteLabel?: string;
    notePlaceholder?: string;
  }>(),
  {
    confirmText: '确认执行',
    busy: false,
    over: false,
    withNote: false,
    noteLabel: '原因（选填，写入流水与审计）',
    notePlaceholder: '例如：淘宝订单退款',
  },
);

const emit = defineEmits<{ cancel: []; confirm: [note: string] }>();

const note = ref('');
const root = ref<HTMLElement | null>(null);
useModal(root, () => {
  if (!props.busy) emit('cancel');
});
</script>

<template>
  <div class="scrim" :class="{ over }" @click="!busy && emit('cancel')"></div>
  <form
    ref="root"
    class="dialog narrow"
    :class="{ over }"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="confirm-title"
    @submit.prevent="emit('confirm', note.trim())"
  >
    <div class="dialog-head">
      <div>
        <h2 id="confirm-title">{{ title }}</h2>
        <p>{{ desc }}</p>
      </div>
      <button type="button" class="btn icon sm ghost" :disabled="busy" aria-label="关闭" @click="emit('cancel')">
        <Icon name="x" :size="16" />
      </button>
    </div>

    <div v-if="withNote" class="dialog-body">
      <div class="field">
        <label for="confirm-note">{{ noteLabel }}</label>
        <input id="confirm-note" v-model="note" class="input" maxlength="100" :placeholder="notePlaceholder" data-autofocus />
      </div>
    </div>

    <div class="dialog-foot">
      <button type="button" class="btn" :disabled="busy" @click="emit('cancel')">取消</button>
      <button type="submit" class="btn danger-solid" :disabled="busy">{{ busy ? '处理中…' : confirmText }}</button>
    </div>
  </form>
</template>
