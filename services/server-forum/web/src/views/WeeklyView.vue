<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import ContentShell from '@/components/ContentShell.vue'
import PosterDialog from '@/components/PosterDialog.vue'
import PackCard from '@/components/PackCard.vue'
import { ApiError } from '@/api/forum'
import { contentApi, type WeeklyDetail } from '@/api/content'
import { useHead } from '@/lib/head'
import { useToast } from '@/stores/toast'

const route = useRoute()
const toast = useToast()
const issue = ref<WeeklyDetail | null>(null)
const missing = ref(false)
const posterOpen = ref(false)
const posterInput = computed(() =>
  issue.value ? { tag: `周刊 ${issue.value.week}`, title: issue.value.title, summary: issue.value.summary || '本周精选', url: `${window.location.origin}/weekly/${issue.value.week}`, seed: issue.value.week.length } : null,
)

useHead(() => ({ title: issue.value ? `${issue.value.title}（${issue.value.week}）` : '周刊' }))

watch(
  () => route.params.week,
  async week => {
    issue.value = null
    missing.value = false
    try {
      issue.value = await contentApi.weekly(String(week))
    } catch (e) {
      if (e instanceof ApiError && (e.status === 404 || e.status === 400)) missing.value = true
      else toast.error(e)
    }
  },
  { immediate: true },
)
</script>

<template>
  <ContentShell>
    <p v-if="missing" class="state">这一期不存在。<RouterLink to="/weekly">看全部周刊</RouterLink></p>
    <p v-else-if="!issue" class="state">加载中</p>
    <article v-else class="issue">
      <p class="week">{{ issue.week }}</p>
      <h1>{{ issue.title }}</h1>
      <p v-if="issue.summary" class="lead">{{ issue.summary }}</p>
      <button class="btn btn-sm" type="button" @click="posterOpen = true">生成海报</button>
      <section v-for="(it, i) in issue.items" :key="i" class="entry">
        <template v-if="it.type === 'pack'">
          <PackCard :pack="it" wide />
          <p v-if="it.note" class="note">{{ it.note }}</p>
        </template>
        <template v-else-if="it.type === 'topic'">
          <h3><RouterLink :to="`/t/${it.id}`">{{ it.title }}</RouterLink></h3>
          <p v-if="it.summary" class="note">{{ it.summary }}</p>
          <p v-if="it.note" class="note">{{ it.note }}</p>
        </template>
        <template v-else>
          <h3 v-if="it.title">{{ it.title }}</h3>
          <p v-if="it.note" class="note">{{ it.note }}</p>
        </template>
      </section>
    </article>
  </ContentShell>
  <PosterDialog :open="posterOpen" :poster="posterInput" :name="issue?.week ?? ''" @close="posterOpen = false" />
</template>

<style scoped>
.state {
  padding: 40px 16px;
  text-align: center;
  color: var(--ink-3);
}
.issue {
  padding: 16px;
}
.week {
  margin: 0;
  font-size: var(--fs-xs);
  color: var(--ink-3);
}
h1 {
  margin: 2px 0 0;
  font-size: var(--fs-xl);
  font-weight: 600;
}
.lead {
  margin: 10px 0 0;
  color: var(--ink-2);
  white-space: pre-line;
}
.entry {
  margin-top: 8px;
}
.entry h3 {
  margin: 14px 0 0;
  font-size: var(--fs-md);
  font-weight: 500;
}
.note {
  margin: 4px 0 0;
  color: var(--ink-2);
  font-size: var(--fs-sm);
}
</style>
