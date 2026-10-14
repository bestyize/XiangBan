<script setup lang="ts">
import { ref, watch } from 'vue'
import { api } from '@/api/forum'
import type { InviteItem, PointsLedgerItem, PointsWallet } from '@/api/types'
import { useSession } from '@/stores/session'
import { useToast } from '@/stores/toast'
import { formatDateTime } from '@/lib/format'
import { inviteUrl } from '@/lib/invite'
import { useHead } from '@/lib/head'
import PagerNav from '@/components/PagerNav.vue'

useHead(() => ({ title: '积分与签到' }))
const session = useSession()
const toast = useToast()
const wallet = ref<PointsWallet | null>(null)
const rows = ref<PointsLedgerItem[]>([])
const invited = ref<InviteItem[]>([])
const tab = ref<'ledger' | 'invites'>('ledger')
const page = ref(1)
const size = ref(30)
const total = ref(0)
const busy = ref(false)
const loading = ref(false)
const failed = ref(false)
let seq = 0
const inviteStatus = { pending: '等待首次签到', rewarded: '已奖励 30 分', capped: '当日奖励名额已满', ineligible: '未满足奖励条件' }

async function load(p = 1) {
  const mine = ++seq
  loading.value = true
  failed.value = false
  try {
    const result = await api.points()
    if (mine !== seq) return
    wallet.value = result
    if (session.info) session.info.user.points = result.balance
    if (tab.value === 'ledger') {
      const list = await api.pointsLedger(p)
      if (mine !== seq) return
      rows.value = list.items
      page.value = list.page
      size.value = list.page_size
      total.value = list.total
    } else {
      const list = await api.invites(p)
      if (mine !== seq) return
      invited.value = list.items
      page.value = list.page
      size.value = list.page_size
      total.value = list.total
    }
  } catch (e) {
    if (mine === seq) { failed.value = true; toast.error(e) }
  } finally {
    if (mine === seq) loading.value = false
  }
}

watch([tab, () => session.user?.uid], () => {
  wallet.value = null
  rows.value = []
  invited.value = []
  total.value = 0
  page.value = 1
  load()
}, { immediate: true })

async function checkIn() {
  if (!session.user || session.restoring || busy.value || wallet.value?.checked_in_today) return
  busy.value = true
  try {
    const result = await api.checkIn()
    toast.show(result.earned ? `签到成功，获得 ${result.earned} 积分` : '今天已经签到')
    await load()
  } catch (e) { toast.error(e) }
  finally { busy.value = false }
}

async function copyInvite() {
  if (!wallet.value) return
  try {
    await navigator.clipboard.writeText(inviteUrl(wallet.value.invite_code))
    toast.show('邀请链接已复制')
  } catch { toast.show('无法自动复制，请长按下方链接复制', 'error') }
}
</script>

