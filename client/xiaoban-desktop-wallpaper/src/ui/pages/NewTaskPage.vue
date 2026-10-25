<script setup lang="ts">
/**
 * 新建任务: a greeting, scenes with quick starts, the message box and a few things to try.
 * The first message creates the conversation; the shell then moves to the `chat` page.
 */
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { ArrowUpRight } from 'lucide-vue-next';
import type { ControlStore } from '../control-types.ts';
import ChatComposer from '../components/ChatComposer.vue';
import ChatExpertChip from '../components/ChatExpertChip.vue';

const props = defineProps<{ store: ControlStore }>();
const chat = props.store.chat;
const composer = ref<InstanceType<typeof ChatComposer> | null>(null);

/* The greeting follows the clock, checked again every few minutes while the page stays open. */
const hour = ref(new Date().getHours());
let clock: ReturnType<typeof setInterval> | undefined;
onMounted(() => { void chat.init(); clock = setInterval(() => { hour.value = new Date().getHours(); }, 5 * 60_000); });
onUnmounted(() => clearInterval(clock));
const greeting = computed(() => (hour.value >= 5 && hour.value < 12 ? '上午好' : hour.value >= 12 && hour.value < 18 ? '下午好' : '晚上好'));

type SceneId = 'work' | 'study' | 'life' | 'create';
interface Scene { id: SceneId; label: string; chips: { label: string; prompt: string }[]; tries: string[] }
const scenes: Scene[] = [
  { id: 'work', label: '办公', chips: [
    { label: '写文档', prompt: '帮我写一份文档，主题是：' },
    { label: '做表格', prompt: '帮我做一张表格，内容是：' },
    { label: '整理文件', prompt: '帮我整理工作区里的文件：按类型分到不同文件夹，并列一份清单' },
    { label: '截图识别', prompt: '帮我识别截图里的文字和表格，整理成可以直接复制的内容' },
    { label: '深度研究', prompt: '帮我深入研究这个问题，给出有来源的结论：' },
    { label: '定时任务', prompt: '每个工作日早上 9:00 提醒我：' },
  ], tries: ['把这周的工作记录整理成周报', '下载文件夹里的发票按月份归档', '每个工作日 9:00 提醒我看今天的日程'] },
  { id: 'study', label: '学习', chips: [
    { label: '讲解概念', prompt: '用通俗的例子给我讲讲：' },
    { label: '整理笔记', prompt: '把这些内容整理成结构清楚的学习笔记：' },
    { label: '出练习题', prompt: '根据下面的内容给我出 5 道练习题，附答案和解析：' },
    { label: '翻译', prompt: '把下面这段翻译成中文，保留专业术语：' },
    { label: '读论文', prompt: '帮我读这篇论文，总结研究问题、方法和结论：' },
    { label: '学习计划', prompt: '帮我制定一个四周的学习计划，目标是：' },
  ], tries: ['用费曼学习法给我讲讲什么是熵', '把这份讲义整理成复习提纲', '帮我做一个四周的英语听力练习计划'] },
  { id: 'life', label: '生活', chips: [
    { label: '喝水提醒', prompt: '每天下午三点提醒我喝水' },
    { label: '换壁纸', prompt: '把壁纸换成一张安静的风景' },
    { label: '旅行规划', prompt: '帮我规划一个周末两天的行程，目的地是：' },
    { label: '今天吃什么', prompt: '根据冰箱里有的食材推荐三道家常菜：' },
    { label: '记账', prompt: '帮我把这些花销整理成一张记账表：' },
    { label: '天气', prompt: '查一下今天的天气，告诉我要不要带伞' },
  ], tries: ['每天下午三点提醒我喝水', '把壁纸换成一张安静的风景', '查一下今天是几号，再告诉我这周还剩几天'] },
  { id: 'create', label: '创作', chips: [
    { label: '写文案', prompt: '帮我写一段文案，用在：' },
    { label: '起标题', prompt: '给下面的内容起 10 个吸引人的标题：' },
    { label: '桌面特效', prompt: '帮我创作一个桌面特效：' },
    { label: '写故事', prompt: '写一个温暖的小故事，主角是：' },
    { label: '润色', prompt: '把这段话润色得更自然一些：' },
    { label: '起名字', prompt: '帮我起几个名字，用在：' },
  ], tries: ['帮我创作一个桌面特效：飘落的樱花', '给我的咖啡店写三条朋友圈文案', '把这段话润色得更自然一些'] },
];
const SCENE_KEY = 'xiaoban.new-task.scene';
function savedScene(): SceneId {
  try { const saved = localStorage.getItem(SCENE_KEY); return scenes.some(s => s.id === saved) ? saved as SceneId : 'work'; } catch { return 'work'; }
}
const sceneId = ref<SceneId>(savedScene());
const scene = computed(() => scenes.find(s => s.id === sceneId.value) ?? scenes[0]);
function pickScene(id: SceneId) { sceneId.value = id; try { localStorage.setItem(SCENE_KEY, id); } catch { /* remembered for this visit only */ } }
const fill = (text: string) => composer.value?.setDraft(text);
/** 召唤了专家时，页面换成它的介绍和它自己的三条推荐提问（WorkBuddy 的 quickPrompts），第一条就是默认引导语。 */
const expert = computed(() => chat.pendingExpert);
const expertPrompts = computed(() => (expert.value?.quickPrompts ?? []).slice(0, 3));
function openAiSettings() { props.store.openSettings('ai'); }
</script>

