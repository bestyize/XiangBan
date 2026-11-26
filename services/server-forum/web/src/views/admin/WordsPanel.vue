<script setup lang="ts">
// 敏感词库：拦截词（拒绝发布）与复核词（发布后进入待审队列）双栏分组展示。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type SensitiveWord } from '@/api/admin'
import { perform } from './shared'

const words = ref<SensitiveWord[]>([])
const text = ref('')
const action = ref<'block' | 'review'>('block')

const blockWords = computed(() => words.value.filter(w => w.action === 'block'))
const reviewWords = computed(() => words.value.filter(w => w.action === 'review'))

async function load() {
  words.value = (await adminApi.words()).items
}

async function add() {
  if (!text.value.trim()) return
  const ok = await perform(async () => {
    const r = await adminApi.addWords(text.value, action.value)
    text.value = ''
    return r
  }, '敏感词已添加')
  if (ok) await load()
}

async function remove(w: SensitiveWord) {
  if (await perform(() => adminApi.deleteWord(w.id), '已删除')) await load()
}

onMounted(load)
</script>

<template>
  <form class="panel adm-card adm-form" @submit.prevent="add">
    <p class="adm-sub" style="margin: 0">
      每行一个词（或用逗号分隔），每词 2–40 字。匹配前自动去除空格与标点；命中「拦截」直接拒绝发布，命中「复核」先发布并进入审核中心待审队列。
    </p>
    <label class="field wide">
      <span>批量添加词条</span>
      <textarea v-model="text" class="textarea" rows="3" spellcheck="false" placeholder="输入词条，每行一个或逗号分隔…" />
    </label>
    <div class="adm-form-actions" style="border-top: 0; padding-top: 0; margin-top: 0">
      <div class="adm-seg" role="group" aria-label="处理方式">
        <button type="button" :class="{ 'is-on': action === 'block' }" @click="action = 'block'">拦截（拒绝发布）</button>
        <button type="button" :class="{ 'is-on': action === 'review' }" @click="action = 'review'">复核（进入待审）</button>
      </div>
      <button class="btn btn-primary" type="submit" :disabled="!text.trim()">添加词条</button>
    </div>
  </form>

  <div class="adm-dash-grid" style="grid-template-columns: repeat(2, minmax(0, 1fr))">
    <section class="panel adm-card">
      <div class="adm-card-head">
        <h3>直接拦截词</h3>
        <span class="adm-tag danger">{{ blockWords.length }} 条</span>
      </div>
      <div class="adm-card-body">
        <p v-if="!blockWords.length" class="adm-empty" style="padding: 16px">暂无拦截词</p>
        <div v-else class="adm-chips" style="margin: 0">
          <button
            v-for="w in blockWords"
            :key="w.id"
            type="button"
            class="adm-tag danger"
            style="border: 0; cursor: pointer"
            title="点击删除"
            @click="remove(w)"
          >
            {{ w.word }} ×
          </button>
        </div>
      </div>
    </section>

    <section class="panel adm-card">
      <div class="adm-card-head">
        <h3>人工复核词</h3>
        <span class="adm-tag suspect">{{ reviewWords.length }} 条</span>
      </div>
      <div class="adm-card-body">
        <p v-if="!reviewWords.length" class="adm-empty" style="padding: 16px">暂无复核词</p>
        <div v-else class="adm-chips" style="margin: 0">
          <button
            v-for="w in reviewWords"
            :key="w.id"
            type="button"
            class="adm-tag suspect"
            style="border: 0; cursor: pointer"
            title="点击删除"
            @click="remove(w)"
          >
            {{ w.word }} ×
          </button>
        </div>
      </div>
    </section>
  </div>
</template>
