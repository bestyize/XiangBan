<script setup lang="ts">
import { computed } from 'vue'
import UiIcon from '@/ui/icons/UiIcon.vue'
import { resolveUiIconName } from '@/ui/icons/icons'

const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 22 })
const resolved = computed(() => resolveUiIconName(props.name))
</script>

<template>
  <UiIcon :name="resolved" :size="size" />
</template>
