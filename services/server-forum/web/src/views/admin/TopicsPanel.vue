<script setup lang="ts">
// 帖子管理：清爽表格 + 侧滑治理抽屉 + 多选批量悬浮操作条 + 软删除居中确认弹层。
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { adminApi, type AdminCategory, type AdminTopic, type PinScope, type TopicAction, type TopicState } from '@/api/admin'
import { postableCategories } from '@/lib/communityStructure'
import AdminDrawer from './AdminDrawer.vue'
import AdminIcon from './AdminIcon.vue'
import AdminModal from './AdminModal.vue'
import AdminPager from './AdminPager.vue'
import { fmtTime, notify, perform, TOPIC_STATUS, topicTone, trustName, usePagedList } from './shared'

const q = ref('')
const status = ref('all')
const category = ref('')
const categories = ref<AdminCategory[]>([])
const selectedIds = ref<number[]>([])
const activeTopic = ref<AdminTopic | null>(null)
const deleteTarget = ref<AdminTopic | null>(null)
const drawerBusy = ref(false)
const batchBusy = ref(false)

const drawerForm = reactive<{ pin_scope: PinScope; category_id: number; featured: boolean; locked: boolean }>({
  pin_scope: 'none',
  category_id: 0,
  featured: false,
  locked: false,
})

const list = usePagedList<AdminTopic>(
  (page, size) =>
    adminApi.topics({ q: q.value.trim(), status: status.value, category: category.value, page, page_size: size }),
  30,
)

const catById = computed(() => new Map(categories.value.map(c => [c.id, c])))
const parents = computed(() => categories.value.filter(c => c.parent_id === null))
const childrenOf = (id: number) => categories.value.filter(c => c.parent_id === id)
const pinScopeLabel: Record<PinScope, string> = {
  none: '不置顶',
  site: '全站置顶',
  parent: '一级板块置顶',
  category: '当前板块置顶',
}

const allChecked = computed(
  () => list.items.value.length > 0 && list.items.value.every(t => selectedIds.value.includes(t.id)),
)

watch(
  () => list.items.value,
  () => {
    selectedIds.value = []
  },
)

function toggleAll(checked: boolean) {
  selectedIds.value = checked ? list.items.value.map(t => t.id) : []
}

function toggleOne(id: number, checked: boolean) {
  if (checked) {
    if (!selectedIds.value.includes(id)) selectedIds.value = [...selectedIds.value, id]
  } else {
    selectedIds.value = selectedIds.value.filter(x => x !== id)
  }
}

function moveTargets(t: AdminTopic) {
  const kind = catById.value.get(t.category_id)?.kind
  return postableCategories(categories.value).filter(
    c => c.kind === kind || c.kind === 'uncategorized' || kind === 'uncategorized',
  )
}

function categoryLabel(c: AdminCategory) {
  const p = c.parent_id ? catById.value.get(c.parent_id) : null
  return p ? `${p.name} / ${c.name}` : c.name
}

function openDrawer(t: AdminTopic) {
  activeTopic.value = t
  drawerForm.pin_scope = t.pin_scope
  drawerForm.category_id = t.category_id
  drawerForm.featured = t.featured
  drawerForm.locked = t.locked
}

async function moderate(t: AdminTopic, action: TopicAction, okText: string) {
  const out: { state?: TopicState } = {}
  const ok = await perform(async () => {
    out.state = await adminApi.moderateTopic(t.id, action)
  }, okText)
  if (ok && out.state) Object.assign(t, out.state)
  return ok
}

