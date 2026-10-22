<script setup lang="ts">
import { BookOpen } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';
defineProps<{ store: ControlStore }>();
</script>
<template>
  <section class="library-page xb-page" data-testid="library-page" aria-label="资料库">
    <header class="xb-page-head"><h1 class="xb-page-title">资料库</h1></header>
    <div class="xb-empty library-empty">
      <span class="xb-empty-icon"><BookOpen :size="26" /></span>
      <h2>资料库正在准备中</h2>
      <p>以后小伴整理过的文件、网页和笔记都会收在这里，新建任务时可以随时引用。</p>
      <button type="button" class="xb-btn" @click="store.navigate('new-task')">先去新建任务</button>
    </div>
  </section>
</template>
<style scoped>
.library-page{flex:1;min-height:0;display:flex;flex-direction:column}
.library-empty{flex:1}
</style>
