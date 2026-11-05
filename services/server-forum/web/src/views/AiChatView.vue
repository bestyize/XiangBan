<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { aiApi, type AiBalance, type AiTier } from '@/api/ai'
import { aiHistoryStorage, AI_HISTORY_LIMITS, readAiHistory, writeAiHistory, type AiConversation } from '@/lib/aiHistory'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
import { aiBucketLabel, formatAiCredits } from '@/lib/aiCredits'
import AiImageGenerator from '@/components/AiImageGenerator.vue'
import type { ImageCapability } from '@/lib/aiImages'

useHead(() => ({ title: '小伴 AI' }))
const session = useSession()
const route = useRoute()
const historyStorage = aiHistoryStorage()
const conversations = ref<AiConversation[]>([])
const selected = ref('')
const current = computed(() => conversations.value.find(c => c.id === selected.value))
const tiers = ref<AiTier[]>([])
const tier = ref('')
const balance = ref<AiBalance | null>(null)
const draft = ref('')
const code = ref('')
const error = ref('')
const accountError = ref('')
const notice = ref('')
const historyWarning = ref(historyStorage ? '' : '浏览器无法保存历史，离开页面后记录会丢失。')
const loading = ref(false)
const busy = ref(false)
const redeeming = ref(false)
const showHistory = ref(false)
const showCredits = ref(false)
const showImages = ref(false)
const imageCapability = ref<ImageCapability>()
const scrollArea = ref<HTMLElement | null>(null)
let controller: AbortController | null = null
let generation = 0
let accountSequence = 0
let accountController: AbortController | null = null
watch(() => route.query.panel, value => { if (value === 'account') showCredits.value = true }, { immediate:true })
const uid = computed(() => {
  try {
    const token = session.tokens?.accessToken
    if (!token || !session.user?.uid) return ''
    const payload = JSON.parse(atob(token.split('.')[1]!.replace(/-/g, '+').replace(/_/g, '/')))
    return payload.sub === session.user.uid ? session.user.uid : ''
  } catch { return '' }
})
function save() {
  if (uid.value && !writeAiHistory(historyStorage, uid.value, conversations.value)) historyWarning.value = '浏览器无法保存历史，离开页面后记录会丢失。'
}
function newChat() {
  if (busy.value) return
  const chat: AiConversation = { id: crypto.randomUUID(), title: '新对话', messages: [] }
  conversations.value = [chat, ...conversations.value].slice(0, AI_HISTORY_LIMITS.conversations); selected.value = chat.id; showHistory.value = false; error.value = ''
  save()
}
function removeChat(id: string) {
  if (busy.value) return
  conversations.value = conversations.value.filter(c => c.id !== id)
  if (selected.value === id) selected.value = conversations.value[0]?.id || ''
  if (!selected.value) newChat()
  save()
}
async function loadAccount(expected = generation) {
  const owner = uid.value
  if (!owner || expected !== generation) return
  const request = ++accountSequence
  accountController?.abort()
  const abort = new AbortController(); accountController = abort
  const valid = () => expected === generation && owner === uid.value && request === accountSequence
  loading.value = true; accountError.value = ''
  try {
    const token = await session.accessToken()
    if (!valid()) return
    if (!token) throw new Error('无法取得登录凭证，请重新登录。')
    const [capabilities, wallet] = await Promise.all([aiApi.capabilities(token, abort.signal), aiApi.balance(token, abort.signal)])
    if (!valid()) return
    const availableTiers = capabilities.tiers
    imageCapability.value = capabilities.media.image
    tiers.value = availableTiers; balance.value = wallet
    tier.value = availableTiers.find(t => t.tier === tier.value && t.available)?.tier || availableTiers.find(t => t.is_default && t.available)?.tier || availableTiers.find(t => t.available)?.tier || ''
    if (!tier.value) accountError.value = '当前没有可用模式，请刷新模式列表或检查点数。'
  } catch (e) {
    if (valid()) {
      abort.abort()
      tiers.value = []; tier.value = ''; balance.value = null; imageCapability.value = undefined
      accountError.value = e instanceof Error ? e.message : 'AI 服务暂不可用。'
    }
  } finally { if (valid()) { loading.value = false; accountController = null } }
}
watch(uid, value => {
  showImages.value = false; imageCapability.value = undefined
  generation++; accountSequence++; controller?.abort(); accountController?.abort(); controller = null; accountController = null; busy.value = false; loading.value = false; redeeming.value = false; showHistory.value = false
  tiers.value = []; tier.value = ''; balance.value = null; error.value = ''; accountError.value = ''; notice.value = ''; draft.value = ''; code.value = ''
  conversations.value = value ? readAiHistory(historyStorage, value) : []
  selected.value = conversations.value[0]?.id || ''
  if (value) { if (!selected.value) newChat(); void loadAccount() }
}, { immediate: true, flush:'sync' })
onBeforeUnmount(() => { generation++; accountSequence++; controller?.abort(); accountController?.abort() })
async function scrollDown() { await nextTick(); scrollArea.value?.scrollTo({ top: scrollArea.value.scrollHeight }) }
async function run(retry = false) {
  const mode = tiers.value.find(item => item.tier === tier.value && item.available)
  if (!uid.value || busy.value || !mode || !current.value) return
  const chat = current.value
  if (!retry && chat.messages.length >= AI_HISTORY_LIMITS.messages - 1) { error.value = '当前对话已达到历史长度上限，请新建对话。'; return }
  if (retry) {
    if (chat.messages.at(-1)?.role === 'assistant') chat.messages.pop()
    if (chat.messages.at(-1)?.role !== 'user') return
  } else {
    const text = draft.value.trim()
    if (!text) return
    if (text.length > 12_000) { error.value = '消息最多 12000 字符，请缩短后重试。'; return }
    chat.messages.push({ role: 'user', content: text }); draft.value = ''
    if (chat.title === '新对话') chat.title = text.slice(0, 24)
  }
  const input = chat.messages.map(m => ({ ...m }))
  chat.messages.push({ role: 'assistant', content: '' })
  const reply = chat.messages[chat.messages.length - 1]!
  save(); error.value = ''; busy.value = true
  const ownGeneration = generation, owner = uid.value
  const valid = () => ownGeneration === generation && uid.value === owner
  const abort = new AbortController(); controller = abort
  try {
    const token = await session.accessToken()
    if (!token) throw new Error('无法取得登录凭证，请重新登录。')
    if (!valid() || abort.signal.aborted) return
    await aiApi.chat(token, mode, input, abort.signal, text => {
      if (!valid() || abort.signal.aborted) return
      if (reply.content.length + text.length > AI_HISTORY_LIMITS.content) throw new Error('回复过长，已停止接收。可以新建对话继续。')
      reply.content += text; void scrollDown()
    })
    if (valid() && !reply.content.trim()) throw new Error('AI 没有返回文字，请重试。')
  } catch (e) {
    if (valid()) error.value = abort.signal.aborted ? '已停止生成。' : e instanceof Error ? e.message : '回复失败，请重试。'
  } finally {
    if (valid()) {
      if (!reply.content.trim() && chat.messages.at(-1) === reply) chat.messages.pop()
      busy.value = false; controller = null; save(); void loadAccount(ownGeneration)
    }
  }
}
function onEnter(event: KeyboardEvent) {
  if (event.isComposing) return
  event.preventDefault(); void run()
}
async function redeem() {
  if (!code.value.trim() || redeeming.value) return
  const expected = generation; redeeming.value = true; notice.value = ''
  try {
    const token = await session.accessToken()
    if (!token || expected !== generation) throw new Error('无法取得登录凭证，请重新登录。')
    const done = await aiApi.redeem(token, code.value.trim())
    if (expected !== generation) return
    code.value = ''; notice.value = `兑换成功，到账 ${formatAiCredits(done.quota, balance.value?.quota_per_unit)}`; await loadAccount(expected)
  } catch (e) { if (expected === generation) notice.value = e instanceof Error ? e.message : '兑换失败，请重试。' }
  finally { if (expected === generation) redeeming.value = false }
}
</script>

