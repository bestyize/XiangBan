<script setup lang="ts">
/**
 * 新建任务. Placeholder for Phase 2: for now it is the chat page's empty conversation.
 * The shell moves to the `chat` page once the first message creates a conversation.
 */
import ChatPage from './ChatPage.vue';
import type { ControlStore } from '../control-types.ts';
defineProps<{ store: ControlStore }>();
</script>
<template>
  <div class="new-task-page" data-testid="new-task-page"><ChatPage :store="store" /></div>
</template>
<style scoped>
.new-task-page{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column}
</style>
