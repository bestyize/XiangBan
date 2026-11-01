<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { discoverApi, type NewsComment } from '@/api/discover'
import { useSession } from '@/stores/session'
import PagerNav from '@/components/PagerNav.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import RequestState from './RequestState.vue'
import { useCommunityResource } from './resource'
import { displayDate } from './contract'
const props = defineProps<{ articleKey:string }>()
const session = useSession()
const page = ref(1), body = ref(''), reply = ref<NewsComment|null>(null)
const busy = ref(false), actionError = ref('')
let context = 0
const { data, loading, error, load } = useCommunityResource(() => discoverApi.comments(props.articleKey, page.value))
watch([() => props.articleKey, () => session.user?.uid, () => session.loggedIn], () => {
  ++context; page.value = 1; body.value = ''; reply.value = null; busy.value = false; actionError.value = ''; void load()
}, { immediate:true })
watch(page, load)
const valid = computed(() => Array.from(body.value.trim()).length >= 2 && Array.from(body.value.trim()).length <= 2000)
async function act(work:() => Promise<unknown>, done?:() => void) {
  if (busy.value || !session.requireLogin()) return
  const mine = context
  busy.value = true; actionError.value = ''
  try {
    await work()
    if (mine !== context) return
    done?.()
    if (page.value !== 1) page.value = 1
    else await load()
  } catch (e) { if (mine === context) actionError.value = e instanceof Error ? e.message : '操作失败，请重试' }
  finally { if (mine === context) busy.value = false }
}
function submit() {
  if (!valid.value) return
  const key = props.articleKey, text = body.value.trim(), parent = reply.value?.id
  void act(() => discoverApi.comment(key, text, parent), () => { body.value = ''; reply.value = null })
}
function toggleLike(item:NewsComment) {
  const key = props.articleKey
  void act(() => discoverApi.likeComment(key, item.id, !item.liked))
}
function remove(item:NewsComment) {
  if (!window.confirm('删除这条评论？')) return
  const key = props.articleKey
  void act(() => discoverApi.deleteComment(key, item.id))
}
function respond(item:NewsComment) { if (session.requireLogin()) reply.value = item }
</script>
<template><section class="comments"><h2>评论<span v-if="data"> · {{ data.total }}</span></h2><RequestState :loading="loading" :error="error" @retry="load" /><p v-if="actionError" class="action-error" role="alert">{{ actionError }}</p><template v-if="data"><article v-for="item in data.items" :key="item.id" class="comment"><header><RouterLink :to="`/u/${item.author.uid}`" class="avatar-link" :aria-label="`查看 ${item.author.nickname} 的主页`"><UserAvatar :name="item.author.nickname" :src="item.author.avatar_url" :size="32" /></RouterLink><div class="comment-author"><RouterLink :to="`/u/${item.author.uid}`">{{ item.author.nickname }}</RouterLink><time>{{ displayDate(item.created_at) }}</time></div></header><p v-if="item.reply_to" class="reply-to">回复评论 #{{ item.reply_to }}</p><p class="body">{{ item.body }}</p><div class="actions"><button :disabled="busy" :aria-pressed="item.liked" @click="toggleLike(item)">{{ item.liked ? '已赞' : '赞' }} {{ item.like_count }}</button><button :disabled="busy" @click="respond(item)">回复</button><button v-if="item.mine" :disabled="busy" @click="remove(item)">删除</button></div></article><p v-if="!data.items.length" class="empty">还没有评论，来聊聊你的看法。</p><PagerNav :page="data.page" :page-size="data.page_size" :total="data.total" @go="page = $event" /></template><form class="compose" @submit.prevent="submit"><template v-if="session.loggedIn"><p v-if="reply" class="reply-to">回复 {{ reply.author.nickname }} <button type="button" @click="reply = null">取消</button></p><label for="news-comment">写评论</label><textarea id="news-comment" v-model="body" :disabled="busy" rows="3" placeholder="分享你的看法，2–2000 字" maxlength="4000" /><div class="submit"><span>{{ Array.from(body.trim()).length }} / 2000</span><button type="submit" :disabled="!valid || busy">{{ busy ? '提交中…' : '发布评论' }}</button></div></template><button v-else type="button" class="login" @click="session.requireLogin()">登录后评论</button></form></section></template>
<style scoped>
.comments { max-width:760px; margin:12px auto 0; background:var(--surface); padding:22px; } h2 { font-size:16px; margin:0 0 16px; } h2 span, .empty { color:var(--ink-3); font-weight:400; } .comment { border-bottom:.5px solid var(--line); padding:16px 0; } header { display:flex; align-items:center; gap:10px; font-size:12px; color:var(--ink-3); } header a { color:var(--ink-2); } .comment-author { display:grid; gap:2px; min-width:0; } .comment-author a { font-size:14px; font-weight:600; overflow-wrap:anywhere; } .comment .body, .comment .reply-to, .comment .actions { margin-left:42px; } .body { font-size:14px; line-height:1.8; white-space:pre-wrap; overflow-wrap:anywhere; margin:8px 0; } .reply-to { color:var(--ink-3); font-size:12px; } .actions { display:flex; gap:16px; } button { border:0; color:var(--ink-3); min-height:44px; background:none; font-size:14px; padding:8px 4px; cursor:pointer; } button[aria-pressed=true] { color:var(--brand); } button:disabled { opacity:.5; cursor:default; } .compose { padding-top:16px; } label { display:block; font-size:14px; margin-bottom:8px; } textarea { width:100%; box-sizing:border-box; resize:vertical; background:var(--surface-2); border:0; border-radius:8px; padding:12px; color:var(--ink); font-size:16px; } .submit { display:flex; align-items:center; justify-content:space-between; gap:8px; } .submit span { font-size:12px; color:var(--ink-3); } .submit button, .login { background:color-mix(in srgb,var(--brand) 8%,var(--surface)); color:var(--brand); border-radius:24px; padding:8px 20px; margin-top:8px; } .action-error { color:var(--brand); font-size:14px; }
</style>
