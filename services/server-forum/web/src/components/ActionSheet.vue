<script setup lang="ts">
import UiActionSheet, { type ActionSheetOption } from '@/ui/overlay/ActionSheet.vue'

defineProps<{
  open: boolean
  label: string
  title?: string
  context?: string
  items?: ActionSheetOption[]
  cancelLabel?: string
  hideCancel?: boolean
}>()
const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <UiActionSheet
    :open="open"
    :label="label"
    :title="title"
    :context="context"
    :items="items"
    :cancel-label="cancelLabel"
    :hide-cancel="hideCancel"
    @close="emit('close')"
  >
    <slot />
  </UiActionSheet>
</template>
