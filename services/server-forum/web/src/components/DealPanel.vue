<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DealStatus, TopicDetail } from '@/api/types'
import DealState from './DealState.vue'
import { api } from '@/api/forum'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { DEAL_STATUS_LABEL, DEAL_TYPE_LABEL, REGION_LABEL, conditionLabels, expiryText, formatDateTime } from '@/lib/format'

const props = defineProps<{ topic: TopicDetail }>()
const session = useSession()
const toast = useToast()

const deal = computed(() => props.topic.deal!)
const conditions = computed(() => conditionLabels(deal.value.conditions))
const valueText = computed(() => deal.value.value_text.trim() || DEAL_TYPE_LABEL[deal.value.deal_type])
const conditionText = computed(() => (conditions.value.length ? conditions.value.join('、') : '无特殊条件'))
const busy = ref(false)

async function vote(value: 'valid' | 'invalid') {
  if (!session.requireLogin()) return
  if (!props.topic.viewer.can_vote) {
    toast.show('成为成员（信任等级 1）后才能投票', 'error')
    return
  }
  busy.value = true
  const next = props.topic.viewer.vote === (value === 'valid' ? 1 : -1) ? 'none' : value
  try {
    const res = await api.vote(props.topic.id, next)
    Object.assign(props.topic.deal!, res)
    props.topic.viewer.vote = res.vote
    if (next !== 'none') toast.show(value === 'valid' ? '谢谢确认，这条羊毛还能领' : '已记录，失效票多了会提醒其他人')
  } catch (e) {
    toast.error(e)
  } finally {
    busy.value = false
  }
}

async function mark(status: DealStatus) {
  busy.value = true
  try {
    const res = await api.setDealStatus(props.topic.id, status)
    props.topic.deal!.deal_status = res.deal_status
    toast.show(`已标记为${DEAL_STATUS_LABEL[status]}`)
  } catch (e) {
    toast.error(e)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <section class="deal" :class="`is-${deal.deal_status}`" aria-label="羊毛信息">
    <dl class="facts">
      <div>
        <dt>产品</dt>
        <dd><RouterLink :to="`/p/${deal.product.slug}`" class="link">{{ deal.product.name }}</RouterLink></dd>
      </div>
      <div class="gain">
        <dt>能领到</dt>
        <dd>{{ valueText }}</dd>
      </div>
      <div>
        <dt>类型</dt>
        <dd>{{ DEAL_TYPE_LABEL[deal.deal_type] }}</dd>
      </div>
      <div>
        <dt>地区</dt>
        <dd>{{ REGION_LABEL[deal.region] }}</dd>
      </div>
      <div>
        <dt>领取条件</dt>
        <dd>{{ conditionText }}</dd>
      </div>
      <div>
        <dt>有效期</dt>
        <dd>
          {{ expiryText(deal.expires_at, deal.long_term) }}
          <span v-if="deal.expires_at && !deal.long_term" class="muted">（{{ formatDateTime(deal.expires_at) }}）</span>
        </dd>
      </div>
      <div>
        <dt>状态</dt>
        <dd><DealState :deal="deal" verbose /></dd>
      </div>
    </dl>

    <div class="check">
      <span class="ask">还能领吗？</span>
      <div class="votes" role="group" aria-label="反馈这条羊毛是否还能领">
        <button
          type="button"
          class="vote yes"
          :class="{ on: topic.viewer.vote === 1 }"
          :aria-pressed="topic.viewer.vote === 1"
          :disabled="busy || deal.deal_status === 'expired'"
          @click="vote('valid')"
        >
          我刚领到了
        </button>
        <button
          type="button"
          class="vote no"
          :class="{ on: topic.viewer.vote === -1 }"
          :aria-pressed="topic.viewer.vote === -1"
          :disabled="busy || deal.deal_status === 'expired'"
          @click="vote('invalid')"
        >
          领不到了
        </button>
      </div>
      <a
        v-if="deal.link_url"
        :href="deal.link_url"
        target="_blank"
        rel="nofollow noopener noreferrer"
        class="go"
      >
        打开活动页
      </a>
    </div>

    <div v-if="topic.viewer.can_moderate" class="mod">
      <span class="muted">守护者操作</span>
      <button v-for="s in ['valid', 'suspect', 'expired'] as const" :key="s" type="button" class="btn btn-sm" :disabled="busy || deal.deal_status === s" @click="mark(s)">
        标为{{ DEAL_STATUS_LABEL[s] }}
      </button>
    </div>
  </section>
</template>

<style scoped>
/* 帖子顶部的几行信息，就是正文前面的普通文字，不画框。 */
.deal {
  margin-bottom: 16px;
  padding-bottom: 14px;
  border-bottom: 1px solid var(--line);
}
.facts {
  margin: 0 0 12px;
  display: grid;
  gap: 4px;
  font-size: var(--fs-md);
}
.facts > div {
  display: grid;
  grid-template-columns: 5.5em minmax(0, 1fr);
  gap: 12px;
  align-items: baseline;
}
dt {
  color: var(--ink-3);
  font-size: var(--fs-sm);
}
dd {
  margin: 0;
  overflow-wrap: anywhere;
}
.link {
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-thickness: 1px;
}
.gain dd {
  font-weight: 700;
}
.check {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px 12px;
}
.ask {
  font-size: var(--fs-sm);
  font-weight: 600;
}
.votes {
  display: flex;
  gap: 8px;
}
.vote {
  height: 32px;
  padding: 0 12px;
  border-radius: var(--r-sm);
  border: 1px solid var(--line-strong);
  background: transparent;
  font-size: var(--fs-sm);
  font-weight: 600;
}
.vote.yes:hover:not(:disabled),
.vote.yes.on {
  border-color: var(--valid);
  color: var(--valid);
  background: var(--valid-soft);
}
.vote.no:hover:not(:disabled),
.vote.no.on {
  border-color: var(--suspect);
  color: var(--suspect);
  background: var(--suspect-soft);
}
.vote:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.go {
  margin-left: auto;
  font-size: var(--fs-sm);
  font-weight: 700;
  text-decoration: underline;
  text-underline-offset: 3px;
}
.mod {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
  font-size: var(--fs-xs);
  margin-top: 12px;
}
.is-expired .gain dd {
  color: var(--ink-3);
  text-decoration: line-through;
}
@media (max-width: 767px) {
  .facts > div {
    grid-template-columns: 5em minmax(0, 1fr);
    gap: 8px;
  }
  .go {
    flex-basis: 100%;
    margin-left: 0;
  }
}
</style>
