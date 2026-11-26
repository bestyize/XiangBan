<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import * as api from '@/admin/api'
import { formatBytes, formatDuration, formatPercent, splitTtl, TTL_UNITS } from '@/admin/format'
import type { CacheOverview } from '@/admin/types'
import ConfirmButton from '@/components/admin/ConfirmButton.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
import Icon from '@/components/Icon.vue'
import { toast } from '@/composables/toast'

const overview = ref<CacheOverview | null>(null)
const loading = ref(true)
const loadError = ref<string | null>(null)

const form = reactive({ enabled: true, ttlValue: 1, ttlUnit: 3600, maxEntries: 1000 })
const saving = ref(false)
const saveError = ref<string | null>(null)
const dirty = ref(false)

let refreshTimer: ReturnType<typeof setInterval> | undefined

function fillForm(data: CacheOverview) {
  const { value, unit } = splitTtl(data.settings.ttl_secs)
  form.enabled = data.settings.enabled
  form.ttlValue = value
  form.ttlUnit = unit
  form.maxEntries = data.settings.max_entries
  dirty.value = false
}

async function load(refillForm = false) {
  try {
    overview.value = await api.getCache()
    if (refillForm || !dirty.value) fillForm(overview.value)
    loadError.value = null
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : '加载失败'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void load(true)
  refreshTimer = setInterval(() => void load(false), 10_000)
})
onBeforeUnmount(() => clearInterval(refreshTimer))

const ttlSecs = computed(() => Math.round(form.ttlValue * form.ttlUnit))
const hitPct = computed(() => Math.round((overview.value?.hit_rate ?? 0) * 100))
const capPct = computed(() => {
  if (!overview.value || !overview.value.settings.max_entries) return 0
  return Math.min(100, Math.round((overview.value.stats.entries / overview.value.settings.max_entries) * 100))
})

async function save() {
  if (saving.value) return
  saving.value = true
  saveError.value = null
  try {
    overview.value = await api.saveCacheSettings({
      enabled: form.enabled,
      ttl_secs: ttlSecs.value,
      max_entries: Math.round(form.maxEntries),
    })
    fillForm(overview.value)
    toast('缓存设置已保存')
  } catch (e) {
    saveError.value = e instanceof Error ? e.message : '保存失败'
  } finally {
    saving.value = false
  }
}

async function clear() {
  try {
    const { cleared } = await api.clearCache()
    toast(cleared ? `已清空 ${cleared} 条缓存` : '缓存本来就是空的')
  } catch (e) {
    toast(e instanceof Error ? e.message : '清空失败')
  }
  await load(false)
}
</script>

<template>
  <div class="xb-page is-narrow">
    <header class="xb-page-head">
      <h1 class="xb-page-title">搜索缓存</h1>
      <div class="xb-page-actions">
        <ConfirmButton label="清空缓存" confirm-label="确认清空" :busy="!overview" @confirm="clear" />
        <button type="button" class="xb-btn-primary" :disabled="saving || !dirty" @click="save">
          {{ saving ? '保存中…' : '保存设置' }}
        </button>
      </div>
    </header>

    <div v-if="loadError" class="xb-alert is-danger" role="alert">
      <div class="xb-alert-l"><Icon name="alert" :size="15" /><span>{{ loadError }}</span></div>
      <button type="button" class="xb-btn-ghost is-sm" @click="load(true)">重试</button>
    </div>

    <div v-if="loading && !overview" class="xb-group xb-empty-box">加载中…</div>

    <template v-if="overview">
      <div class="xb-kpi-grid">
        <div class="xb-kpi-card">
          <div class="xb-kpi-top"><span>命中率</span></div>
          <div class="xb-kpi-val xb-num">{{ formatPercent(overview.hit_rate) }}</div>
          <div class="xb-meter"><i :style="{ width: `${hitPct}%` }" /></div>
        </div>
        <div class="xb-kpi-card">
          <div class="xb-kpi-top"><span>条目容量</span></div>
          <div class="xb-kpi-val xb-num">
            {{ overview.stats.entries }}<span class="sub-u">/{{ overview.settings.max_entries }}</span>
          </div>
          <div class="xb-meter"><i :style="{ width: `${capPct}%` }" /></div>
        </div>
        <div class="xb-kpi-card">
          <div class="xb-kpi-top"><span>占用空间</span></div>
          <div class="xb-kpi-val xb-num">{{ formatBytes(overview.stats.bytes) }}</div>
          <div class="xb-kpi-foot"><span>PostgreSQL 持久化</span></div>
        </div>
        <div class="xb-kpi-card">
          <div class="xb-kpi-top"><span>最早条目</span></div>
          <div class="xb-kpi-val" style="font-size: 18px">
            {{ overview.stats.oldest_age_secs === null ? '—' : formatDuration(overview.stats.oldest_age_secs) }}
          </div>
          <div class="xb-kpi-foot xb-num">命中 {{ overview.stats.hits }} / 回源 {{ overview.stats.misses }}</div>
        </div>
      </div>

      <p v-if="saveError" class="xb-error" role="alert">{{ saveError }}</p>

      <div class="xb-group">
        <div class="xb-row" title="关闭后每次检索都实时回源请求供应商">
          <span style="font-weight: 550">启用搜索缓存</span>
          <ToggleSwitch
            v-model="form.enabled"
            label="启用搜索缓存"
            @update:model-value="dirty = true"
          />
        </div>

        <div class="xb-row" title="超过有效期不再命中，最长 30 天">
          <span style="font-weight: 550">有效期</span>
          <div class="ctrl-row">
            <div class="xb-stepper">
              <button
                type="button"
                aria-label="减少"
                @click="form.ttlValue = Math.max(1, form.ttlValue - 1); dirty = true"
              >
                <Icon name="minus" :size="14" />
              </button>
              <input v-model.number="form.ttlValue" class="xb-num" type="number" min="1" @input="dirty = true" />
              <button
                type="button"
                aria-label="增加"
                @click="form.ttlValue += 1; dirty = true"
              >
                <Icon name="plus" :size="14" />
              </button>
            </div>
            <div class="xb-seg">
              <button
                v-for="u in TTL_UNITS"
                :key="u.secs"
                type="button"
                :class="{ 'is-active': form.ttlUnit === u.secs }"
                @click="form.ttlUnit = u.secs; dirty = true"
              >
                {{ u.label }}
              </button>
            </div>
          </div>
        </div>

        <div class="xb-row" title="超过上限时优先淘汰最久未命中的缓存条目">
          <span style="font-weight: 550">最大条数</span>
          <div class="xb-stepper">
            <button
              type="button"
              aria-label="减少 100"
              @click="form.maxEntries = Math.max(100, form.maxEntries - 100); dirty = true"
            >
              <Icon name="minus" :size="14" />
            </button>
            <input
              v-model.number="form.maxEntries"
              class="xb-num"
              style="width: 84px"
              type="number"
              min="1"
              max="100000"
              @input="dirty = true"
            />
            <button
              type="button"
              aria-label="增加 100"
              @click="form.maxEntries = Math.min(100000, form.maxEntries + 100); dirty = true"
            >
              <Icon name="plus" :size="14" />
            </button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.sub-u {
  font-size: 13px;
  font-weight: 500;
  color: var(--ui-text-tertiary);
}
.ctrl-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
