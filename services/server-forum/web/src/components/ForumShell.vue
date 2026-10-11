<script setup lang="ts">
import SideNav from './SideNav.vue'
import CategoryChips from './CategoryChips.vue'

defineProps<{ active?: string; rail?: boolean }>()
</script>

<template>
  <div class="shell" :class="{ 'has-rail': rail !== false && $slots.rail }">
    <aside class="left">
      <SideNav :active="active" />
    </aside>
    <div class="center">
      <CategoryChips class="chips" :active="active" />
      <slot />
    </div>
    <aside v-if="rail !== false && $slots.rail" class="right">
      <slot name="rail" />
    </aside>
  </div>
</template>

<style scoped>
/* 桌面三栏：分类导航 / 内容 / 侧栏；平板去掉侧栏、分类改成横向标签；手机只留内容。 */
.shell {
  display: grid;
  grid-template-columns: 196px minmax(0, 1fr);
  gap: 24px;
  align-items: start;
}
.shell.has-rail {
  grid-template-columns: 196px minmax(0, 1fr) 272px;
}
.left,
.right {
  position: sticky;
  top: calc(var(--header-h) + 20px);
  max-height: calc(100vh - var(--header-h) - 40px);
  overflow-y: auto;
  scrollbar-width: thin;
}
.right {
  display: grid;
  gap: 16px;
}
/* 灰色只做页面底；中间内容栏是白底，平铺在灰底上，没有边框、圆角和阴影。 */
.center {
  min-width: 0;
  background: var(--surface);
  padding: 18px 22px 24px;
  min-height: 70vh;
}
.chips {
  display: none;
}
@media (max-width: 1099px) {
  .shell,
  .shell.has-rail {
    grid-template-columns: 180px minmax(0, 1fr);
  }
  .right {
    display: none;
  }
}
@media (max-width: 859px) {
  .shell,
  .shell.has-rail {
    grid-template-columns: minmax(0, 1fr);
  }
  .left {
    display: none;
  }
  .chips {
    display: flex;
  }
}
@media (max-width: 767px) {
  /* 手机上白底铺满屏幕宽度。 */
  .center {
    margin: 0 -12px;
    padding: 12px 12px 20px;
    min-height: 60vh;
  }
}
</style>