<template>
  <div class="points-page">
    <h1>积分与签到</h1>
    <p v-if="failed" class="panel empty"><button class="btn" type="button" @click="load(page)">重新加载</button></p>
    <template v-if="wallet">
      <section class="panel wallet">
        <div><span class="muted">可用积分</span><strong class="balance">{{ wallet.balance }}</strong></div>
        <button class="btn btn-primary" type="button" :disabled="busy || wallet.checked_in_today || session.restoring || !session.user" @click="checkIn">
          {{ busy ? '签到中…' : wallet.checked_in_today ? '今日已签到' : '签到 +1' }}
        </button>
        <p class="muted">今日访问 {{ wallet.visited_today ? '已领取 +1' : '待领取' }} · 本月已签到 {{ wallet.month_check_ins }} 天</p>
        <div class="milestones">
          <span :class="{ reached: wallet.month_check_ins >= 7 }">累计 7 天 +3 {{ wallet.month_check_ins >= 7 ? '✓' : '' }}</span>
          <span :class="{ reached: wallet.month_check_ins >= 20 }">累计 20 天 +5 {{ wallet.month_check_ins >= 20 ? '✓' : '' }}</span>
        </div>
      </section>
      <section class="panel invite">
        <h2>邀请朋友，一起发现好内容</h2>
        <p>新用户通过链接首次加入论坛并完成首次签到，你获得 30 积分。每天最多奖励 3 位，超出仍记录邀请人数，不补发奖励。</p>
        <div class="invite-row">
          <input class="input" readonly :value="inviteUrl(wallet.invite_code)" aria-label="我的邀请链接" @focus="($event.target as HTMLInputElement).select()" />
          <button type="button" class="btn" @click="copyInvite">复制链接</button>
        </div>
        <p class="muted">已邀请 {{ wallet.invites }} 人 · 已奖励 {{ wallet.rewarded_invites }} 人 · 累计获得 {{ wallet.invite_income }} 分</p>
      </section>
    </template>
    <section class="panel rules">
      <h2>积分怎么用、怎么赚</h2>
      <p>首次加入论坛 +50；每日访问 +1、签到 +1；帖子加精 +10，同帖只奖励一次。</p>
      <p>附件价格为 0～10 分，同一附件只在首次购买时扣分，积分全部给发布者。求助悬赏先暂存，采纳后给回答者，关闭或到期退还。</p>
      <p class="muted">每天按北京时间 00:00 更新。积分余额与信任等级独立。</p>
    </section>
    <div class="tabs" role="tablist">
      <button type="button" role="tab" :aria-selected="tab === 'ledger'" :class="{ on: tab === 'ledger' }" @click="tab = 'ledger'">积分明细</button>
      <button type="button" role="tab" :aria-selected="tab === 'invites'" :class="{ on: tab === 'invites' }" @click="tab = 'invites'">我的邀请</button>
    </div>
    <div class="panel history" :aria-busy="loading">
      <p v-if="loading" class="muted empty">加载中…</p>
      <template v-else-if="tab === 'ledger'">
        <div v-for="item in rows" :key="item.id" class="entry">
          <div class="detail"><strong>{{ item.description }}</strong><small>{{ formatDateTime(item.created_at) }} · 余额 {{ item.balance_after }}</small></div>
          <strong class="delta" :class="{ positive: item.delta > 0 }">{{ item.delta > 0 ? '+' : '' }}{{ item.delta }}</strong>
        </div>
        <p v-if="!rows.length && !failed" class="muted empty">还没有积分记录。</p>
      </template>
      <template v-else>
        <div v-for="(item, i) in invited" :key="i" class="entry">
          <div class="detail"><strong>{{ item.nickname }}</strong><small>{{ formatDateTime(item.created_at) }}</small></div>
          <span class="muted outcome">{{ inviteStatus[item.invite_status] }}</span>
        </div>
        <p v-if="!invited.length && !failed" class="muted empty">还没有邀请记录，分享你喜欢的帖子也能邀请朋友。</p>
      </template>
    </div>
    <PagerNav :page="page" :page-size="size" :total="total" @go="load" />
  </div>
</template>

<style scoped>
.points-page { max-width: 780px; margin: 0 auto; }
h1 { font-size: var(--fs-2xl); margin: 0 0 16px; }
h2 { font-size: var(--fs-md); margin: 0 0 8px; }
.wallet, .invite, .rules { padding: 18px; margin-bottom: 14px; }
.wallet { display: grid; grid-template-columns: 1fr auto; align-items: center; gap: 10px; }
.balance { display: block; font-size: 36px; font-family: var(--font-display); }
.wallet p, .milestones { grid-column: 1 / -1; }
p { font-size: var(--fs-sm); line-height: 1.7; margin: 4px 0 8px; }
.milestones { display: flex; gap: 12px; flex-wrap: wrap; font-size: var(--fs-xs); color: var(--ink-3); }
.reached { color: var(--brand); }
.invite-row { display: flex; gap: 8px; margin: 12px 0; }
.invite-row input { min-width: 0; flex: 1; font-size: var(--fs-xs); }
.invite-row button { flex: none; }
.tabs { display: flex; gap: 8px; border-bottom: 1px solid var(--line); margin: 20px 0 10px; }
.tabs button { border: 0; background: none; color: var(--ink-3); padding: 8px 10px; font: inherit; font-size: var(--fs-sm); }
.tabs .on { color: var(--ink); box-shadow: inset 0 -2px var(--brand); }
.entry { display: flex; align-items: center; gap: 12px; padding: 12px 16px; }
.entry + .entry { border-top: 1px solid var(--line); }
.detail { display: grid; gap: 4px; min-width: 0; flex: 1; font-size: var(--fs-sm); overflow-wrap: anywhere; }
.detail small, .outcome { color: var(--ink-3); font-size: var(--fs-xs); }
.delta { white-space: nowrap; font-variant-numeric: tabular-nums; }
.positive { color: var(--brand); }
.empty { padding: 28px 16px; text-align: center; }
@media(max-width:767px) { .wallet, .invite, .rules { padding: 14px; } h1 { font-size: var(--fs-xl); } .outcome { max-width: 100px; } }
</style>
