<script setup lang="ts">
// 产品库：列表、新建、行内编辑。接口一次返回全部产品，搜索在前端过滤。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type AdminProduct, type ProductInput } from '@/api/admin'
import ProductForm from './ProductForm.vue'
import { errorText, perform } from './shared'

const all = ref<AdminProduct[]>([])
const loading = ref(true)
const error = ref('')
const q = ref('')
const creating = ref(false)
const editingId = ref<number | null>(null)
const busy = ref(false)

const shown = computed(() => {
  const k = q.value.trim().toLowerCase()
  if (!k) return all.value
  return all.value.filter(p => [p.name, p.slug, p.vendor].some(s => s.toLowerCase().includes(k)))
})

async function load() {
  try {
    all.value = await adminApi.products()
    error.value = ''
  } catch (e) {
    error.value = errorText(e)
  } finally {
    loading.value = false
  }
}

async function create(body: ProductInput) {
  busy.value = true
  const ok = await perform(() => adminApi.createProduct(body), '产品已创建')
  busy.value = false
  if (ok) {
    creating.value = false
    await load()
  }
}

async function update(p: AdminProduct, body: ProductInput) {
  busy.value = true
  const ok = await perform(() => adminApi.updateProduct(p.id, body), '产品已保存')
  busy.value = false
  if (ok) {
    editingId.value = null
    await load()
  }
}

onMounted(load)
</script>

<template>
  <div class="adm-toolbar">
    <input v-model="q" class="input" placeholder="按产品名、slug 或厂商筛选" aria-label="筛选产品" />
    <button class="btn btn-primary" @click="creating = !creating">{{ creating ? '收起新建表单' : '新建产品' }}</button>
  </div>

  <div v-if="creating" class="panel adm-card" style="margin-bottom: 12px">
    <ProductForm :busy="busy" @save="create" @cancel="creating = false" />
  </div>

  <div class="panel adm-card">
    <p v-if="error" class="adm-state error-text">{{ error }}</p>
    <p v-else-if="loading" class="adm-state muted">加载中</p>
    <p v-else-if="!shown.length" class="adm-empty">{{ all.length ? '没有匹配的产品' : '产品库还是空的' }}</p>
    <div v-else>
      <div v-for="p in shown" :key="p.id" class="adm-tree-row">
        <div class="adm-tree-main">
          <div>
            <div class="adm-title">
              <a :href="`/p/${p.slug}`" target="_blank" rel="noopener">{{ p.name }}</a>
            </div>
            <div class="adm-sub">
              <span class="adm-mono">{{ p.slug }}</span>
              <span v-if="p.vendor">{{ p.vendor }}</span>
              <span>{{ p.deal_count }} 条羊毛</span>
              <span v-if="p.website">
                <a class="adm-link" :href="p.website" target="_blank" rel="noopener noreferrer">{{ p.website }}</a>
              </span>
            </div>
            <div v-if="p.description" class="adm-sub"><span>{{ p.description }}</span></div>
          </div>
          <button class="btn btn-sm" @click="editingId = editingId === p.id ? null : p.id">
            {{ editingId === p.id ? '收起' : '编辑' }}
          </button>
        </div>
        <div v-if="editingId === p.id" class="adm-tree-edit">
          <ProductForm :product="p" :busy="busy" @save="update(p, $event)" @cancel="editingId = null" />
        </div>
      </div>
    </div>
  </div>
</template>
