<script setup lang="ts">
// 分类：按父子关系排成树，行内展开编辑；顶部可新建分类。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type AdminCategory, type CategoryInput } from '@/api/admin'
import CategoryForm from './CategoryForm.vue'
import { errorText, perform, trustName } from './shared'

const all = ref<AdminCategory[]>([])
const loading = ref(true)
const error = ref('')
const creating = ref(false)
const editingId = ref<number | null>(null)
const busy = ref(false)

const parents = computed(() => all.value.filter(c => c.parent_id === null))
const childrenOf = (id: number) => all.value.filter(c => c.parent_id === id)

/** 平铺成「父分类、其子分类…」的顺序，顺便标出层级。 */
const rows = computed(() => {
  const out: { c: AdminCategory; child: boolean }[] = []
  for (const p of parents.value) {
    out.push({ c: p, child: false })
    for (const ch of childrenOf(p.id)) out.push({ c: ch, child: true })
  }
  return out
})

async function load() {
  try {
    all.value = await adminApi.categories()
    error.value = ''
  } catch (e) {
    error.value = errorText(e)
  } finally {
    loading.value = false
  }
}

async function create(body: CategoryInput) {
  busy.value = true
  const ok = await perform(() => adminApi.createCategory(body), '分类已创建')
  busy.value = false
  if (ok) {
    creating.value = false
    await load()
  }
}

async function update(c: AdminCategory, body: CategoryInput) {
  busy.value = true
  const ok = await perform(() => adminApi.updateCategory(c.id, body), '分类已保存')
  busy.value = false
  if (ok) {
    editingId.value = null
    await load()
  }
}

function toggleEdit(c: AdminCategory) {
  editingId.value = editingId.value === c.id ? null : c.id
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <button class="btn btn-primary" @click="creating = !creating">{{ creating ? '收起新建表单' : '新建分类' }}</button>
  </div>

  <div v-if="creating" class="panel adm-card" style="margin-bottom: 12px">
    <CategoryForm :parents="parents" :busy="busy" @save="create" @cancel="creating = false" />
  </div>

  <div class="panel adm-card">
    <p v-if="error" class="adm-state error-text">{{ error }}</p>
    <p v-else-if="loading" class="adm-state muted">加载中</p>
    <p v-else-if="!rows.length" class="adm-empty">还没有分类</p>
    <div v-else>
      <div v-for="{ c, child } in rows" :key="c.id" class="adm-tree-row" :class="child ? 'is-child' : 'is-parent'">
        <div class="adm-tree-main">
          <div>
            <div class="adm-title">
              {{ c.name }}
              <span class="adm-tags" style="margin-left: 6px">
                <span v-if="c.admin_only" class="adm-tag line">仅守护者</span>
                <span v-if="!c.on_home" class="adm-tag line">不上首页</span>
                <span v-if="!c.award_points" class="adm-tag line">不奖励积分</span>
              </span>
            </div>
            <div class="adm-sub">
              <span class="adm-mono">{{ c.slug }}</span>
              <span>{{ c.topic_count }} 个帖子</span>
              <span>最低发帖等级 {{ trustName(c.min_post_level) }}</span>
              <span>排序 {{ c.sort_order }}</span>
            </div>
            <div v-if="c.description" class="adm-sub"><span>{{ c.description }}</span></div>
          </div>
          <button class="btn btn-sm" @click="toggleEdit(c)">{{ editingId === c.id ? '收起' : '编辑' }}</button>
        </div>
        <div v-if="editingId === c.id" class="adm-tree-edit">
          <CategoryForm :category="c" :parents="parents" :busy="busy" @save="update(c, $event)" @cancel="editingId = null" />
        </div>
      </div>
    </div>
  </div>
</template>