async function saveDrawer() {
  const t = activeTopic.value
  if (!t || drawerBusy.value) return
  drawerBusy.value = true
  try {
    if (drawerForm.pin_scope !== t.pin_scope) {
      const action: TopicAction = drawerForm.pin_scope === 'none' ? 'unpin' : 'pin'
      const state = await adminApi.moderateTopic(t.id, action, drawerForm.pin_scope)
      Object.assign(t, state)
    }
    if (drawerForm.featured !== t.featured) {
      const state = await adminApi.moderateTopic(t.id, drawerForm.featured ? 'feature' : 'unfeature')
      Object.assign(t, state)
    }
    if (drawerForm.locked !== t.locked) {
      const state = await adminApi.moderateTopic(t.id, drawerForm.locked ? 'lock' : 'unlock')
      Object.assign(t, state)
    }
    if (drawerForm.category_id && drawerForm.category_id !== t.category_id) {
      await adminApi.moveTopic(t.id, drawerForm.category_id)
      const target = catById.value.get(drawerForm.category_id)
      if (target) {
        t.category_id = target.id
        t.category_name = target.name
      }
    }
    notify(`帖子 #${t.id} 设置已保存`)
    activeTopic.value = null
  } catch (e) {
    notify(e instanceof Error ? e.message : '保存失败', 'error')
  } finally {
    drawerBusy.value = false
  }
}

async function confirmDelete() {
  const t = deleteTarget.value
  if (!t) return
  const ok = await moderate(t, 'delete', '帖子已删除')
  if (ok) {
    deleteTarget.value = null
    if (activeTopic.value?.id === t.id) activeTopic.value = null
  }
}

async function batchModerate(action: 'hide' | 'restore' | 'feature', label: string) {
  if (!selectedIds.value.length || batchBusy.value) return
  batchBusy.value = true
  let okCount = 0
  for (const id of selectedIds.value) {
    const t = list.items.value.find(item => item.id === id)
    if (!t) continue
    try {
      const next = await adminApi.moderateTopic(t.id, action)
      Object.assign(t, next)
      okCount++
    } catch {
      // 跳过单条失败并继续处理剩余选中项
    }
  }
  batchBusy.value = false
  selectedIds.value = []
  notify(`${label}（成功 ${okCount} 篇）`)
}

onMounted(async () => {
  void list.load()
  try {
    categories.value = await adminApi.categories()
  } catch {
    // 分类拿不到时只是不能按分类筛选和移动
  }
})
</script>

