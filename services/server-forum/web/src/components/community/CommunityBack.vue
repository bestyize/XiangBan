<script setup lang="ts">
import { useRouter } from 'vue-router'
const props = defineProps<{ fallback:string; label:string }>()
const router = useRouter()
function back(event:MouseEvent) {
  if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return
  event.preventDefault()
  const previous:unknown = window.history.state?.back
  if (typeof previous === 'string' && previous.startsWith('/') && !previous.startsWith('//')) router.back()
  else void router.push(props.fallback)
}
</script>
<template><a :href="fallback" class="community-back" @click="back">← {{ label }}</a></template>
