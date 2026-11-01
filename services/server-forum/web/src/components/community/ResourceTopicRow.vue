<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { TopicItem } from '@/api/types'
import { timeAgo, compactCount, pinScopeLabel, readLevelBadge } from '@/lib/format'
import UserAvatar from '@/components/UserAvatar.vue'
import { displayDate, webLink } from './contract'
const props = withDefaults(defineProps<{ topic: TopicItem & { images?: string[] }; mode?: 'topic'|'media'|'open'|'help' }>(), { mode:'topic' })
const broken = ref<string[]>([])
watch(() => props.topic.id, () => { broken.value = [] })
const images = computed(() => (props.topic.images || []).map(webLink).filter((v): v is string => !!v && !broken.value.includes(v)).slice(0, 3))
const status = computed(() => { const h=props.topic.help; if(!h) return ''; if(h.status==='solved') return '已采纳'; if(h.status==='closed') return '已关闭'; return h.status==='open' && new Date(h.expires_at).getTime()>Date.now() ? '进行中' : '已到期' })
</script>
<template>
  <article class="topic-card" :class="mode">
    <div v-if="topic.help && mode === 'help'" class="help-line"><span class="bounty">{{ topic.help.bounty ? `悬赏 ${topic.help.bounty} 分` : '无悬赏' }}</span><span class="status" :class="topic.help.status">{{ status }}</span><span v-if="status === '进行中'" class="expires">截止 {{ displayDate(topic.help.expires_at) }}</span></div>
    <div class="topic-main">
      <RouterLink v-if="images[0] && ['media','open','help'].includes(mode)" :to="`/t/${topic.id}`" class="cover" :class="mode" tabindex="-1" aria-hidden="true"><img :src="images[0]" alt="" loading="lazy" @error="broken.push(images[0]!)"></RouterLink>
      <div class="topic-content"><h3><span v-if="topic.pinned" class="flag">{{ pinScopeLabel(topic.pin_scope) }}</span><span v-if="topic.featured" class="flag">精选</span><span v-if="topic.min_read_level > 0" class="flag">{{ readLevelBadge(topic.min_read_level) }}</span><RouterLink :to="`/t/${topic.id}`">{{ topic.title || topic.excerpt || '社区分享' }}</RouterLink></h3>
        <div v-if="mode === 'media' || mode === 'open'" class="secondary"><span class="tag">{{ topic.tags?.[0] || topic.category.name }}</span><span v-if="mode === 'open'">{{ compactCount(topic.view_count) }} 浏览</span></div>
      </div>
      <RouterLink v-if="mode === 'topic' && images.length === 1" :to="`/t/${topic.id}`" class="cover small" tabindex="-1" aria-hidden="true"><img :src="images[0]" alt="" loading="lazy" @error="broken.push(images[0]!)"></RouterLink>
    </div>
    <div v-if="mode === 'topic' && images.length > 1" class="gallery"><RouterLink v-for="img in images" :key="img" :to="`/t/${topic.id}`" tabindex="-1" aria-hidden="true"><img :src="img" alt="" loading="lazy" @error="broken.push(img)"></RouterLink></div>
    <div class="meta"><RouterLink :to="`/u/${topic.author.uid}`" class="author"><UserAvatar :name="topic.author.nickname" :src="topic.author.avatar_url" :size="20" />{{ topic.author.nickname }}</RouterLink><RouterLink v-if="mode === 'topic'" :to="`/c/${topic.category.slug}`" class="category">{{ topic.category.name }}</RouterLink><span v-if="topic.locked">已锁定</span><span v-if="topic.like_count">{{ compactCount(topic.like_count) }} 赞</span><span v-if="mode === 'topic' && topic.view_count">{{ compactCount(topic.view_count) }} 浏览</span><span v-if="topic.reply_count || mode === 'help'">{{ compactCount(topic.reply_count) }} {{ mode === 'help' ? '回答' : '回复' }}</span><time :datetime="topic.created_at">{{ timeAgo(topic.created_at) }}</time></div>
  </article>
</template>
<style scoped>
.topic-card{position:relative;padding:14px;background:var(--surface);border-radius:12px;min-width:0}.topic-main{display:flex;align-items:center;gap:12px;min-width:0}.topic-content{flex:1;min-width:0}h3{font-size:16px;line-height:1.5;font-weight:600;margin:0;overflow-wrap:anywhere;display:-webkit-box;-webkit-line-clamp:3;-webkit-box-orient:vertical;overflow:hidden}h3 a::after{content:'';position:absolute;inset:0}h3 a:hover{text-decoration:underline;text-underline-offset:3px}.flag{display:inline-block;font-size:12px;color:var(--brand);margin-right:6px}.cover{width:88px;height:88px;flex:none;overflow:hidden;border-radius:8px;background:var(--surface-2)}.cover.media{width:72px;height:96px}.cover.help,.cover.small{width:72px;height:72px}.cover img,.gallery img{width:100%;height:100%;object-fit:cover;display:block}.gallery{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:6px;margin-top:10px}.gallery a{height:100px;overflow:hidden;border-radius:8px}.meta,.secondary,.help-line{display:flex;align-items:center;gap:8px;font-size:12px;color:var(--ink-3);min-width:0}.meta{position:relative;z-index:1;margin-top:10px;flex-wrap:wrap;pointer-events:none}.meta a{pointer-events:auto}.author{display:flex;align-items:center;gap:6px;min-width:0;flex:1;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.category,.meta time{white-space:nowrap}.secondary{margin-top:8px}.tag,.status{background:var(--surface-2);padding:2px 7px;border-radius:5px}.help-line{margin-bottom:8px}.bounty{color:var(--brand);font-weight:600}.status.open{color:#3973a7}.status.solved{color:#27885b}.expires{margin-left:auto}.media .topic-main,.open .topic-main,.help .topic-main{align-items:flex-start}@media(max-width:360px){.category{display:none}.meta{gap:6px}}
</style>
