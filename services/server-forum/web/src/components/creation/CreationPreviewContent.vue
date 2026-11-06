<script setup lang="ts">
import BuiltinEffectPreview from './BuiltinEffectPreview.vue'
import BuiltinWidgetPreview from './BuiltinWidgetPreview.vue'
import BuiltinPetPreview from './BuiltinPetPreview.vue'
defineProps<{ item:{ id:string; name:string; cover?:string; description:string; kind:string }; wide?:boolean; active?:boolean }>()
const labels:Record<string,string>={game:'游戏',wallpaper:'壁纸',widget:'小组件',effect:'特效',pet:'伙伴'}
</script>
<template>
  <div class="preview-content" :class="{wide}">
    <div class="preview-media" :class="item.kind">
      <BuiltinEffectPreview v-if="item.kind === 'effect'" :id="item.id" :playing="active !== false" />
      <BuiltinWidgetPreview v-else-if="item.kind === 'widget'" :id="item.id" />
      <BuiltinPetPreview v-else-if="item.kind === 'pet'" :id="item.id" :playing="active !== false" />
      <img v-else :src="item.cover" :alt="item.name" />
    </div>
    <div class="preview-copy">
      <p class="detail-kind">{{ labels[item.kind] }}<span>·</span>{{ item.kind === 'wallpaper' ? '创作灵感' : '小伴官方' }}</p>
      <h2 v-if="wide">{{ item.name }}</h2>
      <p class="description">{{ item.description }}</p>
      <p class="detail-note">{{ item.kind === 'wallpaper' ? '这是桌面装扮的灵感预览。' : item.kind === 'pet' ? '轻点伙伴，看看它的表情；移动鼠标，它也会看向你。' : item.kind === 'widget' ? '官方组件外观预览；示例内容不代表你的个人数据，天气与行情不是实时数据。' : '官方内置内容的网页预览。' }}</p>
      <p v-if="wide" class="browse-note">选择下方作品，继续探索。</p>
    </div>
  </div>
</template>
<style scoped>
.preview-content{display:grid;gap:12px;min-width:0}.preview-media{height:min(38dvh,320px);min-height:210px;overflow:hidden;border-radius:12px;background:var(--surface-2)}.preview-media img{display:block;width:100%;height:100%;object-fit:contain}.preview-copy{display:grid;align-content:center;gap:12px;min-width:0}.preview-copy p{margin:0;line-height:1.7;font-size:14px}.preview-copy .detail-kind,.preview-copy .detail-note{color:var(--ink-3);font-size:12px}.detail-kind span{margin:0 8px}.btn{min-height:44px;justify-content:center}.wide{grid-template-columns:minmax(0,1.7fr) minmax(250px,1fr);gap:0;background:var(--surface);border:1px solid var(--line);border-radius:24px;overflow:hidden}.wide .preview-media{height:320px;min-height:0;border-radius:0;background:color-mix(in srgb,var(--brand) 4%,var(--surface-2))}.wide .preview-media.game img,.wide .preview-media.wallpaper img{object-fit:cover}.wide .preview-media.pet{background:radial-gradient(ellipse at 50% 60%,var(--surface) 0%,color-mix(in srgb,var(--brand) 7%,var(--surface-2)) 80%)}.wide .preview-copy{padding:30px;gap:18px}.wide h2{margin:0;font-size:30px;line-height:1.3;letter-spacing:-.03em}.wide .description{font-size:15px;color:var(--ink-2);line-height:1.9}.wide .detail-kind{font-size:13px;color:var(--brand)}.wide .detail-note{font-size:13px}.wide .btn{justify-self:start;gap:28px;padding:12px 22px;border-radius:12px}.wide .browse-note{padding-top:16px;border-top:1px solid var(--line);font-size:12px;color:var(--ink-3)}
@media(min-width:1500px){.wide .preview-media{height:380px}.wide .preview-copy{padding:40px}.wide h2{font-size:36px}}
</style>
