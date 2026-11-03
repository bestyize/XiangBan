<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { TopicItem } from '@/api/types'
import { cardTitleLines, wallpaperRatio, type ArticleLayout } from '@/lib/cardPresentation'
import { timeAgo, compactCount, pinScopeLabel, readLevelBadge } from '@/lib/format'
import { api } from '@/api/forum'
import { useSession } from '@/stores/session'
import UserAvatar from '@/components/UserAvatar.vue'
import DiscoverGlyph from './DiscoverGlyph.vue'
import { displayDate, webLink } from './contract'
const props = withDefaults(defineProps<{ topic: TopicItem & { images?: string[] }; mode?: 'topic'|'media'|'open'|'help'; layout?: ArticleLayout; showCategory?: boolean; noteCover?: boolean }>(), { mode:'topic', layout:'reading', showCategory:false })
const emit = defineEmits<{selectTag:[tag:string]}>()
const session = useSession()
const liked = ref(false), likeCount = ref(0), liking = ref(false), likeError = ref('')
watch(() => props.topic, topic => { liked.value = !!topic.liked; likeCount.value = topic.like_count; likeError.value = '' }, {immediate:true})
async function toggleLike() {
  if (liking.value || !session.requireLogin()) return
  const id = props.topic.id
  liking.value = true; likeError.value = ''
  try {
    const result = await api.likeTopic(id,!liked.value)
    if (props.topic.id === id) { liked.value = result.liked; likeCount.value = result.like_count }
  } catch { if (props.topic.id === id) likeError.value = '点赞失败，请重试' }
  finally { liking.value = false }
}
const broken = ref<string[]>([])
const ratio=ref(16/9)
watch(() => props.topic.id, () => { broken.value = []; ratio.value=16/9 })
const image = computed(() => (props.topic.images || []).map(webLink).find((v): v is string => !!v && !broken.value.includes(v)))
const presentation = computed(() => props.topic.help || props.mode === 'help' ? 'reading' : props.layout)
const visual=computed(()=>presentation.value==='visual' || presentation.value==='masonry')
const coverStats=computed(()=>!!image.value && presentation.value!=='plain')
const showFooter=computed(()=>!coverStats.value || props.topic.like_count>0 || (!visual.value && props.topic.reply_count>0))
function loaded(event:Event) { const img=event.target as HTMLImageElement; ratio.value=wallpaperRatio(img.naturalWidth,img.naturalHeight,2/3) }
const status = computed(() => { const h=props.topic.help; if(!h) return ''; if(h.status==='solved') return '已采纳'; if(h.status==='closed') return '已关闭'; return h.status==='open' && new Date(h.expires_at).getTime()>Date.now() ? '进行中' : '已到期' })
</script>
<template>
  <article class="topic-card" :class="[mode, `layout-${presentation}`, {'has-cover':image && presentation !== 'plain', 'visual-card':visual}]" :style="{'--cover-ratio':ratio,'--title-lines':cardTitleLines(topic.category.slug)}">
    <span v-if="mode === 'help' && !topic.help" class="flag guide">站方指南</span>
    <div v-if="topic.help" class="help-line"><span class="bounty">{{ topic.help.bounty ? `悬赏 ${topic.help.bounty} 分` : '无悬赏' }}</span><span class="status" :class="topic.help.status">{{ status }}</span><span v-if="status === '进行中'" class="expires">截止 {{ displayDate(topic.help.expires_at) }}</span></div>
    <div v-if="noteCover && !image" class="note-title" :class="`tone-${topic.id % 3}`" aria-hidden="true"><small>{{ topic.category.name }}</small><strong>{{ topic.title || topic.excerpt || '社区分享' }}</strong></div>
    <div class="topic-main">
      <div class="topic-content">
        <h3><span v-if="topic.pinned" class="flag">{{ pinScopeLabel(topic.pin_scope) }}</span><span v-if="topic.featured" class="flag">精选</span><span v-if="topic.min_read_level > 0" class="flag">{{ readLevelBadge(topic.min_read_level) }}</span><RouterLink :to="`/t/${topic.id}`">{{ topic.title || topic.excerpt || '社区分享' }}</RouterLink></h3>
        <p v-if="topic.excerpt && (!visual || !image) && topic.excerpt !== topic.title" class="excerpt">{{ topic.excerpt }}</p>
        <div v-if="showCategory || mode === 'media' || mode === 'open'" class="secondary"><RouterLink v-if="showCategory" :to="`/c/${topic.category.slug}`" class="tag">{{ topic.category.name }}</RouterLink><span v-else class="tag">{{ topic.tags?.[0] || topic.category.name }}</span></div>
        <button v-if="noteCover && topic.tags?.[0]" type="button" class="topic-tag" @click="emit('selectTag', topic.tags[0])"># {{ topic.tags[0] }}</button>
      </div>
      <RouterLink v-if="image && presentation !== 'plain'" :to="`/t/${topic.id}`" class="cover" tabindex="-1" aria-hidden="true"><img :src="image" alt="" loading="lazy" decoding="async" @load="loaded" @error="broken.push(image!)"><span class="cover-stats"><span><DiscoverGlyph name="eye" :size="14" />{{ compactCount(topic.view_count) }}</span><span><DiscoverGlyph name="comment" :size="14" />{{ compactCount(topic.reply_count) }}</span></span></RouterLink>
    </div>
    <span v-if="coverStats" class="sr-only">{{ topic.view_count }} 次浏览，{{ topic.reply_count }} {{ topic.help ? '个回答' : '条回复' }}</span>
    <div class="meta" :class="{'last-line':!showFooter}"><RouterLink :to="`/u/${topic.author.uid}`" class="author"><UserAvatar :name="topic.author.nickname" :src="topic.author.avatar_url" :size="18" /><span>{{ topic.author.nickname }}</span></RouterLink><span v-if="topic.locked">已锁定</span><button v-if="noteCover" class="note-like" type="button" :aria-pressed="liked" :aria-label="liked ? '取消点赞' : '点赞'" :disabled="liking" @click="toggleLike">{{ liked ? '♥' : '♡' }} {{ likeCount }}</button><time v-else :datetime="topic.created_at">{{ timeAgo(topic.created_at) }}</time></div>
    <p v-if="likeError" class="note-like-error" role="alert">{{ likeError }}</p>
    <div v-if="showFooter && !noteCover" class="counts"><span v-if="!coverStats && topic.view_count">{{ compactCount(topic.view_count) }} 浏览</span><span v-if="topic.like_count">{{ compactCount(topic.like_count) }} 赞</span><RouterLink v-if="!coverStats" :to="`/t/${topic.id}`">{{ compactCount(topic.reply_count) }} {{ topic.help ? '回答' : '回复' }}</RouterLink><span v-if="topic.reply_count && !visual" class="last-reply">{{ timeAgo(topic.last_reply_at) }}回复</span></div>
  </article>