<template>
  <main class="ai-page">
    <header class="ai-header"><div><h1>小伴 AI</h1><p>聊想法，找灵感</p></div><div class="header-actions"><button v-if="uid" type="button" @click="showImages = !showImages" :aria-expanded="showImages">生图</button><button type="button" @click="showHistory = !showHistory" :aria-expanded="showHistory">历史</button><button type="button" @click="showCredits = !showCredits" :aria-expanded="showCredits">点数</button></div></header>
    <div v-if="!session.loggedIn" class="state"><h2>登录后开始对话</h2><p>和小伴聊聊你的想法。</p><button class="primary" @click="session.requireLogin()">登录</button></div>
    <div v-else-if="!uid" class="state"><p>正在读取账号信息…</p><button @click="session.loadSession()">重试</button></div>
    <template v-else>
      <AiImageGenerator v-show="showImages" :key="uid" :owner="uid" :capability="imageCapability" :get-token="() => session.accessToken()" @generated="loadAccount()" />
      <section v-if="showHistory" class="panel" aria-label="对话历史"><div class="panel-head"><strong>对话历史</strong><button @click="newChat" :disabled="busy">＋ 新对话</button></div><p class="hint">历史仅保存在当前浏览器；最多保留 30 个对话，每个 100 条消息；空间不足时只保留最近记录。</p><div v-for="chat in conversations" :key="chat.id" class="history-row"><button class="history-name" :class="{ active: chat.id === selected }" :disabled="busy" @click="selected = chat.id; showHistory = false; error = ''">{{ chat.title }}</button><button :disabled="busy" :aria-label="`删除${chat.title}`" @click="removeChat(chat.id)">删除</button></div></section>
      <section v-if="showCredits" class="panel" aria-label="小伴点">
        <div class="panel-head"><strong>小伴点</strong><button @click="loadAccount()" :disabled="loading">刷新</button></div>
        <template v-if="balance">
          <p class="credit-total">可用 {{ formatAiCredits(balance.total, balance.quota_per_unit) }}</p>
          <p class="hint">其中付费点数 {{ formatAiCredits(balance.paid, balance.quota_per_unit) }} · 手机、电脑与网页共用</p>
          <ul v-if="balance.buckets.length" class="credit-buckets" aria-label="点数明细">
            <li v-for="bucket in balance.buckets" :key="bucket.id">
              <span>{{ aiBucketLabel(bucket.kind) }}</span>
              <span>{{ formatAiCredits(bucket.remaining, balance.quota_per_unit) }} / {{ formatAiCredits(bucket.total, balance.quota_per_unit) }}</span>
              <span class="hint">{{ bucket.expires_at ? `${bucket.expires_at.slice(0, 10)} 到期` : '长期有效' }}</span>
            </li>
          </ul>
          <p v-if="balance.trial.available" class="hint">首次使用自动发放 {{ formatAiCredits(balance.trial.quota, balance.quota_per_unit) }} 试用点数，{{ balance.trial.days }} 天内有效。</p>
        </template>
        <p v-else class="hint">{{ loading ? '正在读取点数…' : '点数暂不可用' }}</p>
        <form class="redeem" @submit.prevent="redeem"><input v-model="code" aria-label="兑换码" placeholder="输入兑换码" autocomplete="off"><button class="primary" :disabled="redeeming || !code.trim()">兑换</button></form>
        <p v-if="notice" role="status" class="hint">{{ notice }}</p>
      </section>
      <div v-if="accountError" class="error" role="alert">{{ accountError }} <button @click="loadAccount()">重试</button></div>
      <div ref="scrollArea" class="messages" aria-live="polite">
        <div v-if="!current?.messages.length" class="welcome"><div class="spark">✦</div><h2>有什么想聊的？</h2><p>可以问问题、整理思路，或聊聊你的新点子。</p><RouterLink to="/create">想创作作品？前往创造 →</RouterLink></div>
        <article v-for="(message, index) in current?.messages" :key="index" class="message" :class="message.role"><span class="speaker">{{ message.role === 'user' ? '你' : '小伴' }}</span><div class="bubble">{{ message.content || (busy ? '正在思考…' : '') }}</div></article>
        <div v-if="error" class="error" role="alert">{{ error }} <button @click="run(true)" :disabled="busy || !tier">重试</button></div>
      </div>
      <form class="composer" @submit.prevent="run()"><textarea v-model="draft" placeholder="发消息给小伴…" aria-label="消息" rows="2" maxlength="12000" :disabled="busy" @keydown.enter.exact="onEnter"></textarea><div class="composer-bar"><label><span class="sr-only">对话模式</span><select v-model="tier" :disabled="busy || !tiers.length"><option v-if="!tiers.length" value="">{{ loading ? '加载模式…' : '暂无可用模式' }}</option><option v-for="item in tiers" :key="item.tier" :value="item.tier" :disabled="!item.available">{{ item.display_name }}{{ item.available ? '' : item.requires_paid ? '（需要付费点数）' : '（暂不可用）' }}</option></select></label><button v-if="busy" type="button" class="stop" @click="controller?.abort()">停止</button><button v-else class="send" :disabled="!draft.trim() || !tier" aria-label="发送消息">↑</button></div></form>
      <p v-if="historyWarning" class="footnote" role="status">{{ historyWarning }}</p><p class="footnote">对话仅在当前浏览器保留。小伴的回答可能有误，请核实重要信息。</p>
    </template>
  </main>
