<script setup lang="ts">
import type { ControlStore } from '../control-types.ts';
import ChatWorkbench from './ChatWorkbench.vue';
import DesignWorkbench from '../design/DesignWorkbench.vue';
import PreviewPanel from '../preview/PreviewPanel.vue';
import ChatQuestion from '../chat/ChatQuestion.vue';
import GuideModal from '../components/GuideModal.vue';

defineProps<{ store: ControlStore }>();
const nativePanels = { design: DesignWorkbench, preview: PreviewPanel, guide: GuideModal, question: ChatQuestion };
</script>
<template><ChatWorkbench :store="store" :native-panels="nativePanels" /></template>
