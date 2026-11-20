<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { PackItem } from '@/api/content'
import { kindLabel } from '@/lib/content'
import { packLayout, wallpaperRatio, type PackLayout } from '@/lib/cardPresentation'
import UserAvatar from '@/components/UserAvatar.vue'
import { webLink } from './contract'
const props = defineProps<{ pack:PackItem; layout?:PackLayout; discussion?:boolean; plaza?:boolean }>()
const imageFailed=ref(false), ratio=ref(16/9)
const presentation=computed(()=>props.layout || packLayout(props.pack.kind))
const cover=computed(()=>!imageFailed.value ? webLink(props.pack.cover_url) : undefined)
watch(() => props.pack.cover_url, () => { imageFailed.value=false; ratio.value=16/9 })
function loaded(event:Event) {
  if (presentation.value !== 'masonry') return
  const image=event.target as HTMLImageElement
  ratio.value=wallpaperRatio(image.naturalWidth,image.naturalHeight)
}
</script>
<template>
  <RouterLink :to="`/packs/${pack.id}`" class="pack-card" :class="[`layout-${presentation}`, { visual: !!cover, 'plaza-card': plaza }]" :style="{'--cover-ratio':ratio}">
    <div class="cover" :class="{ 'no-cover': !cover }">
      <img v-if="cover" :src="cover" alt="" loading="lazy" decoding="async"
        :class="{ contain: !plaza && (pack.kind === 'game' || pack.kind === 'widget' || presentation === 'directory') }"
        @load="loaded" @error="imageFailed = true">
      <span v-else class="placeholder" aria-hidden="true">{{ kindLabel(pack.kind).slice(0, 2) }}</span>
      <div v-if="plaza" class="plaza-shot-foot">
        <span class="plaza-shot-stats">
          <span v-if="pack.view_count != null" :aria-label="`${pack.view_count} 次浏览`">👁 {{ pack.view_count }}</span>
          <span v-if="pack.reply_count != null" :aria-label="`${pack.reply_count} 条评论`">💬 {{ pack.reply_count }}</span>
        </span>
        <span class="plaza-shot-badge">{{ pack.featured ? '精选' : kindLabel(pack.kind) }}</span>
      </div>
      <span v-else-if="presentation !== 'directory'" class="cover-kind">{{ kindLabel(pack.kind) }}</span>
    </div>
    <div class="copy">
      <span v-if="presentation === 'directory'" class="kind">{{ kindLabel(pack.kind) }}</span>
      <span v-if="discussion" class="kind">创作 · {{ kindLabel(pack.kind) }}</span>
      <h3>{{ pack.title }}</h3>
      <span v-if="discussion" class="work-action">{{ pack.kind === 'game' ? '查看试玩' : '查看导入' }}</span>
      <p v-if="!plaza && pack.summary && (presentation === 'directory' || !cover)" class="summary">{{ pack.summary }}</p>
      <div v-if="plaza && pack.author?.nickname" class="plaza-author">
        <span class="plaza-avatar" :class="{ 'plaza-avatar--wancisou': pack.slug === 'wancisou' }" aria-hidden="true">{{ Array.from(pack.author.nickname.trim())[0] ?? '?' }}</span>
        {{ pack.author.nickname }}
      </div>
      <div v-else-if="!plaza && (pack.author?.nickname || pack.apply_count > 0)" class="author">
        <UserAvatar v-if="pack.author?.nickname" :name="pack.author.nickname" :size="18" />
        <span class="author-name">{{ pack.author?.nickname || '社区作品' }}</span>
        <small v-if="pack.apply_count > 0">{{ pack.apply_count }} 次使用</small>
      </div>
    </div>
  </RouterLink>
</template>
<style scoped>
.pack-card{display:flex;flex-direction:column;min-width:0;background:var(--surface);border:1px solid var(--line);border-radius:12px;overflow:hidden;color:var(--ink);transition:border-color .15s}.pack-card:hover{border-color:var(--line-strong)}.cover{position:relative;background:var(--surface-2);aspect-ratio:16/9;overflow:hidden;flex:none}.cover img{display:block;width:100%;height:100%;object-fit:cover}.cover img.contain{object-fit:contain}.cover-kind{position:absolute;left:8px;top:8px;background:#0009;color:#fff;border-radius:4px;font-size:12px;padding:2px 6px}.placeholder{display:grid;place-items:center;height:100%;font-size:24px;color:var(--ink-3)}.copy{padding:12px 14px;min-width:0;flex:1}.kind{font-size:12px;color:var(--ink-3)}h3{font-size:14px;font-weight:400;line-height:1.55;margin:0;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden;overflow-wrap:anywhere}.summary{font-size:13px;line-height:1.6;color:var(--ink-2);margin:6px 0 0;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}.author{display:flex;align-items:center;gap:6px;margin-top:12px;font-size:12px;color:var(--ink-3);min-width:0;flex-wrap:wrap}.author-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;flex:1;min-width:40px}.author small{white-space:nowrap;font-size:12px}.layout-visual h3{min-height:3.1em}.layout-directory{flex-direction:row;align-items:flex-start;gap:12px;padding:16px}.layout-directory .cover{width:48px;aspect-ratio:1;border-radius:10px}.layout-directory .placeholder{font-size:16px;color:var(--brand)}.layout-directory .copy{padding:0}.layout-directory h3{margin-top:4px}.layout-masonry .cover{aspect-ratio:var(--cover-ratio,1.77778)}
@media(max-width:480px){.copy{padding:10px}h3{font-size:14px}.layout-directory{padding:14px}.author{gap:5px}}
.work-action{display:inline-block;background:var(--brand-soft);color:var(--brand);border-radius:6px;padding:3px 6px;margin-top:5px;font-size:11px}
.plaza-card {
  border-color: var(--community-divider);
  border-radius: 18px;
  box-shadow: var(--shadow-card);
  transition: transform 0.18s, box-shadow 0.18s;
}
.plaza-card:hover { transform: translateY(-2px); box-shadow: 0 16px 34px -10px rgba(15, 23, 42, 0.14); }
.plaza-card:hover h3 { color: var(--brand); }
.plaza-card .cover { aspect-ratio: 3 / 2; }
.plaza-shot-foot {
  position: absolute;
  inset: auto 0 0;
  padding: 16px 8px 6px;
  background: linear-gradient(to top, rgba(12, 16, 24, 0.72), rgba(12, 16, 24, 0));
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: #fff;
  font-size: 10.5px;
}
.plaza-shot-stats { display: flex; gap: 6px; }
.plaza-shot-badge {
  padding: 2px 7px;
  border-radius: 999px;
  background: rgba(17, 24, 39, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.18);
  font-size: 10px;
  font-weight: 600;
}
.plaza-card .copy { padding: 9px 10px 10px; }
.plaza-card h3 {
  display: block;
  min-height: 0;
  margin-top: 1px;
  font-size: 13.5px;
  font-weight: 600;
  line-height: inherit;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.plaza-author { margin-top: 8px; font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.plaza-avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: inline-grid;
  place-items: center;
  font-size: 10.5px;
  font-weight: 700;
  color: var(--brand-ink);
  background: var(--brand-fill);
}
.plaza-avatar--wancisou { color: #fff; background: #d9a726; }
@media (max-width: 1023px) { .plaza-card { border-radius: 12px; } }
</style>
