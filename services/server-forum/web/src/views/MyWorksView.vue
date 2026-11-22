<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppIcon from '@/components/AppIcon.vue'
import ContentShell from '@/components/ContentShell.vue'
import MySchemesPanel from '@/components/MySchemesPanel.vue'
import { contentApi, type MyPack } from '@/api/content'
import { kindLabel } from '@/lib/content'
import { timeAgo } from '@/lib/format'
import { useHead } from '@/lib/head'
import { usePackPublisher } from '@/stores/packPublisher'
import { useToast } from '@/stores/toast'

const props = defineProps<{ embedded?: boolean }>()
useHead(() => ({ title: '我的作品' }))
const toast = useToast()
const publisher = usePackPublisher()
const route = useRoute()

const worksPath = computed(() => (props.embedded ? '/create/works' : '/me/works'))
const schemeTab = computed(() => route.query.tab === 'schemes')
const items = ref<MyPack[]>([])
const loading = ref(true)

type StatusFilter = 'all' | 'live' | 'review' | 'removed'
const statusFilter = ref<StatusFilter>('all')
const openMenuId = ref<number | null>(null)

function filterKey(p: MyPack): Exclude<StatusFilter, 'all'> {
  if (p.status === 'removed') return 'removed'
  if (p.status === 'rejected' || p.in_review) return 'review'
  return 'live'
}

function statusInfo(p: MyPack): { label: string; cls: string } {
  if (p.status === 'removed') return { label: '已下架', cls: 'removed' }
  if (p.status === 'rejected') return { label: '未通过', cls: 'danger' }
  if (p.in_review) return { label: '审核中', cls: 'review' }
  return { label: p.trust_level === 0 ? '官方' : '已上线', cls: 'live' }
}

function kindMeta(kind: string): { icon: string; cls: string } {
  if (kind === 'game') return { icon: 'gamepad', cls: 'game' }
  if (kind === 'app' || kind === 'tool') return { icon: 'grid', cls: 'app' }
  return { icon: 'ai', cls: 'scheme' }
}

const filteredItems = computed(() =>
  statusFilter.value === 'all' ? items.value : items.value.filter(p => filterKey(p) === statusFilter.value),
)

const counts = computed(() => ({
  all: items.value.length,
  live: items.value.filter(p => filterKey(p) === 'live').length,
  review: items.value.filter(p => filterKey(p) === 'review').length,
  removed: items.value.filter(p => filterKey(p) === 'removed').length,
}))

async function load() {
  loading.value = true
  try {
    items.value = (await contentApi.myPacks()).items
  } catch (e) {
    toast.error(e)
  } finally {
    loading.value = false
  }
}

async function remove(p: MyPack) {
  openMenuId.value = null
  if (!window.confirm(`确定下架「${p.title}」？已经应用过它的人不受影响。`)) return
  try {
    await contentApi.removeOwn(p.id)
    toast.show('已下架')
    await load()
  } catch (e) {
    toast.error(e)
  }
}

function openNewVersion(p: MyPack) {
  openMenuId.value = null
  publisher.open(p.id)
}

function closeMenuOnOutside(event: MouseEvent) {
  const el = event.target as HTMLElement | null
  if (openMenuId.value !== null && !el?.closest('.w-menu-wrap')) openMenuId.value = null
}

onMounted(() => document.addEventListener('click', closeMenuOnOutside))
onBeforeUnmount(() => document.removeEventListener('click', closeMenuOnOutside))
watch(schemeTab, value => { if (!value) void load() }, { immediate: true })
</script>

