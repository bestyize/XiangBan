<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { aiApi, aiBaseUrl, type AiBalance, type AiTier } from '@/api/ai'
import { aiHistoryStorage, readAiHistory } from '@/lib/aiHistory'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
import { aiBucketLabel, formatAiCredits } from '@/lib/aiCredits'
import AiImageGenerator from '@/components/AiImageGenerator.vue'
import type { ImageCapability } from '@/lib/aiImages'

const WebWorkbench = defineAsyncComponent(() => import('xiaoban-desktop/web-workbench'))
useHead(() => ({ title: '小伴 AI' }))
const session = useSession()
const route = useRoute()
const tiers = ref<AiTier[]>([])
const choices = computed(() => tiers.value.filter(t => t.available).map(t => ({id:t.tier, label:t.display_name, default:t.is_default})))
const balance = ref<AiBalance | null>(null)
const code = ref(''), notice = ref(''), accountError = ref('')
const loading = ref(false), redeeming = ref(false), showCredits = ref(false), showImages = ref(false)
const imageCapability = ref<ImageCapability>()
let generation = 0, accountController: AbortController | undefined
watch(() => route.query.panel, value => { if (value === 'account') showCredits.value = true }, { immediate:true })
const uid = computed(() => {
  try {
    const token = session.tokens?.accessToken
    if (!token || !session.user?.uid) return ''
    const payload = JSON.parse(atob(token.split('.')[1]!.replace(/-/g, '+').replace(/_/g, '/')))
    return payload.sub === session.user.uid ? session.user.uid : ''
  } catch { return '' }
})

const legacy = computed(() => uid.value ? readAiHistory(aiHistoryStorage(), uid.value).filter(c => c.messages.length) : [])
const request = computed(() => {
  const owner = uid.value
  return async (path: string, init: RequestInit) => {
    const token = await session.accessToken()
    if (!owner || owner !== uid.value || !token || init.signal?.aborted) throw Error('请重新登录小伴')
    const base = aiBaseUrl(import.meta.env.VITE_AI_BASE_URL || '')
    return fetch(base + '/api/workbench' + path, { ...init, headers: { Authorization: 'Bearer ' + token, ...(init.body ? {'Content-Type':'application/json'} : {}) }, redirect:'error', cache:'no-store' })
  }
})
async function loadAccount(expected = generation) {
  const owner = uid.value
  if (!owner || expected !== generation) return
  accountController?.abort()
  const abort = new AbortController()
  accountController = abort
  const valid = () => expected === generation && owner === uid.value && !abort.signal.aborted
  loading.value = true
  accountError.value = ''
  try {
    const token = await session.accessToken()
    if (!token) throw Error('请重新登录小伴')
    if (!valid()) return
    const [capabilities, wallet] = await Promise.allSettled([aiApi.capabilities(token, abort.signal), aiApi.balance(token, abort.signal)])
    if (!valid()) return
    if (capabilities.status === 'fulfilled') {
      tiers.value = capabilities.value.tiers
      imageCapability.value = capabilities.value.media.image
    }
    if (wallet.status === 'fulfilled') balance.value = wallet.value
    if (capabilities.status === 'rejected') throw capabilities.reason
    if (wallet.status === 'rejected') throw wallet.reason
  } catch (e) { if (valid()) accountError.value = e instanceof Error ? e.message : 'AI 服务暂不可用。' }
  finally { if (valid()) loading.value = false }
}
watch(uid, value => {
  generation++
  accountController?.abort()
  showImages.value = false
  imageCapability.value = undefined
  tiers.value = []; balance.value = null; notice.value = ''; code.value = ''; accountError.value = ''
  loading.value = false; redeeming.value = false
  if (value) void loadAccount()
}, {immediate:true, flush:'sync'})
onBeforeUnmount(() => { generation++; accountController?.abort() })
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
  <section class="ai-workspace">
    <header class="ai-header">
      <h1>小伴 AI</h1>
      <nav aria-label="工作台功能">
        <button v-if="uid && imageCapability" @click="showImages = !showImages" :aria-expanded="showImages">生图</button>
        <button v-if="uid" @click="showCredits = !showCredits" :aria-expanded="showCredits">点数</button>
        <RouterLink to="/ai/download">下载客户端</RouterLink>
      </nav>
    </header>
    <div v-if="!session.loggedIn" class="state"><h2>和小伴聊聊</h2><p>登录后可以在网页上开始对话，下次回来接着聊。</p><button class="btn btn-primary" @click="session.requireLogin()">登录小伴</button></div>
    <div v-else-if="!uid" class="state"><p>正在读取账号信息…</p><button @click="session.loadSession()">重试</button></div>
    <template v-else>
      <AiImageGenerator v-if="showImages" :key="uid" :owner="uid" :capability="imageCapability" :get-token="() => session.accessToken()" @generated="loadAccount()" />
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
      <WebWorkbench :key="uid" :owner="uid" :request="request" :tiers="choices" />
      <details v-if="legacy.length" class="legacy"><summary>此浏览器的旧对话</summary>
        <details v-for="conversation in legacy" :key="conversation.id"><summary>{{ conversation.title }}</summary>
          <p v-for="(message,i) in conversation.messages" :key="i"><b>{{ message.role === 'user' ? '你' : '小伴' }}</b> {{ message.content }}</p>
        </details>
      </details>
    </template>
  </section>
</template>
<style scoped>
.ai-workspace{max-width:1120px;margin:0 auto;color:var(--ink)}
.ai-header,.ai-header nav,.panel-head,.redeem{display:flex;align-items:center;gap:12px}
.ai-header,.panel-head{justify-content:space-between}.ai-header{padding:9px 2px 15px}.ai-header h1{font-size:21px;margin:0}
.ai-header button,.ai-header a,.panel button,.state button,.error button{font:inherit;cursor:pointer}
.ai-header nav>*{border:0;background:transparent;color:var(--ink-2);font-size:14px;text-decoration:none;white-space:nowrap}
.state,.panel,.legacy{padding:18px;border-radius:16px;background:var(--surface);margin-bottom:12px}
.state{text-align:center;padding:42px 18px}.hint{color:var(--ink-2);font-size:13px}.credit-total{font-size:20px;font-weight:650}
.credit-buckets{list-style:none;padding:0}.credit-buckets li{display:flex;gap:12px;flex-wrap:wrap}.error{padding:10px}.redeem input{min-width:0}.legacy p{white-space:pre-wrap;overflow-wrap:anywhere}
</style>