</template>
<style scoped>
.topic-card{position:relative;display:flex;flex-direction:column;padding:16px;background:var(--surface);border:1px solid var(--line);border-radius:12px;min-width:0;transition:border-color .15s}.topic-card:hover{border-color:var(--line-strong)}.topic-main{display:flex;align-items:flex-start;gap:16px;min-width:0}.topic-content{flex:1;min-width:0}h3{font-size:14px;line-height:1.6;font-weight:400;margin:0;overflow-wrap:anywhere;display:-webkit-box;-webkit-line-clamp:var(--title-lines,2);-webkit-box-orient:vertical;overflow:hidden}h3 a::after{content:'';position:absolute;inset:0}h3 a:hover{color:var(--brand)}.flag{display:inline-block;font-size:12px;color:var(--brand);margin-right:6px}.guide{margin-bottom:8px}.cover{width:112px;aspect-ratio:16/9;flex:none;overflow:hidden;border-radius:8px;background:var(--surface-2)}.cover img{width:100%;height:100%;object-fit:cover;display:block}.excerpt{font-size:13px;color:var(--ink-2);line-height:1.7;margin:7px 0 0;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere}.meta,.secondary,.help-line,.counts{display:flex;align-items:center;gap:8px;font-size:12px;color:var(--ink-3);min-width:0}.meta,.secondary,.counts{position:relative;z-index:1;pointer-events:none}.meta a,.secondary a,.counts a{pointer-events:auto}.meta{margin-top:12px;justify-content:space-between;flex-wrap:wrap}.author{display:flex;align-items:center;gap:6px;min-width:0;max-width:100%}.author>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.meta time{white-space:nowrap}.secondary{margin-top:8px}.tag,.status{background:var(--surface-2);padding:2px 7px;border-radius:5px}.help-line{margin-bottom:10px;flex-wrap:wrap}.bounty{color:var(--brand);font-weight:600}.status.open{color:var(--brand)}.status.solved{color:var(--ink-2)}.expires{margin-left:auto}.counts{margin-top:8px;flex-wrap:wrap;gap:4px 12px}.last-reply{margin-left:auto}
.visual-card{padding:0;overflow:hidden;height:100%}.visual-card .topic-main{flex-direction:column-reverse;gap:0}.visual-card .cover{width:100%;aspect-ratio:16/9;border-radius:0}.visual-card .topic-content{width:100%;box-sizing:border-box;padding:12px 14px 0}.visual-card h3{font-size:14px;line-height:1.6;font-weight:400;-webkit-line-clamp:var(--title-lines,2)}.visual-card .meta{margin-top:auto;padding:12px 14px 0;align-items:flex-start;gap:6px}.visual-card .meta time{font-size:12px}.visual-card .counts{padding:0 14px 14px;font-size:12px;gap:4px 8px}.visual-card .last-reply{display:none}.layout-plain .excerpt{color:var(--ink-3)}
@media(max-width:480px){.topic-card:not(.visual-card){padding:14px}.cover{width:96px}.topic-main{gap:12px}.visual-card .topic-content{padding:10px 10px 0}.visual-card h3{font-size:14px}.visual-card .meta{padding:10px 10px 0}.visual-card .counts{padding:0 10px 12px}.visual-card .meta time{width:100%}}
.layout-masonry{height:auto}.layout-masonry .cover{aspect-ratio:var(--cover-ratio,1.77778)}.layout-masonry h3{min-height:0;-webkit-line-clamp:var(--title-lines,2)}
.cover{position:relative}.cover-stats{position:absolute;inset:auto 0 0;display:flex;flex-wrap:wrap;align-items:center;gap:2px 8px;padding:12px 8px 5px;background:linear-gradient(transparent,rgba(0,0,0,.72));color:#fff;font-size:12px;font-weight:400;line-height:1.3;font-variant-numeric:tabular-nums;text-shadow:0 1px 2px #0008}.cover-stats>span{display:inline-flex;align-items:center;gap:4px;white-space:nowrap}.cover-stats svg{flex:none}.visual-card .cover-stats{padding:22px 10px 8px;gap:4px 12px}.visual-card .meta.last-line{padding-bottom:14px}.topic-card:not(.visual-card) .cover-stats{padding:10px 4px 4px;gap:2px 6px}.topic-card:not(.visual-card) .cover-stats svg{width:12px;height:12px}
.note-title{background:#f2e6d7;color:#594437;padding:12px;min-height:112px;display:grid;align-content:center;gap:6px}.note-title.tone-1{background:#f4e0df;color:#703d42}.note-title.tone-2{background:#e8ecdf;color:#45543e}.note-title strong{display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:3;overflow:hidden;overflow-wrap:anywhere}.topic-tag{position:relative;z-index:1;border:0;background:none;color:var(--brand);padding:5px 0;font-size:12px;overflow-wrap:anywhere}
.note-like{pointer-events:auto;border:0;background:none;font:inherit;color:var(--ink-3);padding:4px 0}.note-like[aria-pressed=true]{color:var(--brand)}.note-like-error{font-size:11px;color:var(--brand);padding:0 10px}.visual-card .meta:has(.note-like){padding-bottom:9px}
</style>