<template>
  <ContentShell>
    <div class="w-page" :class="{ embedded }">
      <header class="w-head">
        <component :is="embedded ? 'h2' : 'h1'">{{ embedded ? '已发布作品' : '我的作品' }}</component>
        <div class="w-head-ops">
          <RouterLink to="/me/content-bookmarks" class="w-pill secondary">
            <AppIcon name="bookmark" :size="14" />
            <span>作品收藏</span>
          </RouterLink>
          <RouterLink v-if="schemeTab" class="w-pill primary" to="/schemes/new">
            <AppIcon name="plus" :size="14" />
            <span>发布方案</span>
          </RouterLink>
          <button v-else type="button" class="w-pill primary" aria-haspopup="dialog" @click="publisher.open()">
            <AppIcon name="plus" :size="14" />
            <span>发布内容包</span>
          </button>
        </div>
      </header>

      <section class="w-card">
        <nav class="w-tabs" aria-label="我的社区内容">
          <RouterLink :to="worksPath" :class="{ active: !schemeTab }">
            <span>内容包</span>
            <small v-if="!loading">{{ items.length }}</small>
          </RouterLink>
          <RouterLink :to="{ path: worksPath, query: { tab: 'schemes' } }" :class="{ active: schemeTab }">
            <span>方案</span>
          </RouterLink>
        </nav>

        <template v-if="!schemeTab">
          <div v-if="!loading && items.length" class="w-filters">
            <button type="button" class="w-chip" :class="{ active: statusFilter === 'all' }" @click="statusFilter = 'all'">全部 {{ counts.all }}</button>
            <button type="button" class="w-chip" :class="{ active: statusFilter === 'live' }" @click="statusFilter = 'live'">已上线 {{ counts.live }}</button>
            <button type="button" class="w-chip" :class="{ active: statusFilter === 'review' }" @click="statusFilter = 'review'">审核中 {{ counts.review }}</button>
            <button type="button" class="w-chip" :class="{ active: statusFilter === 'removed' }" @click="statusFilter = 'removed'">已下架 {{ counts.removed }}</button>
          </div>

          <p v-if="loading" class="w-state">加载中…</p>
          <div v-else-if="!items.length" class="w-empty">
            <span class="w-empty-ico"><AppIcon name="folder" :size="22" /></span>
            <strong>还没有发布过作品</strong>
            <p>发布到社区的小程序、游戏与桌面装扮都在这里统一管理版本。</p>
            <button type="button" class="w-pill primary" @click="publisher.open()">发布内容包</button>
          </div>
          <div v-else-if="!filteredItems.length" class="w-empty">
            <span class="w-empty-ico"><AppIcon name="folder" :size="22" /></span>
            <strong>该状态下暂无作品</strong>
            <p>切换其他状态筛选查看已发布作品。</p>
          </div>
          <ul v-else class="w-list">
            <li v-for="p in filteredItems" :key="p.id" class="w-row">
              <span class="w-ico" :class="kindMeta(p.kind).cls">
                <AppIcon :name="kindMeta(p.kind).icon" :size="21" />
              </span>
              <RouterLink :to="`/packs/${p.id}`" class="w-main">
                <div class="w-title-row">
                  <strong class="w-title">{{ p.title }}</strong>
                  <span class="w-badge" :class="statusInfo(p).cls">{{ statusInfo(p).label }}</span>
                </div>
                <p class="w-meta">
                  <span>{{ kindLabel(p.kind) }}</span>
                  <span aria-hidden="true">·</span>
                  <span>v{{ p.version }}</span>
                  <template v-if="p.apply_count">
                    <span aria-hidden="true">·</span>
                    <span>{{ p.apply_count }} 次使用</span>
                  </template>
                  <span aria-hidden="true">·</span>
                  <span>{{ timeAgo(p.updated_at) }}更新</span>
                </p>
              </RouterLink>
              <div class="w-ops">
                <button
                  v-if="p.status !== 'removed'"
                  type="button"
                  class="w-pill secondary sm w-ver-pc"
                  aria-haspopup="dialog"
                  @click="openNewVersion(p)"
                >新版本</button>
                <div class="w-menu-wrap">
                  <button
                    type="button"
                    class="w-icon-btn"
                    aria-label="更多操作"
                    @click.stop="openMenuId = openMenuId === p.id ? null : p.id"
                  >
                    <AppIcon name="more" :size="18" />
                  </button>
                  <div v-if="openMenuId === p.id" class="w-pop-menu" role="menu">
                    <RouterLink :to="`/packs/${p.id}`" class="w-pop-item" role="menuitem" @click="openMenuId = null">
                      <AppIcon name="eye" :size="15" />
                      <span>查看详情</span>
                    </RouterLink>
                    <button
                      v-if="p.status !== 'removed'"
                      type="button"
                      class="w-pop-item"
                      role="menuitem"
                      @click="openNewVersion(p)"
                    >
                      <AppIcon name="upload" :size="15" />
                      <span>发布新版本</span>
                    </button>
                    <button
                      v-if="p.status !== 'removed'"
                      type="button"
                      class="w-pop-item danger"
                      role="menuitem"
                      @click="remove(p)"
                    >
                      <AppIcon name="trash" :size="15" />
                      <span>下架作品</span>
                    </button>
                  </div>
                </div>
              </div>
            </li>
          </ul>
        </template>
        <MySchemesPanel v-else />
      </section>
    </div>
  </ContentShell>
</template>

<style scoped>
.w-page {
  max-width: 820px;
  margin: 0 auto;
  padding: 6px 0 28px;
}
.w-page.embedded { max-width: none; padding-top: 0; }

.w-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
}
.w-head h1,
.w-head h2 {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 700;
}
.w-head-ops {
  display: flex;
  align-items: center;
  gap: 8px;
}

.w-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  height: 32px;
  padding: 0 13px;
  border: 0;
  border-radius: 999px;
  font-size: 12.5px;
  font-weight: 600;
  white-space: nowrap;
  transition: opacity 0.14s, background-color 0.14s;
}
.w-pill:hover { opacity: 0.88; }
.w-pill.primary { background: var(--brand-soft); color: var(--brand); }
.w-pill.secondary { background: var(--surface-2); color: var(--ink-2); font-weight: 500; }
.w-pill.sm { height: 30px; padding: 0 11px; font-size: 12px; }

