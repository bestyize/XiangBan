<script setup lang="ts">
// 敏感词：拦截的词直接拒绝发布，复核的词先发布再进待审队列。匹配前会去掉空格和标点，所以不用逐个写变体。
import { onMounted, ref } from 'vue'
import { adminApi, type SensitiveWord } from '@/api/admin'
import { perform } from './shared'

const words = ref<SensitiveWord[]>([])
const text = ref('')
const action = ref<'block' | 'review'>('block')

async function load() {
  words.value = (await adminApi.words()).items
}

async function add() {
  if (!text.value.trim()) return
  const ok = await perform(async () => {
    const r = await adminApi.addWords(text.value, action.value)
    text.value = ''
    return r
  }, '已添加')
  if (ok) await load()
}

async function remove(w: SensitiveWord) {
  if (await perform(() => adminApi.deleteWord(w.id), '已删除')) await load()
}

onMounted(load)
</script>

<template>
  <form class="panel adm-card adm-form" @submit.prevent="add">
    <p class="adm-sub">每行一个词（也可以用逗号分隔），每个词 2～40 个字。命中“拦截”的内容无法发布，且不会告诉用户是哪个词；命中“复核”的内容先发布并进入待审队列。</p>
    <label class="field wide">
      <span>词</span>
      <textarea v-model="text" class="input" rows="4" spellcheck="false" />
    </label>
    <div class="adm-form-actions">
      <select v-model="action" class="select" aria-label="处理方式">
        <option value="block">拦截（拒绝发布）</option>
        <option value="review">复核（先发布，进待审）</option>
      </select>
      <button class="btn btn-primary" type="submit">添加</button>
    </div>
  </form>

  <div class="panel adm-card">
    <p v-if="!words.length" class="adm-empty">还没有敏感词</p>
    <table v-else class="adm-table">
      <thead><tr><th>词</th><th>处理方式</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="w in words" :key="w.id">
          <td data-label="词">{{ w.word }}</td>
          <td data-label="处理方式">{{ w.action === 'block' ? '拦截' : '复核' }}</td>
          <td data-label="操作" class="adm-actions"><button class="btn btn-sm" @click="remove(w)">删除</button></td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
