<script setup lang="ts">
import { ref } from 'vue'
import * as api from '@/admin/api'
import type { HotlinkPolicyValue, HotlinkSetting, ProviderConfig } from '@/admin/types'
import AdminSelect, { type SelectOption } from '@/components/admin/AdminSelect.vue'
import { toast } from '@/composables/toast'

const props = defineProps<{ provider: ProviderConfig }>()
const emit = defineEmits<{ changed: [] }>()

type Choice = HotlinkPolicyValue | 'default'
const busy = ref(false)

const KINDS = [
  { key: 'image', label: '图片：' },
  { key: 'video', label: '视频：' },
] as const

const POLICY_TEXT: Record<HotlinkPolicyValue, string> = {
  source: '直链优先',
  copy: '本站副本',
}

function current(setting: HotlinkSetting): Choice {
  return setting.custom ? setting.policy : 'default'
}

function optionsFor(setting: HotlinkSetting): SelectOption[] {
  return [
    { value: 'default', label: `默认 (${POLICY_TEXT[setting.default]})`, desc: '恢复系统默认链接策略' },
    { value: 'source', label: '直链优先', dot: 'ok', desc: '客户端优先使用来源直链，采集时不下载素材' },
    { value: 'copy', label: '本站副本', dot: 'info', desc: '采集时保存副本到本站服务器，直链失效时兜底' },
  ]
}

async function change(kind: 'image' | 'video', value: string) {
  if (busy.value) return
  busy.value = true
  try {
    await api.setProviderHotlink(props.provider.id, { [kind]: value as Choice })
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
  <div
    class="xb-row"
    role="group"
    :aria-label="`${provider.name} 素材链接策略`"
    title="直链优先可节省存储与带宽，本站副本由服务器保存并在直链失效时兜底"
  >
    <span class="label">素材链接策略</span>
    <div class="controls">
      <AdminSelect
        v-for="kind in KINDS"
        :key="kind.key"
        :model-value="current(provider.hotlink[kind.key])"
        :options="optionsFor(provider.hotlink[kind.key])"
        :prefix="kind.label"
        :disabled="busy"
        sm
        @update:model-value="change(kind.key, $event)"
      />
    </div>
  </div>
</template>

<style scoped>
.label {
  font-weight: 550;
}
.controls {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
