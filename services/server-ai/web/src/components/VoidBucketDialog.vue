<script setup lang="ts">
import { ref } from 'vue';
import { useModal } from '../composables/useModal';
import { kindLabels } from '../format';
import type { Bucket } from '../types';
import Money from './Money.vue';

const props = defineProps<{ bucket: Bucket; busy: boolean }>();
const emit = defineEmits<{ cancel: []; confirm: [note: string] }>();

const note = ref('');
const root = ref<HTMLElement | null>(null);
useModal(root, () => emit('cancel'));
</script>

<template>
  <div class="scrim over" @click="emit('cancel')"></div>
  <form ref="root" class="dialog over" role="alertdialog" aria-modal="true" aria-labelledby="void-title" @submit.prevent="emit('confirm', note.trim())">
    <div class="dialog-head">
      <h2 id="void-title">扣回这份额度？</h2>
      <p class="hint">将扣回这份{{ kindLabels[props.bucket.kind] ?? '' }}额度里剩下的 <Money :points="props.bucket.remaining" size="sm" />，用户马上就不能再用。已经消耗的部分不会退回，此操作不能撤销。</p>
    </div>
    <div class="dialog-body">
      <div class="field">
        <label for="void-note">原因（选填）</label>
        <input id="void-note" v-model="note" class="input" maxlength="100" placeholder="例如：淘宝订单退款" data-autofocus />
      </div>
    </div>
    <div class="dialog-foot">
      <button type="button" class="btn" @click="emit('cancel')">取消</button>
      <button type="submit" class="btn danger" :disabled="busy">{{ busy ? '扣回中…' : '确认扣回' }}</button>
    </div>
  </form>
</template>
