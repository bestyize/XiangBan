<script setup lang="ts">
import { ref, watch } from 'vue'
import ModalDialog from './ModalDialog.vue'
import { api } from '@/api/forum'
import { useToast } from '@/stores/toast'

const props = defineProps<{ target: { type: 'topic' | 'post'; id: number } | null }>()
const emit = defineEmits<{ close: [] }>()
const toast = useToast()

const REASONS = [
  { key: 'spam', label: '垃圾广告或私发推广' },
  { key: 'account_trade', label: '分享或交易 API Key、账号' },
  { key: 'abuse', label: '人身攻击或开盒' },
  { key: 'politics', label: '引战或时政' },
  { key: 'copyright', label: '侵犯版权' },
  { key: 'illegal', label: '违法违规' },
  { key: 'other', label: '其他' },
]

const reason = ref('')
const detail = ref('')
const busy = ref(false)

watch(
  () => props.target,
  () => {
    reason.value = ''
    detail.value = ''
  },
)

async function submit() {
  if (!props.target || !reason.value) return
  busy.value = true
  try {
    await api.report(props.target.type, props.target.id, reason.value, detail.value)
    toast.show('已提交举报，管理员会尽快处理')
    emit('close')
  } catch (e) {
    toast.error(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <ModalDialog :open="!!target" :title="target?.type === 'post' ? '举报这条回复' : '举报这个帖子'" @close="emit('close')">
    <form class="form" @submit.prevent="submit">
      <fieldset class="reasons">
        <legend class="sr-only">举报原因</legend>
        <label v-for="r in REASONS" :key="r.key" class="reason" :class="{ on: reason === r.key }">
          <input v-model="reason" type="radio" name="reason" :value="r.key" />
          <span>{{ r.label }}</span>
        </label>
      </fieldset>
      <label class="field">
        <span>补充说明（可选）</span>
        <textarea v-model="detail" class="input" rows="3" maxlength="500" placeholder="比如是哪一部分违规" />
      </label>
      <div class="actions">
        <button type="button" class="btn" @click="emit('close')">取消</button>
        <button type="submit" class="btn btn-primary" :disabled="!reason || busy">提交举报</button>
      </div>
    </form>
  </ModalDialog>
</template>

<style scoped>
.form {
  display: grid;
  gap: 14px;
}
.reasons {
  border: 0;
  padding: 0;
  margin: 0;
  display: grid;
  gap: 6px;
}
.reason {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border: 1px solid var(--line);
  border-radius: var(--r-sm);
  font-size: var(--fs-sm);
  cursor: pointer;
}
.reason.on {
  border-color: var(--ink);
  background: var(--surface-2);
}
.reason input {
  accent-color: var(--ink);
}
textarea.input {
  resize: vertical;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
