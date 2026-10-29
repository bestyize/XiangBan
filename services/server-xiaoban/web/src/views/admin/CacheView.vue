<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'
import * as api from '@/admin/api'
import { formatBytes, formatDuration, formatPercent, splitTtl, TTL_UNITS } from '@/admin/format'
import type { CacheOverview } from '@/admin/types'
import ConfirmButton from '@/components/admin/ConfirmButton.vue'
import ToggleSwitch from '@/components/admin/ToggleSwitch.vue'
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
  load(true)
  // Only the figures move on their own; edits in progress are left alone.
  refreshTimer = setInterval(() => load(false), 10_000)
})

onBeforeUnmount(() => clearInterval(refreshTimer))

const ttlSecs = computed(() => Math.round(form.ttlValue * form.ttlUnit))

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
  <div class="a-page">
    <header class="a-head">
      <h1 class="display">搜索缓存</h1>
      <p>
        完全相同的搜索（关键词、分类、画幅、来源、页码都一样）会直接返回已保存的结果，不再请求供应商，
        用来节省它们的调用额度。结果保存在 PostgreSQL 里，重启服务后仍然有效。
      </p>
    </header>

    <p v-if="loading" class="a-note">加载中…</p>
    <p v-else-if="loadError" class="a-error" role="alert">
      {{ loadError }}
      <button class="a-link" type="button" @click="load(true)">重试</button>
    </p>

    <template v-if="overview">
      <section class="a-section" aria-label="缓存状态">
        <dl class="a-figures">
          <div class="a-figure">
            <dt>命中率</dt>
            <dd>{{ formatPercent(overview.hit_rate) }}</dd>
          </div>
          <div class="a-figure">
            <dt>已缓存 / 上限</dt>
            <dd>{{ overview.stats.entries }} / {{ overview.settings.max_entries }}</dd>
          </div>
          <div class="a-figure">
            <dt>占用空间</dt>
            <dd>{{ formatBytes(overview.stats.bytes) }}</dd>
          </div>
          <div class="a-figure">
            <dt>最早一条已保存</dt>
            <dd>{{ overview.stats.oldest_age_secs === null ? '—' : formatDuration(overview.stats.oldest_age_secs) }}</dd>
          </div>
        </dl>
        <p class="a-small">
          服务启动以来：{{ overview.stats.hits }} 次直接用缓存，{{ overview.stats.misses }} 次去请求了供应商。
        </p>
      </section>

      <section class="a-section" aria-label="缓存设置">
        <div class="a-section-head">
          <h2 class="display">设置</h2>
        </div>

        <form class="a-form" @submit.prevent="save" @input="dirty = true" @change="dirty = true">
          <label class="switch-line">
            <span>
              启用缓存
              <span class="a-hint">关闭后每次搜索都会请求供应商，已保存的结果也不会被使用。</span>
            </span>
            <ToggleSwitch v-model="form.enabled" label="启用缓存" @update:model-value="dirty = true" />
          </label>

          <label class="a-field">
            有效期
            <span class="a-unit">
              <input v-model.number="form.ttlValue" class="a-input" type="number" min="1" step="1" required />
              <select v-model.number="form.ttlUnit" class="a-select" aria-label="有效期单位">
                <option v-for="unit in TTL_UNITS" :key="unit.secs" :value="unit.secs">{{ unit.label }}</option>
              </select>
            </span>
            <span class="a-hint">
              超过这个时间的结果不再使用。最长 30 天。修改后立刻对已缓存的结果生效。
            </span>
          </label>

          <label class="a-field">
            最多保存的条数
            <input v-model.number="form.maxEntries" class="a-input" type="number" min="1" max="100000" step="1" required />
            <span class="a-hint">超过时，先淘汰最久没被用到的结果。每条约 20 到 60 KB。</span>
          </label>

          <p v-if="saveError" class="a-error" role="alert">{{ saveError }}</p>
          <div class="a-actions">
            <button class="a-btn primary" type="submit" :disabled="saving || !dirty">
              {{ saving ? '保存中…' : '保存设置' }}
            </button>
          </div>
        </form>
      </section>

      <section class="a-section" aria-label="清空缓存">
        <div class="a-section-head">
          <div>
            <h2 class="display">清空</h2>
            <p class="a-sub">立刻删除所有已保存的搜索结果。添加、删除或停用密钥时也会自动清空。</p>
          </div>
          <ConfirmButton label="清空缓存" confirm-label="确认清空" @confirm="clear" />
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.switch-line {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
  font-size: 14px;
}

.switch-line > span {
  display: grid;
  gap: 2px;
}
</style>
