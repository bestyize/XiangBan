<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { discoverApi, type NewsComment } from '@/api/discover'
import { useSession } from '@/stores/session'
import PagerNav from '@/components/PagerNav.vue'
import UserAvatar from '@/components/UserAvatar.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import RequestState from './RequestState.vue'
import { useCommunityResource } from './resource'
import { displayDate } from './contract'
const props = defineProps<{ owner:string; repo:string; editorial?:boolean }>()
const session = useSession()
const page = ref(1), body = ref(''), reply = ref<NewsComment|null>(null)
const busy = ref(false), actionError = ref('')
const expanded = ref(false), editor = ref<HTMLTextAreaElement | null>(null)
async function focusComposer() {
  if (!session.requireLogin()) return
  expanded.value = true
  await nextTick()
  editor.value?.focus()
}
function closeComposer() { expanded.value = false; reply.value = null }
function replyName(id:number) { return data.value?.items.find(item => item.id === id)?.author.nickname ?? `评论 #${id}` }
let context = 0
const { data, loading, error, load } = useCommunityResource(() => discoverApi.repoComments(props.owner, props.repo, page.value))
watch([() => props.owner, () => props.repo, () => session.user?.uid, () => session.loggedIn], () => {
  ++context; page.value = 1; body.value = ''; reply.value = null; busy.value = false; actionError.value = ''; expanded.value = false; void load()
}, { immediate:true })
watch(page, load)
const length = computed(() => Array.from(body.value.trim()).length)
const valid = computed(() => length.value >= 2 && length.value <= 2000)
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
  const { owner, repo } = props, text = body.value.trim(), parent = reply.value?.id
  void act(() => discoverApi.repoComment(owner, repo, text, parent), () => { body.value = ''; reply.value = null; expanded.value = false })
}
function toggleLike(item:NewsComment) {
  const { owner, repo } = props
  void act(() => discoverApi.likeRepoComment(owner, repo, item.id, !item.liked))
}
function remove(item:NewsComment) {
  if (!window.confirm('删除这条评论？')) return
  const { owner, repo } = props
  void act(() => discoverApi.deleteRepoComment(owner, repo, item.id))
}
function respond(item:NewsComment) { if (session.requireLogin()) { reply.value = item; void focusComposer() } }
defineExpose({ total: computed(() => data.value?.total ?? null), focusComposer })
</script>
<template>
  <section id="comments" class="comments" :class="{ editorial }">
    <h2 v-if="editorial && data">共 {{ data.total }} 条评论</h2>
    <h2 v-else>评论<span v-if="data"> · {{ data.total }}</span></h2>
    <RequestState :loading="loading" :error="error" @retry="load" />
    <template v-if="data">
      <article v-for="item in data.items" :key="item.id" class="comment" :class="{ 'comment-reply': editorial && item.reply_to }">
        <template v-if="editorial">
          <RouterLink :to="`/u/${item.author.uid}`" class="avatar-link" :aria-label="`查看 ${item.author.nickname} 的主页`">
            <UserAvatar :name="item.author.nickname" :src="item.author.avatar_url" :size="item.reply_to ? 26 : 34" />
          </RouterLink>
          <div class="comment-content">
            <RouterLink class="comment-nickname" :to="`/u/${item.author.uid}`">{{ item.author.nickname }}</RouterLink>
            <p class="body"><span v-if="item.reply_to" class="reply-to">回复 {{ replyName(item.reply_to) }}： </span>{{ item.body }}</p>
            <div class="comment-meta">
              <time :datetime="item.created_at">{{ displayDate(item.created_at) }}</time>
              <button type="button" :disabled="busy" @click="respond(item)">回复</button>
              <button v-if="item.mine" type="button" :disabled="busy" @click="remove(item)">删除</button>
            </div>
          </div>
          <button type="button" class="comment-like" :disabled="busy" :aria-pressed="item.liked"
            :aria-label="`${item.liked ? '取消赞' : '赞'}这条评论，${item.like_count} 个赞`" @click="toggleLike(item)">
            <svg viewBox="0 0 24 24" width="19" height="19" :fill="item.liked ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="1.6" aria-hidden="true">
              <path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z" />
            </svg>
            <span v-if="item.like_count">{{ item.like_count }}</span>
          </button>
        </template>
        <template v-else>
          <header><RouterLink :to="`/u/${item.author.uid}`" class="avatar-link" :aria-label="`查看 ${item.author.nickname} 的主页`"><UserAvatar :name="item.author.nickname" :src="item.author.avatar_url" :size="32" /></RouterLink><div class="comment-author"><RouterLink :to="`/u/${item.author.uid}`">{{ item.author.nickname }}</RouterLink><time>{{ displayDate(item.created_at) }}</time></div></header>
          <p v-if="item.reply_to" class="reply-to">回复 {{ replyName(item.reply_to) }}</p>
          <p class="body">{{ item.body }}</p>
          <div class="actions"><button class="comment-like" :disabled="busy" :aria-pressed="item.liked" :aria-label="`${item.liked ? '取消赞' : '赞'}这条评论，${item.like_count} 个赞`" @click="toggleLike(item)">{{ item.liked ? '已赞' : '赞' }}{{ item.like_count }}</button><button :disabled="busy" @click="respond(item)">回复</button><button v-if="item.mine" :disabled="busy" @click="remove(item)">删除</button></div>
        </template>
      </article>
      <p v-if="!data.items.length" class="empty">还没有评论，来说说你怎么用它的。</p>
      <PagerNav :page="data.page" :page-size="data.page_size" :total="data.total" @go="page = $event" />
    </template>
    <div :class="editorial ? 'readme-bottom-bar' : ''">
      <div v-if="editorial && !expanded" class="readme-bottom-inner">
        <button type="button" class="comment-capsule" @click="focusComposer">说点什么…</button>
        <slot name="actions" />
        <a class="comment-count" href="#comments" :aria-label="`查看评论${data ? '，共 ' + data.total + ' 条' : ''}`"><DiscoverGlyph name="comment" :size="25"/><span>{{ data?.total ?? '评论' }}</span></a>
      </div>
      <form v-if="!editorial || expanded" class="compose" @submit.prevent="submit">
        <template v-if="session.loggedIn">
          <div class="compose-title"><label for="repo-comment">{{ reply ? `回复 ${reply.author.nickname}` : '写评论' }}</label><button v-if="editorial" type="button" @click="closeComposer">收起</button><button v-else-if="reply" type="button" @click="reply = null">取消回复</button></div>
          <textarea id="repo-comment" ref="editor" v-model="body" :disabled="busy" rows="3" placeholder="分享你的看法，2–2000 字" maxlength="4000" />
          <div class="submit"><span>{{ length }} / 2000</span><button type="submit" :disabled="!valid || busy">{{ busy ? '提交中…' : editorial ? '发送' : '发布评论' }}</button></div>
        </template><button v-else type="button" class="login" @click="session.requireLogin()">登录后评论</button>
      </form>
      <p v-if="actionError" class="action-error" role="alert">{{ actionError }}</p>
    </div>
  </section>
