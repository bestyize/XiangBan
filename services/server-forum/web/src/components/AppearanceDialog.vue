<script setup lang="ts">
import { useTheme } from '@/lib/theme'
import ModalDialog from './ModalDialog.vue'
import AppearanceOptions from './AppearanceOptions.vue'
const { appearanceOpen } = useTheme()
</script>
<template><ModalDialog :open="appearanceOpen" title="外观与配色" :width="480" @close="appearanceOpen = false"><AppearanceOptions v-if="appearanceOpen" /></ModalDialog></template>