<template>
  <div class="new-task-page" data-testid="new-task-page" data-tauri-drag-region>
    <div class="hero">
      <section v-if="expert" class="expert-intro" data-testid="new-task-expert">
        <ChatExpertChip :expert="expert" :removable="true" :desktop="store.desktop" @clear="chat.pendingExpert = null" />
        <h1 class="greeting">{{ expert.name }}<small v-if="expert.profession">{{ expert.profession }}</small></h1>
        <p v-if="expert.description" class="expert-desc">{{ expert.description }}</p>
        <div v-if="expert.tags?.length" class="expert-tags"><span v-for="tag in expert.tags" :key="tag">{{ tag }}</span></div>
      </section>
      <h1 v-else class="greeting" data-testid="new-task-greeting">{{ greeting }}，今天让小伴帮你做点什么？</h1>
      <div v-if="!expert" class="scenes" role="tablist" aria-label="场景">
        <button v-for="item in scenes" :key="item.id" type="button" role="tab" :aria-selected="item.id === sceneId" :data-testid="`new-task-scene-${item.id}`" @click="pickScene(item.id)">{{ item.label }}</button>
      </div>

      <div v-if="!expert" class="chips" role="tabpanel" :aria-label="scene.label">
        <button v-for="chip in scene.chips" :key="chip.label" type="button" class="quick-chip" :data-testid="`new-task-chip-${chip.label}`" :disabled="!store.desktop || !chat.configured" @click="fill(chip.prompt)">{{ chip.label }}</button>
      </div>

      <ChatComposer ref="composer" class="hero-composer" :store="store" page="new-task" workspace pet="lg" :min-rows="3" placeholder="描述你想完成的事。可以添加文件、截图，或调用专家和技能" @settings="openAiSettings" />

      <p v-if="!store.desktop" class="setup-note" data-testid="new-task-preview-only">浏览器预览里只能看到界面，请在桌面应用中使用小伴 AI。</p>
      <p v-else-if="chat.ready && !chat.configured" class="setup-note" data-testid="new-task-unconfigured">先给小伴接上一个模型：登录后可直接用小伴 AI，也可以配置自己的模型服务。<button type="button" class="xb-link" data-testid="new-task-go-settings" @click="openAiSettings">去设置</button></p>

      <div class="tries">
        <span class="tries-cap">{{ expert ? '试试这样问我' : '试试这样问' }}</span>
        <button v-for="(text, n) in (expert ? expertPrompts : scene.tries)" :key="text" type="button" class="try" :data-testid="`new-task-try-${n}`" :disabled="!store.desktop || !chat.configured" @click="fill(text)">{{ text }}<ArrowUpRight :size="14" :stroke-width="2" /></button>
      </div>
    </div>
    <p class="disclaimer">内容由 AI 生成，请核对重要信息</p>
  </div>
