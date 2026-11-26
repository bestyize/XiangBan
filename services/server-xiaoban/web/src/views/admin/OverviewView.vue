<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import Icon from '@/components/Icon.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import * as api from '@/admin/api'
import type { AiCatalogue } from '@/admin/ai-types'
import { formatBytes, formatPercent } from '@/admin/format'
import type { CacheOverview, ProviderConfig, ProxyInfo, ProxyStrategy } from '@/admin/types'
import { toast } from '@/composables/toast'
import type { DesktopConfig, DesktopRelease } from '@/desktop'
import { useCatalog } from '@/stores/catalog'
import type { WallpaperItem } from '@/types'

const router = useRouter()
const catalog = useCatalog()

const loading = ref(true)
const publishedTotal = ref(0)
const videoTotal = ref(0)
const pendingTotal = ref(0)
const pendingItems = ref<WallpaperItem[]>([])
const releases = ref<DesktopRelease[]>([])
const desktop = ref<DesktopConfig | null>(null)
const aiCatalogue = ref<AiCatalogue | null>(null)
const aiBusy = ref(false)
const providers = ref<ProviderConfig[]>([])
const proxies = ref<ProxyInfo[]>([])
const proxyStrategy = ref<ProxyStrategy>('round_robin')
const cache = ref<CacheOverview | null>(null)

const latestPublished = computed(() => releases.value.find((r) => r.published))
const draftReleases = computed(() => releases.value.filter((r) => !r.published).length)
const activeNotices = computed(() => desktop.value?.announcements.filter((a) => a.published).length ?? 0)
const xiaobanAiOn = computed(() => aiCatalogue.value?.xiaobanAi?.enabled ?? true)
const activeProxies = computed(() => proxies.value.filter((p) => p.enabled).length)
const proxyPct = computed(() => (proxies.value.length ? Math.round((activeProxies.value / proxies.value.length) * 100) : 0))

const catName = (id?: string) => catalog.categories.find((c) => c.id === id)?.name || id || '未分类'

async function loadAll() {
  loading.value = true
  if (!catalog.categories.length) void catalog.load()
  const [pubRes, vidRes, pendRes, relRes, deskRes, aiRes, provRes, proxyRes, cacheRes] = await Promise.allSettled([
    api.listAdminWallpapers({ status: 'published', page: 1, per_page: 1 }),
    api.listAdminWallpapers({ status: 'published', media_type: 'video', page: 1, per_page: 1 }),
    api.listAdminWallpapers({ status: 'pending', page: 1, per_page: 5 }),
    api.listAppReleases(),
    api.getDesktopConfig(),
    api.getAiCatalogue(),
    api.listProviders(),
    api.listProxies(),
    api.getCache(),
  ])
  if (pubRes.status === 'fulfilled') publishedTotal.value = pubRes.value.total
  if (vidRes.status === 'fulfilled') videoTotal.value = vidRes.value.total
  if (pendRes.status === 'fulfilled') {
    pendingTotal.value = pendRes.value.total
    pendingItems.value = pendRes.value.items
  }
  if (relRes.status === 'fulfilled') releases.value = relRes.value
  if (deskRes.status === 'fulfilled') desktop.value = deskRes.value
  if (aiRes.status === 'fulfilled') aiCatalogue.value = aiRes.value
  if (provRes.status === 'fulfilled') providers.value = provRes.value
  if (proxyRes.status === 'fulfilled') {
    proxies.value = proxyRes.value.proxies
    proxyStrategy.value = proxyRes.value.strategy
  }
  if (cacheRes.status === 'fulfilled') cache.value = cacheRes.value
  loading.value = false
}

onMounted(loadAll)

async function quickPublish(item: WallpaperItem) {
  try {
    await api.updateWallpaperStatus(item.id, 'published')
    pendingItems.value = pendingItems.value.filter((x) => x.id !== item.id)
    pendingTotal.value = Math.max(0, pendingTotal.value - 1)
    publishedTotal.value++
    toast('已上架公开')
  } catch (e) {
    toast(e instanceof Error ? e.message : '上架失败')
  }
}

async function toggleXiaobanAi(nextVal: boolean) {
  if (!aiCatalogue.value || aiBusy.value) return
  aiBusy.value = true
  try {
    const payload = JSON.parse(JSON.stringify(aiCatalogue.value)) as AiCatalogue
    payload.xiaobanAi = { enabled: nextVal }
    aiCatalogue.value = await api.saveAiCatalogue(payload)
    toast(nextVal ? '已开启“小伴 AI”入口' : '已关闭“小伴 AI”入口')
  } catch (e) {
    toast(e instanceof Error ? e.message : '切换失败')
  } finally {
    aiBusy.value = false
  }
}

