<script setup lang="ts">
// 帖子管理：搜索与筛选，行内隐藏/删除/置顶/加精/锁定，移动分类，修改羊毛状态。
import { computed, onMounted, ref } from 'vue'
import { adminApi, type AdminCategory, type AdminTopic, type TopicAction, type TopicState } from '@/api/admin'
import AdminPager from './AdminPager.vue'
import { fmtTime, perform, TOPIC_STATUS, topicTone, usePagedList } from './shared'

const q = ref('')
const status = ref('all')
const category = ref('')
const categories = ref<AdminCategory[]>([])

const list = usePagedList<AdminTopic>(
  (page, size) =>
    adminApi.topics({ q: q.value.trim(), status: status.value, category: category.value, page, page_size: size }),
  30,
)

const catById = computed(() => new Map(categories.value.map(c => [c.id, c])))
const parents = computed(() => categories.value.filter(c => c.parent_id === null))
const childrenOf = (id: number) => categories.value.filter(c => c.parent_id === id)
const leaves = computed(() => categories.value.filter(c => !categories.value.some(x => x.parent_id === c.id)))

/** 可移动到的分类：叶子分类，且类型（羊毛/普通）与原分类一致。 */
function moveTargets(t: AdminTopic) {
  const kind = catById.value.get(t.category_id)?.kind
  return leaves.value.filter(c => c.kind === kind && c.id !== t.category_id)
}

function categoryLabel(c: AdminCategory) {
  const p = c.parent_id ? catById.value.get(c.parent_id) : null
  return p ? `${p.name} / ${c.name}` : c.name
}

async function moderate(t: AdminTopic, action: TopicAction, okText: string, confirmText?: string) {
  if (confirmText && !window.confirm(confirmText)) return
  const out: { state?: TopicState } = {}
  const ok = await perform(async () => {
    out.state = await adminApi.moderateTopic(t.id, action)
  }, okText)
  if (ok && out.state) Object.assign(t, out.state)
}

async function move(t: AdminTopic, ev: Event) {
  const el = ev.target as HTMLSelectElement
  const id = Number(el.value)
  el.value = ''
  if (!id) return
  const target = catById.value.get(id)
  const ok = await perform(() => adminApi.moveTopic(t.id, id), `已移动到「${target?.name ?? id}」`)
  if (ok && target) {
    t.category_id = target.id
    t.category_name = target.name
  }
}

onMounted(async () => {
  list.load()
  try {
    categories.value = await adminApi.categories()
  } catch {
    // 分类拿不到时只是不能按分类筛选和移动
  }
})
</script>

<template>
  <div class="adm-toolbar">
    <form @submit.prevent="list.search()">
      <input v-model="q" class="input" placeholder="搜索标题或帖子 ID" aria-label="搜索帖子" />
      <button class="btn" type="submit">搜索</button>
    </form>
    <select v-model="status" class="select" aria-label="状态" @change="list.search()">
      <option value="all">全部状态</option>
      <option value="normal">正常</option>
      <option value="hidden">已隐藏</option>
      <option value="deleted">已删除</option>
    </select>
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

  <div class="panel adm-card">
    <p v-if="list.error.value" class="adm-state error-text">{{ list.error.value }}</p>
    <p v-else-if="list.loading.value && !list.items.value.length" class="adm-state muted">加载中</p>
    <p v-else-if="!list.items.value.length" class="adm-empty">没有符合条件的帖子</p>
    <table v-else class="adm-table">
      <thead>
        <tr>
          <th>帖子</th>
          <th>状态</th>
          <th class="num">回复</th>
          <th class="act">操作</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="t in list.items.value" :key="t.id">
          <td class="full">
            <div class="adm-title">
              <a :href="`/t/${t.id}`" target="_blank" rel="noopener">{{ t.title }}</a>
              <span class="adm-tags" style="margin-left: 6px">
                <span v-if="t.pinned" class="adm-tag line">置顶</span>
                <span v-if="t.featured" class="adm-tag line">精华</span>
                <span v-if="t.locked" class="adm-tag line">已锁定</span>
                <span v-if="t.open_reports" class="adm-tag danger">{{ t.open_reports }} 条待处理举报</span>
              </span>
            </div>
            <div class="adm-sub">
              <span>#{{ t.id }}</span>
              <span>{{ t.category_name }}</span>
              <span>
                <a class="adm-link" :href="`/u/${t.author_uid}`" target="_blank" rel="noopener">{{ t.author_nickname }}</a>
              </span>
              <span>{{ fmtTime(t.created_at) }}</span>
            </div>
          </td>
          <td data-label="状态" class="shrink">
            <span class="adm-tags">
              <span class="adm-tag" :class="topicTone(t.status)">{{ TOPIC_STATUS[t.status] }}</span>
              <span v-if="t.is_deal" class="adm-tag">羊毛</span>
            </span>
          </td>
          <td data-label="回复" class="num">{{ t.reply_count }}</td>
          <td class="full act">
            <div class="adm-actions">
              <button v-if="t.status === 'normal'" class="btn btn-sm" @click="moderate(t, 'hide', '帖子已隐藏')">隐藏帖子</button>
              <button v-else class="btn btn-sm" @click="moderate(t, 'restore', '帖子已恢复')">恢复</button>
              <button
                v-if="t.status !== 'deleted'"
                class="btn btn-sm btn-danger"
                @click="moderate(t, 'delete', '帖子已删除', `确定删除帖子「${t.title}」？删除后前台不可见，可以再点「恢复」找回。`)"
              >
                删除帖子
              </button>
              <button class="btn btn-sm" @click="moderate(t, t.pinned ? 'unpin' : 'pin', t.pinned ? '已取消置顶' : '已置顶')">
                {{ t.pinned ? '取消置顶' : '置顶' }}
              </button>
              <button
                class="btn btn-sm"
                @click="moderate(t, t.featured ? 'unfeature' : 'feature', t.featured ? '已取消加精' : '已加精')"
              >
                {{ t.featured ? '取消加精' : '加精' }}
              </button>
              <button class="btn btn-sm" @click="moderate(t, t.locked ? 'unlock' : 'lock', t.locked ? '已解除锁定' : '已锁定')">
                {{ t.locked ? '解除锁定' : '锁定回复' }}
              </button>
              <select class="select is-sm" aria-label="移动到分类" @change="move(t, $event)">
                <option value="">移动到分类</option>
                <option v-for="c in moveTargets(t)" :key="c.id" :value="c.id">{{ categoryLabel(c) }}</option>
              </select>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <AdminPager :page="list.page.value" :page-size="list.pageSize" :total="list.total.value" :loading="list.loading.value" @go="list.goto" />
</template>