</template>

<style scoped>
.ai-page { max-width: 760px; margin: 0 auto; padding: 16px 16px 24px; min-height: calc(100dvh - var(--header-h) - var(--tabbar-h)); display: flex; flex-direction: column; }
.ai-header,.panel-head,.composer-bar,.header-actions,.history-row,.redeem { display:flex; align-items:center; justify-content:space-between; gap:8px; }
h1,h2,p { margin:0; } h1 { font-size:20px; } h2 { font-size:16px; } .ai-header p,.hint,.footnote { color:var(--ink-3); font-size:12px; }
button { border:0; background:var(--surface-2); border-radius:999px; min-height:44px; padding:0 16px; } button:disabled { opacity:.5; cursor:not-allowed; } .primary { color:var(--brand); background:color-mix(in srgb, var(--brand) 9%, var(--surface)); }
.state,.welcome { text-align:center; margin:auto; padding:48px 16px; color:var(--ink-2); } .state h2,.welcome h2 { color:var(--ink); } .state button { margin-top:16px; } .welcome p { margin:8px 0 16px; } .welcome a { color:var(--brand); } .spark { color:var(--brand); font-size:32px; }
.panel { background:var(--surface); border-radius:16px; padding:16px; margin-top:12px; } .panel p { margin:8px 0; } .history-row { border-top:1px solid var(--line); } .history-row button { background:transparent; } .history-name { flex:1; text-align:left; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; } .history-name.active { color:var(--brand); } .redeem input { min-width:0; flex:1; border:0; border-radius:8px; background:var(--surface-2); padding:12px; }
.credit-total { font-size:24px; font-weight:600; font-variant-numeric:tabular-nums; overflow-wrap:anywhere; }
.credit-buckets { list-style:none; padding:0; margin:12px 0; } .credit-buckets li { display:flex; flex-wrap:wrap; gap:4px 12px; padding:6px 0; font-size:13px; font-variant-numeric:tabular-nums; }
.messages { flex:1; min-height:260px; max-height:calc(100dvh - 310px); overflow-y:auto; display:flex; flex-direction:column; gap:16px; padding:24px 0; overscroll-behavior:contain; }
.message { max-width:90%; } .message.user { align-self:flex-end; } .speaker { display:block; color:var(--ink-3); font-size:12px; margin:0 4px 4px; } .user .speaker { text-align:right; } .bubble { white-space:pre-wrap; overflow-wrap:anywhere; background:var(--surface); border-radius:16px; padding:12px 16px; } .user .bubble { background:color-mix(in srgb, var(--brand) 9%, var(--surface)); }
.error { color:var(--danger); font-size:14px; padding:8px 0; } .error button { color:var(--brand); background:transparent; }
.composer { position:sticky; bottom:16px; z-index:1; background:var(--surface); border-radius:16px; padding:12px; box-shadow:0 2px 12px rgba(0,0,0,.06); } textarea { display:block; resize:vertical; width:100%; max-height:160px; border:0; outline:0; background:transparent; font-size:16px; } select { border:0; background:transparent; min-height:44px; max-width:200px; } .send { width:44px; padding:0; color:var(--brand-ink); background:var(--brand-fill); font-size:24px; } .send:disabled { background:var(--ink-3); } .footnote { text-align:center; padding-top:8px; } .sr-only { position:absolute; width:1px; height:1px; overflow:hidden; clip-path:inset(50%); }
@media (max-width:1023px) { .composer { bottom:var(--mobile-nav-offset, 84px); } .ai-page { padding-bottom:calc(12px + env(safe-area-inset-bottom)); } .messages { max-height:calc(100dvh - 300px - env(safe-area-inset-bottom)); } }
</style>