</template>

<style scoped>
.new-task-page{flex:1;min-height:0;min-width:0;display:flex;flex-direction:column;align-items:center;overflow-y:auto;overflow-x:hidden;background:var(--ui-surface);position:relative}
.hero{width:100%;max-width:780px;padding:clamp(48px,14vh,128px) 40px 56px;box-sizing:border-box;display:flex;flex-direction:column;align-items:center}
.greeting{margin:0;font-size:30px;font-weight:600;letter-spacing:-.01em;line-height:1.3;text-align:center;color:var(--ui-text-primary)}
.scenes{margin-top:18px;display:inline-flex;padding:3px;border-radius:12px;background:var(--ui-control);gap:2px}
.scenes button{height:30px;padding:0 14px;border:0;border-radius:9px;background:transparent;font:inherit;font-size:13px;color:var(--ui-text-secondary);cursor:pointer}
.scenes button:hover{color:var(--ui-text-primary)}
.scenes button[aria-selected=true]{background:var(--ui-surface);color:var(--ui-text-primary);font-weight:500;box-shadow:0 0 0 .5px rgba(0,0,0,.06),0 1px 3px rgba(0,0,0,.08)}
html[data-theme=dark] .scenes button[aria-selected=true]{background:#48484C}
.chips{align-self:stretch;margin-top:44px;display:flex;flex-wrap:wrap;gap:8px}
.quick-chip{height:32px;padding:0 14px;border-radius:999px;border:0;background:var(--ui-surface);font:inherit;font-size:13px;color:var(--ui-text-primary);cursor:pointer;box-shadow:0 0 0 1px var(--ui-border),0 1px 2px rgba(0,0,0,.04)}
.quick-chip:hover:not(:disabled){box-shadow:0 0 0 1px var(--ui-border-strong),0 2px 6px rgba(0,0,0,.06)}
.quick-chip:disabled{color:var(--ui-text-disabled);cursor:not-allowed}
html[data-theme=dark] .quick-chip{background:var(--ui-surface-alt)}
.hero-composer{margin-top:14px}
.expert-intro{display:flex;flex-direction:column;align-items:center;gap:10px;margin-bottom:6px}
.expert-intro .greeting small{display:block;margin-top:6px;font-size:14px;font-weight:400;letter-spacing:0;color:var(--ui-text-secondary)}
.expert-desc{margin:0;max-width:560px;text-align:center;font-size:13.5px;line-height:1.7;color:var(--ui-text-secondary)}
.expert-tags{display:flex;flex-wrap:wrap;justify-content:center;gap:6px}
.expert-tags span{padding:2px 10px;border-radius:999px;background:var(--ui-control);font-size:12px;color:var(--ui-text-secondary)}
.setup-note{align-self:stretch;margin:12px 0 0;padding:9px 14px;border-radius:10px;background:var(--ui-surface-sunken);font-size:12.5px;line-height:1.6;color:var(--ui-text-secondary)}
.setup-note .xb-link{margin-left:6px;font-weight:500}
.tries{align-self:stretch;margin-top:36px;display:flex;flex-direction:column;align-items:flex-start;gap:2px}
.tries-cap{font-size:12px;font-weight:500;color:var(--ui-text-secondary);padding:0 14px 6px}
.try{display:inline-flex;align-items:center;gap:8px;height:36px;padding:0 14px;border-radius:10px;border:0;background:transparent;font:inherit;font-size:13.5px;color:var(--ui-text-primary);cursor:pointer;text-align:left}
.try:hover:not(:disabled){background:var(--ui-control)}
.try svg{flex:none;color:var(--ui-text-tertiary)}
.try:disabled{cursor:default}
.disclaimer{position:sticky;bottom:0;margin:auto 0 0;padding:10px 0 12px;font-size:11px;color:var(--ui-text-tertiary);text-align:center}
@media(max-width:760px){.hero{padding:40px 16px 48px}.greeting{font-size:24px}.chips{margin-top:28px}}
</style>