<template>
  <div class="adm-toolbar is-spread">
    <div class="adm-toolbar" style="margin: 0">
      <form @submit.prevent="list.search()">
        <div class="adm-search-box">
          <AdminIcon name="search" :size="14" />
          <input v-model="q" class="input" placeholder="搜索标题或帖子 #ID" aria-label="搜索帖子" />
        </div>
        <button class="btn" type="submit">搜索</button>
      </form>
      <div class="adm-seg" role="group" aria-label="帖子状态">
        <button
          v-for="[val, lbl] in [['all', '全部'], ['normal', '正常'], ['hidden', '已隐藏'], ['deleted', '已删除']]"
          :key="val"
          type="button"
          :class="{ 'is-on': status === val }"
          @click="status = val; list.search()"
        >
          {{ lbl }}
        </button>
      </div>
      <select v-model="category" class="select" aria-label="分类" @change="list.search()">
        <option value="">全部分类</option>
        <template v-for="p in parents" :key="p.id">
          <option v-if="!childrenOf(p.id).length" :value="p.id">{{ p.name }}</option>
          <optgroup v-else :label="p.name">
            <option :value="p.id">{{ p.name }}（含全部子分类）</option>
            <option v-for="c in childrenOf(p.id)" :key="c.id" :value="c.id">{{ c.name }}</option>
          </optgroup>
        </template>
      </select>
    </div>
    <span class="muted">共 {{ list.total.value }} 篇</span>
  </div>

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">正在读取帖子列表…</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有符合条件的帖子</p>
    <div v-else class="adm-table-scroll">
      <table class="adm-table">
        <thead>
          <tr>
            <th style="width: 36px">
              <input type="checkbox" :checked="allChecked" aria-label="全选当前页" @change="toggleAll(($event.target as HTMLInputElement).checked)" />
            </th>
            <th>帖子与标签</th>
            <th>板块</th>
            <th>状态</th>
            <th class="num">回复</th>
            <th class="num" style="width: 150px">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="t in list.items.value" :key="t.id">
            <td>
              <input
                type="checkbox"
                :checked="selectedIds.includes(t.id)"
                :aria-label="`选择帖子 #${t.id}`"
                @change="toggleOne(t.id, ($event.target as HTMLInputElement).checked)"
              />
            </td>
            <td class="full">
              <div class="adm-title">
                <a :href="`/t/${t.id}`" target="_blank" rel="noopener">{{ t.title }}</a>
                <span class="adm-tags" style="margin-left: 6px">
                  <span v-if="t.pinned" class="adm-tag info">{{ pinScopeLabel[t.pin_scope] }}</span>
                  <span v-if="t.featured" class="adm-tag brand">精华</span>
                  <span v-if="t.locked" class="adm-tag line">已锁定</span>
                  <span v-if="t.open_reports" class="adm-tag danger">{{ t.open_reports }} 条待处理举报</span>
                </span>
              </div>
              <div class="adm-sub">
                <span class="adm-mono">#{{ t.id }}</span>
                <span>
                  <a class="adm-link" :href="`/u/${t.author_uid}`" target="_blank" rel="noopener">{{ t.author_nickname }}</a>
                </span>
                <span>{{ fmtTime(t.created_at) }}</span>
              </div>
            </td>
            <td data-label="板块" class="shrink">{{ t.category_name }}</td>
            <td data-label="状态" class="shrink">
              <span class="adm-tags">
                <span class="adm-tag" :class="topicTone(t.status)">
                  <span class="adm-dot" :class="t.status === 'normal' ? 'ok' : t.status === 'hidden' ? 'warn' : 'danger'" />
                  {{ TOPIC_STATUS[t.status] }}
                </span>
                <span v-if="t.min_read_level > 0" class="adm-tag">仅{{ trustName(t.min_read_level) }}以上</span>
              </span>
            </td>
            <td data-label="回复" class="num">{{ t.reply_count }}</td>
            <td class="num shrink">
              <div class="adm-actions">
                <button class="btn btn-sm" type="button" @click="openDrawer(t)">管理</button>
                <button
                  v-if="t.status === 'normal'"
                  class="btn btn-sm btn-ghost"
                  type="button"
                  @click="moderate(t, 'hide', '帖子已隐藏')"
                >
                  隐藏
                </button>
                <button
                  v-else
                  class="btn btn-sm btn-soft"
                  type="button"
                  @click="moderate(t, 'restore', '帖子已恢复')"
                >
                  恢复
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value" @go="list.goto" />
  </div>

  <div v-if="selectedIds.length" class="adm-batch-bar">
    <span>已勾选 <strong>{{ selectedIds.length }}</strong> 篇帖子</span>
    <div class="adm-actions">
      <button class="btn btn-sm" type="button" :disabled="batchBusy" @click="batchModerate('hide', '已批量隐藏')">批量隐藏</button>
      <button class="btn btn-sm" type="button" :disabled="batchBusy" @click="batchModerate('restore', '已批量恢复')">批量恢复</button>
      <button class="btn btn-sm" type="button" :disabled="batchBusy" @click="batchModerate('feature', '已批量加精')">批量加精</button>
      <button class="btn btn-sm" type="button" @click="selectedIds = []">取消选择</button>
    </div>
  </div>

  <AdminDrawer :open="!!activeTopic" :title="activeTopic ? `管理帖子 #${activeTopic.id}` : ''" @close="activeTopic = null">
    <template v-if="activeTopic">
      <div style="margin-bottom: 18px">
        <div class="adm-title" style="font-size: 16px; line-height: 1.45; margin-bottom: 6px">{{ activeTopic.title }}</div>
        <div class="adm-sub">
          <span>作者 {{ activeTopic.author_nickname }}</span>
          <span>发布于 {{ fmtTime(activeTopic.created_at) }}</span>
          <span>{{ activeTopic.reply_count }} 条回复</span>
          <a class="adm-link" :href="`/t/${activeTopic.id}`" target="_blank" rel="noopener">查看前台原帖 ↗</a>
        </div>
      </div>

      <div class="adm-form-grid" style="margin-bottom: 18px">
        <label class="field">
          <span>置顶范围</span>
          <select v-model="drawerForm.pin_scope" class="select" style="width: 100%">
            <option value="none">不置顶</option>
            <option value="site">全站置顶</option>
            <option v-if="catById.get(activeTopic.category_id)?.section === 'resources'" value="parent">一级板块置顶</option>
            <option v-if="catById.get(activeTopic.category_id)?.section !== 'resources' || catById.get(activeTopic.category_id)?.parent_id" value="category">当前板块置顶</option>
          </select>
        </label>
        <label class="field">
          <span>所属板块</span>
          <select v-model.number="drawerForm.category_id" class="select" style="width: 100%">
            <option v-for="c in moveTargets(activeTopic)" :key="c.id" :value="c.id">{{ categoryLabel(c) }}</option>
          </select>
        </label>
      </div>

      <div class="panel adm-card" style="padding: 14px 16px; display: grid; gap: 14px; margin-bottom: 18px">
        <label style="display: flex; align-items: center; justify-content: space-between; gap: 12px; cursor: pointer">
          <div>
            <div class="adm-title" style="font-size: 13px">设为精华帖</div>
            <div class="adm-sub">同帖首次加精会自动向作者发放 +10 社区积分</div>
          </div>
          <input v-model="drawerForm.featured" type="checkbox" />
        </label>
        <label style="display: flex; align-items: center; justify-content: space-between; gap: 12px; padding-top: 12px; border-top: 1px solid var(--adm-line-subtle); cursor: pointer">
          <div>
            <div class="adm-title" style="font-size: 13px">锁定回复</div>
            <div class="adm-sub">锁定后普通用户不可继续跟帖回复</div>
          </div>
          <input v-model="drawerForm.locked" type="checkbox" />
        </label>
      </div>

      <div class="adm-callout" style="border-left-color: var(--adm-danger)">
        <div class="adm-title" style="color: var(--adm-danger); margin-bottom: 6px">可见性与删除</div>
        <p style="margin: 0 0 10px">隐藏或删除后前台不可见，随时可点「恢复」找回。</p>
        <div class="adm-actions">
          <button
            v-if="activeTopic.status === 'normal'"
            class="btn btn-sm"
            type="button"
            @click="moderate(activeTopic, 'hide', '帖子已隐藏')"
          >
            隐藏帖子
          </button>
          <button v-else class="btn btn-sm btn-soft" type="button" @click="moderate(activeTopic, 'restore', '帖子已恢复')">
            恢复正常
          </button>
          <button
            v-if="activeTopic.status !== 'deleted'"
            class="btn btn-sm btn-danger"
            type="button"
            @click="deleteTarget = activeTopic"
          >
            删除帖子
          </button>
        </div>
      </div>
    </template>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="activeTopic = null">取消</button>
      <button class="btn btn-primary" type="button" :disabled="drawerBusy" @click="saveDrawer">
        {{ drawerBusy ? '保存中…' : '保存变更' }}
      </button>
    </template>
  </AdminDrawer>

  <AdminModal :open="!!deleteTarget" title="确认删除帖子？" @close="deleteTarget = null">
    <p v-if="deleteTarget" style="margin: 0; color: var(--adm-ink-2)">
      确定删除帖子「<strong>{{ deleteTarget.title }}</strong>」？删除后前台不可见，可以随时在后台点击「恢复」找回。
    </p>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="deleteTarget = null">取消</button>
      <button class="btn btn-danger-fill" type="button" @click="confirmDelete">确认删除</button>
    </template>
  </AdminModal>
</template>
