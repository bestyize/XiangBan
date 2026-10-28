<script setup lang="ts">
import { computed } from 'vue'
import { useMeta } from '@/stores/meta'

const props = defineProps<{ active?: string }>()
const meta = useMeta()

const current = computed(() => (props.active ? meta.bySlug.get(props.active) : undefined))
/** 进入某个一级分类后，第二行显示它的子分类。 */
const root = computed(() => meta.tree.find(n => n.slug === meta.parentOf(current.value)?.slug))
</script>

<template>
  <div class="chips-wrap">
    <nav class="row" aria-label="分类">
      <RouterLink to="/discuss" class="chip" :class="{ on: active === 'home' }">首页</RouterLink>
      <RouterLink
        v-for="node in meta.tree"
        :key="node.id"
        :to="`/c/${node.slug}`"
        class="chip"
        :class="{ on: root?.slug === node.slug }"
        >{{ node.name }}</RouterLink
      >
    </nav>
    <nav v-if="root && root.children.length" class="row sub" :aria-label="`${root.name}的子分类`">
      <RouterLink :to="`/c/${root.slug}`" class="chip" :class="{ on: active === root.slug }">全部</RouterLink>
      <RouterLink
        v-for="c in root.children"
        :key="c.id"
        :to="`/c/${c.slug}`"
        class="chip"
        :class="{ on: active === c.slug }"
        >{{ c.name }}</RouterLink
      >
    </nav>
  </div>
</template>

<style scoped>
.chips-wrap {
  flex-direction: column;
  gap: 8px;
  margin: 0 -12px 12px;
}
.row {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  padding: 0 12px;
  scrollbar-width: none;
}
.row::-webkit-scrollbar {
  display: none;
}
.chip {
  flex: none;
  padding: 5px 12px;
  border-radius: 16px;
  border: 1px solid var(--line);
  background: var(--surface);
  font-size: var(--fs-sm);
  font-weight: 600;
  color: var(--ink-2);
}
.chip.on {
  background: var(--ink);
  border-color: var(--ink);
  color: var(--paper);
}
.sub .chip {
  font-weight: 500;
  font-size: var(--fs-xs);
  padding: 4px 10px;
}
.sub .chip.on {
  background: var(--brand);
  border-color: var(--brand);
  color: var(--brand-ink);
}
</style>