function goSource(id: string) {
  router.push({ path: '/admin/keys', query: { source: id } })
}
</script>

<template>
  <div class="xb-page">
    <header class="xb-page-head">
      <h1 class="xb-page-title">运营总览</h1>
      <div class="xb-page-actions">
        <RouterLink to="/admin/collector" class="xb-btn-ghost">
          <Icon name="compass" :size="14" /><span>采集壁纸</span>
        </RouterLink>
        <RouterLink to="/admin/upload" class="xb-btn-ghost">
          <Icon name="upload" :size="14" /><span>自定义上架</span>
        </RouterLink>
        <RouterLink to="/admin/apps" class="xb-btn-primary">
          <Icon name="plus" :size="14" /><span>发布版本</span>
        </RouterLink>
      </div>
    </header>

    <div class="xb-kpi-grid">
      <button type="button" class="xb-kpi-card is-clickable" title="前往壁纸图库" @click="router.push('/admin/wallpapers')">
        <div class="xb-kpi-top"><span>已上架壁纸</span><Icon name="image" :size="15" /></div>
        <div class="xb-kpi-val xb-num">{{ loading ? '—' : publishedTotal }}</div>
        <div class="xb-kpi-foot">
          <span>视频 {{ videoTotal }}</span>
          <span>·</span>
          <span v-if="pendingTotal > 0" class="xb-pill is-warn">{{ pendingTotal }} 待审</span>
          <span v-else class="xb-pill is-ok">已清零</span>
        </div>
      </button>

      <button type="button" class="xb-kpi-card is-clickable" title="管理客户端版本发布" @click="router.push('/admin/apps')">
        <div class="xb-kpi-top"><span>线上最新版本</span><Icon name="package" :size="15" /></div>
        <div class="xb-kpi-val xb-num">{{ latestPublished ? `v${latestPublished.version}` : '—' }}</div>
        <div class="xb-kpi-foot">
          <span v-if="latestPublished" class="xb-pill is-ok">
            <i class="dot" />{{ Object.keys(latestPublished.platforms).length }}/4 平台
          </span>
          <span v-if="draftReleases">{{ draftReleases }} 草稿</span>
        </div>
      </button>

      <button type="button" class="xb-kpi-card is-clickable" title="管理官网与多端公告" @click="router.push('/admin/desktop')">
        <div class="xb-kpi-top"><span>生效中公告</span><Icon name="megaphone" :size="15" /></div>
        <div class="xb-kpi-val xb-num">
          {{ activeNotices }}<span class="unit">条</span>
        </div>
        <div class="xb-kpi-foot"><span>官网 · 桌面 · Android</span></div>
      </button>

      <button type="button" class="xb-kpi-card is-clickable" title="查看搜索缓存命中与容量" @click="router.push('/admin/cache')">
        <div class="xb-kpi-top"><span>缓存命中率</span><Icon name="database" :size="15" /></div>
        <div class="xb-kpi-val xb-num">{{ cache ? formatPercent(cache.hit_rate) : '—' }}</div>
        <div class="xb-kpi-foot">
          <span v-if="cache">{{ cache.stats.entries }}/{{ cache.settings.max_entries }} 条 · {{ formatBytes(cache.stats.bytes) }}</span>
          <span v-else>等待加载</span>
        </div>
      </button>
    </div>

    <div class="xb-split-2">
      <div class="col-stack">
        <section>
          <div class="xb-sec-head">
            <h2 class="xb-sec-title">
              待审核壁纸
              <span v-if="pendingTotal > 0" class="xb-pill is-warn">{{ pendingTotal }}</span>
            </h2>
            <RouterLink :to="{ path: '/admin/wallpapers', query: { status: 'pending' } }" class="xb-btn-ghost is-sm">
              全部
            </RouterLink>
          </div>
          <div class="xb-group">
            <div v-if="!pendingItems.length" class="empty-row">
              {{ loading ? '加载中…' : '暂无待审核壁纸' }}
            </div>
            <div v-for="w in pendingItems" :key="w.id" class="xb-row">
              <div class="xb-row-main">
                <div class="thumb">
                  <img :src="w.preview_url" :alt="w.title" loading="lazy" />
                </div>
                <div class="info">
                  <div class="xb-row-title truncate">{{ w.title }}</div>
                  <div class="xb-row-meta">
                    <span>{{ w.media_type === 'video' ? '视频' : '图片' }}</span>
                    <span>·</span>
                    <span class="xb-num">{{ w.resolution.width }}×{{ w.resolution.height }}</span>
                    <span>·</span>
                    <span>{{ catName(w.category) }}</span>
                    <span>·</span>
                    <span>{{ w.source_type === 'custom' ? '自主上架' : w.provider }}</span>
                  </div>
                </div>
              </div>
              <div class="xb-row-actions">
                <RouterLink :to="{ path: '/admin/wallpapers', query: { status: 'pending' } }" class="xb-btn-ghost is-sm">
                  查看
                </RouterLink>
                <button type="button" class="xb-btn-primary is-sm" @click="quickPublish(w)">上架</button>
              </div>
            </div>
          </div>
        </section>

        <section>
          <div class="xb-sec-head">
            <h2 class="xb-sec-title">
              AI 模型目录
              <span v-if="aiCatalogue" class="xb-pill">修订 #{{ aiCatalogue.revision }}</span>
            </h2>
            <RouterLink to="/admin/ai-providers" class="xb-btn-ghost is-sm">管理</RouterLink>
          </div>
          <div class="xb-group">
            <div class="xb-row" title="桌面端排首位的官方额度服务入口，关闭后客户端下次刷新将隐藏">
              <div class="xb-row-main">
                <span class="xb-pill is-accent"><Icon name="sparkles" :size="12" /><span>小伴 AI 入口</span></span>
                <span class="sub-txt">{{ xiaobanAiOn ? '桌面端展示中' : '已隐藏' }}</span>
              </div>
              <ToggleSwitch
                :model-value="xiaobanAiOn"
                label="在桌面端显示小伴 AI 入口"
                :disabled="aiBusy || !aiCatalogue"
                @update:model-value="toggleXiaobanAi"
              />
            </div>
            <div v-if="aiCatalogue?.providers.length" class="xb-row tags-row">
              <span
                v-for="p in aiCatalogue.providers"
                :key="p.id"
                class="xb-pill"
                :class="{ 'is-ok': p.enabled }"
                :title="p.endPoint"
              >
                <i class="dot" />{{ p.provider }}
              </span>
            </div>
          </div>
        </section>
      </div>

      <div class="col-stack">
        <section>
          <div class="xb-sec-head">
            <h2 class="xb-sec-title">图源状态</h2>
            <RouterLink to="/admin/keys" class="xb-btn-ghost is-sm">配置</RouterLink>
          </div>
          <div class="xb-group">
            <div v-if="!providers.length" class="empty-row">{{ loading ? '加载中…' : '暂无图源配置' }}</div>
            <div
              v-for="s in providers"
              :key="s.id"
              class="xb-row is-click"
              @click="goSource(s.id)"
            >
              <div class="info">
                <div class="xb-row-title">
                  <span>{{ s.name }}</span>
                  <span v-if="s.use_proxy" class="xb-pill is-info">代理</span>
                </div>
                <div class="xb-row-meta">
                  <span>{{ s.keys.length ? `${s.keys.filter((k) => k.enabled).length}/${s.keys.length} 密钥` : '免密/未配' }}</span>
                  <span>·</span>
                  <span>{{ s.hotlink.image.policy === 'copy' ? '本站副本' : '直链优先' }}</span>
                </div>
              </div>
              <span v-if="s.keys.some((k) => k.cooldown_secs)" class="xb-pill is-warn"><i class="dot" />冷却中</span>
              <span v-else-if="s.keys.some((k) => k.enabled) || s.id === 'bing'" class="xb-pill is-ok"><i class="dot" />正常</span>
              <span v-else class="xb-pill">停用</span>
            </div>
          </div>
        </section>

        <section>
          <div class="xb-sec-head">
            <h2 class="xb-sec-title">代理池</h2>
            <RouterLink to="/admin/proxies" class="xb-btn-ghost is-sm">测速</RouterLink>
          </div>
          <div class="xb-group xb-group-pad proxy-box">
            <div class="proxy-head">
              <span class="sub-txt">{{ proxyStrategy === 'round_robin' ? '顺序轮询' : '随机挑选' }}</span>
              <strong class="xb-num">{{ activeProxies }} / {{ proxies.length }} 在线</strong>
            </div>
            <div class="xb-meter"><i :style="{ width: `${proxyPct}%` }" /></div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.unit {
  font-size: 13px;
  font-weight: 500;
  color: var(--ui-text-tertiary);
}
.col-stack {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.empty-row {
  padding: 28px 16px;
  text-align: center;
  color: var(--ui-text-tertiary);
  font-size: 13px;
}
.thumb {
  width: 72px;
  height: 46px;
  border-radius: 7px;
  overflow: hidden;
  flex: none;
  background: var(--ui-control);
}
.thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.info { min-width: 0; flex: 1; }
.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sub-txt {
  font-size: 12.5px;
  color: var(--ui-text-secondary);
}
.tags-row {
  gap: 6px;
  justify-content: flex-start;
}
.is-click {
  cursor: pointer;
  transition: background 0.14s;
}
.is-click:hover {
  background: var(--ui-surface-hover);
}
.proxy-box {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.proxy-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 13px;
}
</style>
