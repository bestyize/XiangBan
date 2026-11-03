<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { OpenSourceProjectSummary } from '@/lib/openSource'
import { resolveOpenSourceCover } from '@/lib/openSourceCovers'
const props = defineProps<{ project: OpenSourceProjectSummary }>()
const project = computed(() => resolveOpenSourceCover(props.project))
const brokenImage = ref(false), brokenIcon = ref(false)
watch(() => [props.project.full_name, props.project.cover_url, props.project.icon_url], () => { brokenImage.value = false; brokenIcon.value = false })
const style = computed(() => !brokenImage.value && project.value.cover_url && project.value.cover_style !== 'illustration' ? project.value.cover_style : 'illustration')
const layout = computed(() => project.value.cover_layout)
const accent = computed(() => project.value.cover_accent)
const lines = computed(() => (project.value.cover_lines?.length ? project.value.cover_lines : project.value.tags).slice(0, 3))
</script>
<template>
  <div class="os-cover" :class="[`os-cover-${style}`, `os-cover-layout-${layout}`, `os-cover-variant-${project.cover_variant}`, { 'os-cover-offset': project.cover_variant === 'offset' }]" :style="{ '--cover-accent': accent, '--cover-focus': `${project.cover_focus_x}% ${project.cover_focus_y}%` }" aria-hidden="true">
    <img v-if="style === 'screen' || style === 'photo'" class="os-cover-image" :src="project.cover_url" alt="" loading="lazy" @error="brokenImage = true">
    <div class="os-cover-brand"><img v-if="project.icon_url && !brokenIcon" :class="{'os-icon-mark':project.icon_fit === 'mark-left'}" :src="project.icon_url" alt="" loading="lazy" @error="brokenIcon = true"><span>{{ project.name }}</span></div>
    <template v-if="style === 'illustration'">
      <span class="os-cover-caption">{{ layout === 'terminal' ? '命令示例' : layout === 'notes' ? '内容示意' : '流程示意' }}</span>
      <div v-if="layout === 'terminal'" class="os-cover-terminal"><span class="os-cover-terminal-label">{{ project.name.toLowerCase() }}</span><code>{{ lines[0] }}</code><div class="os-cover-terminal-meta">{{ lines.slice(1).join(' · ') }}</div><span class="os-cover-cursor" /></div>
      <div v-else-if="layout === 'notes'" class="os-cover-notes"><div v-for="(line, index) in lines" :key="line"><span class="os-cover-note-index">0{{ index + 1 }}</span><span>{{ line }}</span><i /></div></div>
      <div v-else class="os-cover-flow"><template v-for="(line, index) in lines" :key="line"><div class="os-cover-node"><span class="os-cover-node-mark" /><span>{{ line }}</span></div><svg v-if="index < lines.length - 1" viewBox="0 0 36 24" fill="none"><path d="M2 12h28m-6-6 6 6-6 6" /></svg></template></div>
    </template>
  </div>
