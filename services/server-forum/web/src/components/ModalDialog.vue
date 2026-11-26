<script setup lang="ts">
import ResponsiveDialog from '@/ui/overlay/ResponsiveDialog.vue'

defineProps<{
  open: boolean
  title: string
  width?: number
  scrollable?: boolean
  keepOnBackdrop?: boolean
  closeDisabled?: boolean
  hideHeader?: boolean
  mobileMode?: 'sheet' | 'center'
}>()
const emit = defineEmits<{ close: [] }>()
</script>

<template>
  <ResponsiveDialog
    :open="open"
    :title="title"
    :width="width"
    :scrollable="scrollable"
    :keep-on-backdrop="keepOnBackdrop"
    :close-disabled="closeDisabled"
    :hide-header="hideHeader"
    :mobile-mode="mobileMode"
    @close="emit('close')"
  >
    <slot />
  </ResponsiveDialog>
</template>