.w-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: 16px;
  box-shadow: var(--shadow-card);
  overflow: visible;
}

.w-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 12px;
  border-bottom: 1px solid var(--line);
}
.w-tabs a {
  position: relative;
  height: 44px;
  padding: 0 12px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 14.5px;
  font-weight: 500;
  color: var(--ink-2);
}
.w-tabs a small { font-size: 12px; color: var(--ink-3); }
.w-tabs a.active { color: var(--ink); font-weight: 650; }
.w-tabs a.active::after {
  content: '';
  position: absolute;
  left: 50%;
  bottom: 0;
  transform: translateX(-50%);
  width: 20px;
  height: 2.5px;
  border-radius: 999px;
  background: var(--brand);
}

.w-filters {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--line);
}
.w-chip {
  height: 28px;
  padding: 0 11px;
  border: 0;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--ink-2);
  font-size: 12px;
  font-weight: 500;
}
.w-chip.active {
  background: var(--brand-soft);
  color: var(--brand);
  font-weight: 600;
}

.w-list {
  list-style: none;
  margin: 0;
  padding: 0;
}
.w-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 13px;
  padding: 14px 14px 14px 18px;
  transition: background-color 0.12s;
}
.w-row:hover { background: var(--surface-2); }
.w-row + .w-row::before {
  content: '';
  position: absolute;
  top: 0;
  left: 77px;
  right: 16px;
  height: 1px;
  background: var(--line);
}

.w-ico {
  width: 46px;
  height: 46px;
  border-radius: 13px;
  display: grid;
  place-items: center;
  flex: none;
}
.w-ico.app { background: rgba(47, 111, 219, 0.11); color: #2f6fdb; }
.w-ico.game { background: rgba(217, 90, 43, 0.12); color: #d95a2b; }
.w-ico.scheme { background: rgba(113, 82, 216, 0.11); color: #7152d8; }

.w-main {
  flex: 1;
  min-width: 0;
}
.w-title-row {
  display: flex;
  align-items: center;
  gap: 7px;
  min-width: 0;
}
.w-title {
  font-size: 15px;
  line-height: 21px;
  font-weight: 600;
  color: var(--ink);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.w-badge {
  flex: none;
  height: 20px;
  padding: 0 7px;
  border-radius: 999px;
  font-size: 11px;
  line-height: 20px;
  font-weight: 550;
}
.w-badge.live { background: rgba(47, 163, 107, 0.12); color: #2fa36b; }
.w-badge.review { background: rgba(178, 106, 0, 0.13); color: #b26a00; }
.w-badge.removed { background: var(--surface-2); color: var(--ink-3); }
.w-badge.danger { background: var(--brand-soft); color: var(--danger, #e3203c); }

.w-meta {
  margin: 3px 0 0;
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 2px 6px;
  font-size: 12px;
  color: var(--ink-3);
}

.w-ops {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: none;
}
.w-icon-btn {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 10px;
  background: transparent;
  color: var(--ink-3);
  display: grid;
  place-items: center;
}
.w-icon-btn:hover { background: var(--surface-2); color: var(--ink); }

.w-menu-wrap { position: relative; }
.w-pop-menu {
  position: absolute;
  right: 0;
  top: 38px;
  min-width: 146px;
  padding: 5px;
  border-radius: 12px;
  border: 1px solid var(--line);
  background: var(--surface);
  box-shadow: 0 10px 28px rgba(0, 0, 0, 0.14);
  z-index: 30;
}
.w-pop-item {
  width: 100%;
  height: 36px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--ink);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
  text-align: left;
}
.w-pop-item:hover { background: var(--surface-2); }
.w-pop-item.danger { color: var(--danger, #e3203c); }

.w-state { padding: 48px 16px; text-align: center; color: var(--ink-3); }
.w-empty {
  padding: 56px 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 9px;
}
.w-empty-ico {
  width: 48px;
  height: 48px;
  border-radius: 15px;
  background: var(--surface-2);
  color: var(--ink-3);
  display: grid;
  place-items: center;
}
.w-empty strong { font-size: 15px; font-weight: 600; color: var(--ink); }
.w-empty p { margin: 0; font-size: 13px; color: var(--ink-3); max-width: 260px; }

@media (max-width: 767px) {
  .w-head { padding: 4px 12px 0; margin-bottom: 10px; }
  .w-card { border-radius: 0; border-left: 0; border-right: 0; box-shadow: none; }
  .w-ver-pc { display: none; }
  .w-row { padding: 13px 10px 13px 16px; }
  .w-row + .w-row::before { left: 75px; right: 0; }
}
</style>
