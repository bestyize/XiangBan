<script setup lang="ts">
// 内容页的外壳：居中的白色单栏。栏目入口（首页、资源、讨论）在顶栏。
</script>

<template>
  <div class="content-shell"><slot /></div>
</template>

<style scoped>
.content-shell {
  max-width: 960px;
  margin: 0 auto;
  background: var(--surface);
  min-height: 70vh;
  padding-bottom: 24px;
}
@media (max-width: 767px) {
  .content-shell {
    margin: 0 -12px;
  }
}
</style>