</template>
<style scoped>
.os-cover{position:relative;width:100%;aspect-ratio:16/9;overflow:hidden;isolation:isolate;background:color-mix(in srgb,var(--cover-accent) 7%,var(--surface));border-bottom:1px solid var(--line);container-type:inline-size}
.os-cover-brand{position:absolute;left:5%;top:7%;z-index:2;display:flex;align-items:center;gap:8px;max-width:72%;color:var(--ink);font-size:clamp(12px,3.2cqw,16px);line-height:1.2;font-weight:500}
.os-cover-brand>img{width:clamp(24px,7cqw,36px);height:clamp(24px,7cqw,36px);object-fit:contain;background:#fff;border:1px solid #ededed;border-radius:8px;padding:4px;box-sizing:border-box}
.os-cover-brand>span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.os-cover-image{display:block;position:absolute;object-fit:cover}
.os-cover-screen .os-cover-image{left:8%;top:28%;width:88%;height:80%;object-position:left top;border:1px solid color-mix(in srgb,var(--cover-accent) 18%,var(--line));border-radius:8px 8px 0 0}
.os-cover-photo .os-cover-image{inset:0;width:100%;height:100%;object-position:var(--cover-focus,50% 46%)}
.os-cover-photo .os-cover-brand{top:auto;bottom:7%;background:#fff;color:#222;border:1px solid #ededed;padding:6px 10px;border-radius:10px;max-width:80%}
.os-cover-brand>img.os-icon-mark{width:20px;height:28px;border:0;padding:0;object-fit:cover;object-position:left}
.os-cover-caption{position:absolute;top:10%;right:5%;color:var(--ink-3);font-size:12px;line-height:1.4}
.os-cover-flow{position:absolute;left:7%;right:7%;top:36%;bottom:14%;display:flex;align-items:center;justify-content:center}
.os-cover-node{display:flex;flex:1;min-width:0;min-height:54%;flex-direction:column;justify-content:center;align-items:center;gap:8px;padding:10px 4px;border:1px solid color-mix(in srgb,var(--cover-accent) 35%,var(--line));border-radius:12px;background:var(--surface);color:var(--ink-2);font-size:clamp(12px,3.3cqw,16px);box-sizing:border-box}
.os-cover-node-mark{width:12px;height:12px;border:1.5px solid var(--cover-accent);border-radius:4px;opacity:.8}
.os-cover-node:nth-of-type(3) .os-cover-node-mark{border-radius:50%}.os-cover-flow>svg{flex:0 0 12%;width:12%;color:var(--cover-accent);opacity:.65;overflow:visible}.os-cover-flow>svg path{stroke:currentColor;stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}
.os-cover-offset .os-cover-node:first-child{transform:translateY(-12%)}.os-cover-offset .os-cover-node:last-child{transform:translateY(12%)}
.os-cover-terminal{position:absolute;left:7%;right:7%;top:31%;bottom:12%;padding:clamp(12px,4cqw,20px);border:1px solid var(--line);border-radius:10px;background:var(--surface);display:flex;flex-direction:column;justify-content:center;gap:10px;overflow:hidden}
.os-cover-terminal-label{font-family:var(--font-mono);font-size:12px;color:var(--ink-3)}
.os-cover-terminal code{font-family:var(--font-mono);font-size:clamp(12px,3cqw,16px);color:var(--ink);white-space:nowrap}
.os-cover-terminal-meta{font-size:12px;color:var(--ink-3)}.os-cover-cursor{position:absolute;bottom:14%;right:6%;width:6px;height:12px;background:var(--cover-accent);opacity:.45}
.os-cover-notes{position:absolute;left:12%;right:10%;top:31%;bottom:10%;display:flex;flex-direction:column;justify-content:center;gap:8px}
.os-cover-notes>div{display:flex;align-items:center;gap:10px;min-height:25%;padding:6px 12px;box-sizing:border-box;border:1px solid var(--line);border-radius:8px;background:var(--surface);font-size:clamp(12px,3cqw,15px);color:var(--ink-2)}
.os-cover-notes>div:nth-child(2){transform:translateX(-5%)}.os-cover-notes>div:nth-child(3){transform:translateX(3%)}.os-cover-note-index{font-family:var(--font-mono);font-size:12px;color:var(--ink-3)}.os-cover-notes i{width:18%;height:3px;border-radius:2px;background:var(--cover-accent);opacity:.15;margin-left:auto}
.os-cover-variant-balanced .os-cover-notes>div{transform:none}.os-cover-variant-offset.os-cover-screen .os-cover-image{left:4%}.os-cover-variant-wide.os-cover-screen .os-cover-image{left:0;top:25%;width:100%;height:75%;border-inline:0;border-radius:0}.os-cover-variant-wide .os-cover-flow{left:4%;right:4%}.os-cover-variant-wide .os-cover-notes{left:7%;right:7%}.os-cover-variant-wide .os-cover-notes>div{transform:none}.os-cover-variant-offset .os-cover-terminal{left:10%;right:4%}.os-cover-variant-wide .os-cover-terminal{left:4%;right:4%}
</style>
