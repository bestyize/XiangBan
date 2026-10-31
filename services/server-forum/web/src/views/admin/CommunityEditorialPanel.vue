<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { communityEditorial, type EditorialPack, type FeaturedItem } from '@/api/communityEditorial'
import CommunityChallengesPanel from './CommunityChallengesPanel.vue'
import { kindLabel } from '@/lib/content'
const items = ref<FeaturedItem[]>([])
const candidates = ref<EditorialPack[]>([])
const revision = ref(0)
const query = ref('')
const loading = ref(false)
const saving = ref(false)
const searching = ref(false)
const error = ref('')
const searchError = ref('')
const saved = ref(false)
async function load() {
  loading.value = true; error.value = ''; saved.value = false
  try { const data = await communityEditorial.featured(); items.value = data.items; revision.value = data.revision }
  catch (e) { error.value = e instanceof Error ? e.message : '加载失败' }
  finally { loading.value = false }
}
async function search() {
  searching.value = true; searchError.value = ''
  try { candidates.value = (await communityEditorial.candidates(query.value.trim())).items }
  catch (e) { searchError.value = e instanceof Error ? e.message : '搜索失败' }
  finally { searching.value = false }
}
function add(pack: EditorialPack) {
  if (items.value.some(item => item.pack_id === pack.id) || items.value.length >= 30) return
  items.value.push({ pack_id: pack.id, sort_order: items.value.length, visible: true, pack }); saved.value = false
}
function move(index: number, direction: number) {
  const target = index + direction
  if (target < 0 || target >= items.value.length) return
  const item = items.value.splice(index, 1)[0]!
  items.value.splice(target, 0, item); saved.value = false
}
async function save() {
  saving.value = true; error.value = ''; saved.value = false
  try {
    const data = await communityEditorial.saveFeatured(revision.value, items.value.map((item, index) => ({ pack_id: item.pack_id, sort_order: index })))
    items.value = data.items; revision.value = data.revision; saved.value = true
  } catch (e) { error.value = e instanceof Error ? e.message : '保存失败，请重新加载后重试' }
  finally { saving.value = false }
}
onMounted(load)
</script>
<template>
  <section class="panel adm-card editorial-panel">
    <div class="adm-toolbar"><h3>首页精选作品</h3><button class="btn" :disabled="loading || saving" @click="load">重新加载</button><button class="btn btn-primary" :disabled="loading || saving || revision === 0" @click="save">{{ saving ? '保存中…' : '保存精选顺序' }}</button></div>
    <p class="adm-sub">人工选择最多 30 件已上架作品。下架、作者被封禁或关联讨论不再公开时，作品会立即退出公开精选。</p>
    <p v-if="error" role="alert">{{ error }} <button class="btn" :disabled="loading || saving" @click="load">重新加载</button></p>
    <p v-if="saved" role="status">精选顺序已保存。</p>
    <p v-if="loading" role="status">正在加载精选…</p>
    <p v-else-if="!items.length" class="adm-sub">尚未选择精选作品，请从下方搜索添加。</p>
    <ol v-else class="editorial-list">
      <li v-for="(item, index) in items" :key="item.pack_id">
        <span>{{ item.pack.title }} · {{ kindLabel(item.pack.kind) }} <strong v-if="!item.visible">当前不可公开，请移除后再保存</strong></span>
        <span class="editorial-actions"><button class="btn" :disabled="index === 0 || saving" @click="move(index, -1)">上移</button><button class="btn" :disabled="index === items.length - 1 || saving" @click="move(index, 1)">下移</button><button class="btn" :disabled="saving" @click="items.splice(index, 1); saved = false">移除</button></span>
      </li>
    </ol>
    <form class="adm-toolbar" @submit.prevent="search"><input v-model="query" class="input" maxlength="100" placeholder="搜索作品标题或标识" aria-label="搜索公开作品"><button class="btn" :disabled="searching">{{ searching ? '搜索中…' : '搜索作品' }}</button></form>
    <p v-if="searchError" role="alert">{{ searchError }} <button class="btn" @click="search">重试</button></p>
    <p v-if="!searching && !searchError && !candidates.length" class="adm-sub">搜索后展示可公开精选的作品。</p>
    <ul class="editorial-list"><li v-for="pack in candidates" :key="pack.id"><span>{{ pack.title }} · {{ kindLabel(pack.kind) }}</span><button class="btn" :disabled="saving || items.length >= 30 || items.some(item => item.pack_id === pack.id)" @click="add(pack)">{{ items.some(item => item.pack_id === pack.id) ? '已选择' : '加入精选' }}</button></li></ul>
  </section>
  <CommunityChallengesPanel />
</template>
<style scoped>
.editorial-panel { display: grid; gap: 12px; }
.editorial-panel h3 { margin: 0; margin-right: auto; }
.editorial-list { margin: 0; padding-left: 24px; }
.editorial-list li { padding: 8px 0; display: flex; justify-content: space-between; gap: 12px; align-items: center; }
.editorial-actions { display: flex; gap: 6px; flex-shrink: 0; }
.editorial-panel strong { color: var(--danger, #b63333); font-weight: 500; }
@media (max-width: 700px) { .editorial-list li { align-items: flex-start; flex-direction: column; } }
</style>
