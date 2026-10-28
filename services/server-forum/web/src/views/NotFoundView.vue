<script setup lang="ts">
import { useHead } from '@/lib/head'
useHead(() => ({ title: '页面不存在' }))
</script>

<template>
  <div class="nf">
    <p class="code" aria-hidden="true">404</p>
    <h1>这个页面不存在</h1>
    <p class="muted">链接可能写错了，或者内容已经被删除。</p>
    <div class="go">
      <RouterLink to="/" class="btn btn-primary">回到首页</RouterLink>
      <RouterLink to="/resources" class="btn">看看资源</RouterLink>
    </div>
  </div>
</template>

<style scoped>
.nf {
  text-align: center;
  padding: 60px 20px;
}
.code {
  margin: 0;
  font-family: var(--font-display);
  font-size: 72px;
  font-weight: 800;
  line-height: 1;
  color: var(--brand);
  -webkit-text-stroke: 2px var(--ink);
}
h1 {
  margin: 14px 0 6px;
  font-size: var(--fs-xl);
}
.go {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-top: 20px;
}
</style>