</template>
<style scoped>
.comments { padding:20px 0 8px; } h2 { font-size:17px; font-weight:700; margin:0 0 16px; } h2 span, .empty { color:var(--ink-3); font-weight:400; } .comment { border-bottom:.5px solid var(--line); padding:16px 0; } header { display:flex; align-items:center; gap:10px; font-size:12px; color:var(--ink-3); } header a { color:var(--ink-2); } .comment-author { display:grid; gap:2px; min-width:0; } .comment-author a { font-size:14px; font-weight:600; overflow-wrap:anywhere; } .comment .body, .comment .reply-to, .comment .actions { margin-left:42px; } .body { font-size:14px; line-height:1.8; white-space:pre-wrap; overflow-wrap:anywhere; margin:8px 0; } .reply-to { color:var(--ink-3); font-size:12px; } .actions { display:flex; gap:16px; } button { border:0; color:var(--ink-3); min-height:44px; background:none; font-size:14px; padding:8px 4px; cursor:pointer; } button[aria-pressed=true] { color:var(--brand); } button:disabled { opacity:.5; cursor:default; } .compose { padding-top:16px; } label { display:block; font-size:14px; margin-bottom:8px; } textarea { width:100%; box-sizing:border-box; resize:vertical; background:var(--surface-2); border:0; border-radius:8px; padding:12px; color:var(--ink); font-size:16px; } .submit { display:flex; align-items:center; justify-content:space-between; gap:8px; } .submit span { font-size:12px; color:var(--ink-3); } .submit button, .login { background:color-mix(in srgb,var(--brand) 8%,var(--surface)); color:var(--brand); border-radius:24px; padding:8px 20px; margin-top:8px; } .action-error { color:var(--brand); font-size:14px; }

.compose-title{display:flex;justify-content:space-between;align-items:center}.compose-title label{margin:0}.compose-title button{font-size:13px}
.editorial {
  margin-top: 24px;
  padding: 16px clamp(12px, 3vw, 24px) 12px;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: var(--surface);
  scroll-margin-top: 80px;
}
.editorial h2 { font-size: 14px; margin-bottom: 8px; }
.editorial .comment {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) 36px;
  align-items: start;
  gap: 10px;
  padding: 14px 0 9px;
  border-bottom: 0;
}
.editorial .comment-reply { margin-inline-start: 44px; padding-top: 6px; }
.comment-content { min-width: 0; }
.comment-nickname { color: var(--ink-3); font-size: 13px; overflow-wrap: anywhere; }
.editorial .comment .body { margin: 4px 0 0; line-height: 1.6; }
.editorial .comment .reply-to { margin: 0; font-size: inherit; }
.comment-meta { display: flex; flex-wrap: wrap; align-items: center; gap: 0 12px; color: var(--ink-3); font-size: 12px; }
.comment-meta button { min-height: 30px; padding: 2px 0; color: var(--ink-2); font-size: inherit; }
.editorial .comment-like { display: grid; justify-items: center; align-content: start; gap: 2px; min-height: 36px; padding: 3px 0; font-size: 11px; }
.editorial .empty { margin: 0; padding: 20px 0; font-size: 13px; }
.editorial .readme-bottom-bar{position:fixed;z-index:38;bottom:0;left:0;right:0;background:var(--surface);border-top:1px solid var(--line);padding:8px 20px calc(8px + env(safe-area-inset-bottom))}.readme-bottom-inner{display:flex;align-items:center;gap:24px;max-width:760px;margin:auto;min-height:48px}.readme-bottom-inner .comment-capsule{width:36%;flex:0 1 36%;margin-right:auto;text-align:left;background:var(--surface-2);border-radius:24px;padding:10px 16px;font-size:14px;min-height:40px;color:var(--ink-3)}.comment-count{display:flex;align-items:center;gap:6px;font-size:13px;min-height:44px;color:var(--ink)}.editorial .compose{max-width:760px;margin:auto;padding:0}.editorial .submit button{background:var(--brand-fill,var(--brand));color:white;margin-top:0;min-height:36px}.editorial .action-error{max-width:760px;margin:6px auto}.editorial .readme-bottom-bar textarea{max-height:30vh}
@media(min-width:1100px){.editorial .readme-bottom-bar{left:200px}}
@media(max-width:400px){.readme-bottom-inner{gap:16px}.editorial .readme-bottom-bar{padding-inline:16px}}
</style>
