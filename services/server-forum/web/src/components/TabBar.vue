<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { areaOf } from '@/lib/sections'
import { useTheme, THEME_LABEL } from '@/lib/theme'
import BrandMark from './BrandMark.vue'
import AppIcon from './AppIcon.vue'
import ModalDialog from './ModalDialog.vue'
const route = useRoute(), router = useRouter(), { preference, openAppearance } = useTheme()
const area = computed(() => areaOf(route)), publishing = ref(false)
const links = [{to:'/ai',key:'ai',label:'小伴 AI'}, {to:'/create',key:'create',label:'创造'}, {to:'/',key:'community',label:'社区'}, {to:'/me',key:'me',label:'我的'}]
function publish(path: string) { publishing.value = false; router.push(path) }
</script>
<template>
  <nav class="primary-nav" aria-label="主导航">
    <RouterLink class="nav-brand" to="/"><BrandMark /><span>超级小伴</span></RouterLink>
    <div class="nav-items">
      <template v-for="(item,index) in links" :key="item.key">
        <button v-if="index === 2" type="button" class="nav-publish" :class="{active:publishing}" aria-label="发布" @click="publishing = true"><AppIcon name="plus" /><span>发布</span></button>
        <RouterLink :to="item.to" :class="['nav-item',{active:area === item.key && !publishing}]" :aria-label="item.label" :aria-current="area === item.key ? 'page' : undefined"><AppIcon :name="item.key === 'ai' ? 'chat' : item.key === 'create' ? 'ai' : item.key === 'community' ? 'grid' : item.key" /><span>{{ item.label }}</span></RouterLink>
      </template>
    </div>
    <div class="nav-bottom">
      <button @click="openAppearance"><AppIcon name="sun" :size="18" />外观 · {{ THEME_LABEL[preference] }}</button>
      <a href="https://wallpaper.xiangban.xyz" target="_blank" rel="noopener noreferrer"><AppIcon name="download" :size="18" />下载客户端</a>
      <div><RouterLink to="/rules">社区规则</RouterLink><RouterLink to="/privacy">隐私政策</RouterLink></div>
    </div>
  </nav>
  <ModalDialog :open="publishing" title="分享一点新鲜事" @close="publishing = false">
    <div class="publish-options">
      <button @click="publish('/new')"><span class="option-icon"><AppIcon name="community" /></span><span><b>发布帖子</b><small>交流想法、分享经验或发起求助</small></span><AppIcon name="arrow" :size="18" /></button>
      <button @click="publish('/publish')"><span class="option-icon"><AppIcon name="upload" /></span><span><b>分享已有作品</b><small>上传你在客户端完成的作品资源包</small></span><AppIcon name="arrow" :size="18" /></button>
    </div>
  </ModalDialog>
</template>
<style scoped>
.primary-nav{position:fixed;z-index:35;left:0;top:0;bottom:0;width:216px;background:var(--surface);display:flex;flex-direction:column;padding:28px 20px}
.nav-brand{display:flex;align-items:center;gap:10px;font-size:19px;font-weight:750;padding:0 8px 32px;white-space:nowrap}.nav-brand :deep(svg){width:30px;height:30px}
.nav-items{display:flex;flex-direction:column;gap:8px}.nav-item,.nav-publish{display:flex;align-items:center;gap:14px;min-height:48px;border-radius:14px;padding:0 16px;color:var(--ink-2);border:0;background:none;font-size:15px;font-weight:550}
.nav-item:hover{background:var(--surface-2)}.nav-item.active{background:var(--brand-soft);color:var(--brand);font-weight:650}
.nav-publish{order:5;margin-top:16px;background:var(--brand-soft);color:var(--brand)}
.nav-bottom{margin-top:auto;display:grid;gap:8px;padding:24px 8px 0;font-size:13px;color:var(--ink-3)}.nav-bottom>button,.nav-bottom>a{display:flex;align-items:center;gap:10px;min-height:40px;background:none;border:0;padding:0;text-align:left}.nav-bottom>div{display:flex;gap:12px;margin-top:16px;font-size:12px}
.publish-options{display:grid;gap:8px}.publish-options>button{display:flex;align-items:center;gap:12px;width:100%;text-align:left;padding:16px 12px;border:0;border-radius:12px;background:var(--surface-2)}.publish-options>button>span:nth-child(2){flex:1}.publish-options b{display:block;font-size:15px;font-weight:600}.publish-options small{display:block;font-size:12px;color:var(--ink-3);margin-top:3px}.option-icon{color:var(--brand)}
@media(max-width:1023px){.primary-nav{top:auto;left:20px;right:20px;bottom:calc(14px + env(safe-area-inset-bottom));width:auto;height:60px;border-radius:30px;padding:0 8px;background:var(--surface);box-shadow:0 8px 28px #00000012;backdrop-filter:blur(16px)}.nav-brand,.nav-bottom{display:none}.nav-items{display:flex;flex-direction:row;justify-content:space-between;gap:0;align-items:center;height:100%}.nav-item,.nav-publish{order:initial;display:flex;flex-direction:row;justify-content:center;align-items:center;gap:6px;min-width:0;width:44px;min-height:44px;height:44px;padding:0;margin:0;border-radius:22px;font-size:14px;flex-shrink:1;background:none;color:var(--ink-2)}.nav-item>span,.nav-publish>span{display:none}.nav-item.active,.nav-publish.active{width:auto;flex-shrink:0;padding:0 14px;color:var(--brand-ink);background:var(--brand-fill)}.nav-item.active>span,.nav-publish.active>span{display:block;white-space:nowrap}.nav-item>svg,.nav-publish>svg{flex:none;width:24px;height:24px}}
@media(min-width:768px) and (max-width:1023px){.primary-nav{max-width:540px;margin:auto}}
@media(min-width:1100px){
.primary-nav{width:200px;padding:28px 16px;border-right:1px solid var(--line)}.nav-brand{font-size:18px;padding:0 8px 36px;gap:8px}.nav-brand :deep(svg){width:28px;height:28px}.nav-items{gap:6px}.nav-item,.nav-publish{border-radius:10px;font-size:14px;min-height:46px;gap:12px}.nav-publish{margin-top:24px;border:1px solid var(--line);background:var(--surface)}.nav-bottom{padding-left:8px;padding-right:0;font-size:12px}.nav-bottom>div{gap:10px}
}
</style>
