<script setup lang="ts">
import { ref } from 'vue'
import * as api from '@/admin/api'
import type { HotlinkPolicyValue, HotlinkSetting, ProviderConfig } from '@/admin/types'
import { toast } from '@/composables/toast'

const props = defineProps<{ provider: ProviderConfig }>()
const emit = defineEmits<{ changed: [] }>()

type Choice = HotlinkPolicyValue | 'default'
const busy = ref(false)

const KINDS = [
  { key: 'image', label: '图片' },
  { key: 'video', label: '视频' },
] as const

const POLICY_TEXT: Record<HotlinkPolicyValue, string> = {
  source: '直链优先',
  copy: '本站副本',
}

function current(setting: HotlinkSetting): Choice {
  return setting.custom ? setting.policy : 'default'
}

async function change(kind: 'image' | 'video', value: Choice) {
  if (busy.value) return
  busy.value = true
  try {
    await api.setProviderHotlink(props.provider.id, { [kind]: value })
    toast('素材链接策略已保存')
  } catch (e) {
    toast(e instanceof Error ? e.message : '保存失败')
  } finally {
    busy.value = false
    emit('changed')
  }
}
</script>

<template>
  <div class="hotlink" role="group" :aria-label="`${provider.name} 素材链接策略`">
    <div class="hotlink-text">
      <strong>素材链接</strong>
      <p class="a-hint">
        直链优先：客户端先使用来源原始链接，失败时才用本站副本，采集时不下载素材。本站副本：由本站保存并提供。
      </p>
    </div>
    <label v-for="kind in KINDS" :key="kind.key" class="a-field">
      {{ kind.label }}
      <select
        class="a-select"
        :value="current(provider.hotlink[kind.key])"
        :disabled="busy"
        @change="change(kind.key, ($event.target as HTMLSelectElement).value as Choice)"
      >
        <option value="default">默认（{{ POLICY_TEXT[provider.hotlink[kind.key].default] }}）</option>
        <option value="source">{{ POLICY_TEXT.source }}</option>
        <option value="copy">{{ POLICY_TEXT.copy }}</option>
      </select>
    </label>
  </div>
</template>

<style scoped>
.hotlink {
  display: grid;
  grid-template-columns: minmax(0, 2fr) repeat(2, minmax(140px, 1fr));
  gap: 12px 16px;
  align-items: end;
}

.hotlink-text p {
  margin-top: 2px;
}

@media (max-width: 720px) {
  .hotlink {
    grid-template-columns: 1fr;
  }
}
</style>
