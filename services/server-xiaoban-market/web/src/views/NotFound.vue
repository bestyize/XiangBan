<template>
  <main class="nf">
    <h1>页面不存在</h1>
    <p>你访问的地址没有内容。</p>
    <RouterLink class="btn btn-primary" to="/">回到小伴市场</RouterLink>
  </main>
</template>

<style scoped>
.nf { min-height: 60vh; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; text-align: center; }
h1 { font-size: 26px; font-weight: 600; }
p { color: var(--ink-2); }
</style>
