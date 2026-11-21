<script setup lang="ts">
import { computed, ref } from 'vue'
import AppIcon from '@/components/AppIcon.vue'

const props = defineProps<{ limit: number; locked?: boolean; busy?: boolean; uploading?: boolean; replyTo?: string }>()
const emit = defineEmits<{ submit: [body: string]; expand: []; cancelReply: [] }>()
const draft = defineModel<string>({ required: true })
const input = ref<HTMLInputElement>()
const tooLong = computed(() => Array.from(draft.value.trim()).length > props.limit)
function submit() {
  const text = draft.value.trim()
  if (text && !tooLong.value && !props.locked && !props.busy && !props.uploading) emit('submit', text)
}
defineExpose({ focus: () => input.value?.focus() })
</script>

<template>
  <form class="xhs-pc-form" @submit.prevent="submit">
    <div class="xhs-pc-input-wrap">
      <button class="comment-input-tool" type="button" aria-label="打开完整评论编辑" :disabled="locked" @click="$emit('expand')">
        <AppIcon name="edit" :size="14" />
      </button>
      <input ref="input" v-model="draft" class="comment-input" type="text" aria-label="评论内容"
        :placeholder="locked ? '暂不可评论' : replyTo ? `回复 @${replyTo}…` : '说点什么…'"
        :disabled="locked" :readonly="busy" :maxlength="limit * 2" :aria-invalid="tooLong || undefined" required />
      <button v-if="replyTo" class="comment-input-tool" type="button" aria-label="取消回复" :disabled="busy" @click="$emit('cancelReply')">
        <AppIcon name="close" :size="14" />
      </button>
    </div>
    <button class="xhs-send-btn is-ready" type="submit" :disabled="locked || busy || uploading || tooLong">
      {{ busy ? '发送中…' : '发送' }}
    </button>
  </form>
</template>

<style scoped>
.comment-input-tool { display: inline-grid; place-items: center; padding: 0; border: 0; background: transparent; color: var(--ink-3); }
.xhs-pc-input-wrap .comment-input:focus-visible { outline: 2px solid var(--focus); outline-offset: 1px; }
.comment-input-tool:disabled, .xhs-send-btn:disabled { opacity: .5; cursor: default; }
</style>
