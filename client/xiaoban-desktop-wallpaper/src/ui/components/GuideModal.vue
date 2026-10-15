<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NModal, NTabs, NTabPane, NTag } from 'naive-ui';
import { ArrowUpRight, Check, Copy, ExternalLink, Sparkles, Wand2, Monitor, Coffee, FolderOpen, Settings2 } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';

const props = defineProps<{
  show: boolean;
  desktop?: boolean;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  'select-prompt': [prompt: string];
  'open-settings': [];
}>();

const activeTab = ref('creation');
const copiedIndex = ref<string | null>(null);

interface GuideCard {
  id: string;
  badge: string;
  badgeColor?: string;
  title: string;
  desc: string;
  prompt: string;
}

const creationCards: GuideCard[] = [
  {
    id: 'c1',
    badge: '现场造挂件',
    title: '极简赛博朋克时钟',
    desc: '让小伴现场手写 HTML/JS 组件，立即自动挂载到桌面上。',
    prompt: '帮我写一个极简赛博朋克风格的时钟挂件，放在桌面上',
  },
  {
    id: 'c2',
    badge: '现场造挂件',
    title: '下班倒计时便签',
    desc: '自定义标题和倒计时目标，支持自定义样式与微光动画。',
    prompt: '帮我写一个下班倒计时便签挂件，写上“下班不慌，慢慢来”，放到桌面',
  },
  {
    id: 'c3',
    badge: '现场写壁纸',
    title: '暗黑星轨 Canvas 壁纸',
    desc: '让小伴现场编写动态 Canvas 壁纸代码，并一键应用到当前桌面。',
    prompt: '帮我写一个暗黑流动星轨的动态壁纸，带上微光粒子，做完直接应用到桌面',
  },
  {
    id: 'c4',
    badge: '粒子特效',
    title: '春日微风落樱特效',
    desc: '编写粒子漂浮动画特效包，落花在屏幕上静静飘动。',
    prompt: '帮我做一个春日微风带着粉色樱花瓣飘落的桌面特效，并切换到它',
  },
];

const controlCards: GuideCard[] = [
  {
    id: 'd1',
    badge: '壁纸联动',
    title: '沉浸风景搭配',
    desc: '同时切换壁纸与桌面氛围粒子，一句话完成桌面换装。',
    prompt: '换一张安静的雪山风景壁纸，并开启落雪特效',
  },
  {
    id: 'd2',
    badge: '小伴互动',
    title: '外观与情绪调整',
    desc: '切换小伴的形象皮肤、神态表情，或者隐藏/唤出小伴。',
    prompt: '把小伴换成毛绒小猫皮肤，做一个开心的表情',
  },
  {
    id: 'd3',
    badge: '快捷游戏',
    title: '摸鱼放松片刻',
    desc: '一句话唤起水墨剑客、俄罗斯方块、贪吃蛇等离线小游戏。',
    prompt: '打开水墨剑客小游戏，我想放松一下',
  },
];

const lifeCards: GuideCard[] = [
  {
    id: 'l1',
    badge: '健康作息',
    title: '定时饮水吃水果',
    desc: '自动录入系统关怀提醒规则，到点以温和气泡通知，不打扰全屏沉浸。',
    prompt: '每天下午三点半提醒我喝水吃点水果',
  },
  {
    id: 'l2',
    badge: '专注关怀',
    title: '防久坐与颈椎提醒',
    desc: '设定周期性休息间隔，工作再忙也别忘了起身活动。',
    prompt: '工作每隔 45 分钟提醒我闭眼休息，活动活动颈椎',
  },
  {
    id: 'l3',
    badge: '生活管家',
    title: '下班提醒与天气',
    desc: '查询天气变化与穿衣指引，或是提醒准时下班。',
    prompt: '每天下午六点半提醒我：准备下班，今天辛苦啦',
  },
];

const workspaceCards: GuideCard[] = [
  {
    id: 'w1',
    badge: '本地文件',
    title: '工作区文件整理',
    desc: '分析工作目录结构，读取或汇总代码文件与日常笔记。',
    prompt: '帮我看看当前工作目录里有什么文件，整理一份清晰的清单',
  },
  {
    id: 'w2',
    badge: '视觉识图',
    title: '粘贴截图一键分析',
    desc: '直接 Ctrl+V / Cmd+V 粘贴错误截图、UI 设计稿，最多支持 8 张图并发分析。',
    prompt: '看下我刚粘贴的截图，分析一下报错的原因和解决办法',
  },
  {
    id: 'w3',
    badge: '拟真语音',
    title: 'Edge-TTS 自然朗读',
    desc: '支持几十种近乎真人的中文与方言音色，随时收听小伴回答与诗意文字。',
    prompt: '念一段温柔的晚安寄语给我听吧',
  },
];

function tryPrompt(text: string) {
  emit('select-prompt', text);
  emit('update:show', false);
}

async function copyPrompt(text: string, id: string) {
  try {
    await navigator.clipboard.writeText(text);
    copiedIndex.value = id;
    setTimeout(() => {
      if (copiedIndex.value === id) copiedIndex.value = null;
    }, 2000);
  } catch {
    /* ignore clipboard error */
  }
}

async function openOnlineGuide() {
  const url = 'https://wallpaper.xiangban.xyz/guide';
  if (props.desktop) {
    try {
      await invoke('open_official_site');
    } catch {
      window.open(url, '_blank');
    }
  } else {
    window.open(url, '_blank');
  }
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    title="小伴桌面 · AI 玩法指南"
    style="width: min(94vw, 720px); max-height: 88vh; display: flex; flex-direction: column;"
    class="guide-modal"
    @update:show="emit('update:show', $event)"
  >
    <div class="guide-lead">
      <div class="lead-icon"><Sparkles :size="20" /></div>
      <div class="lead-copy">
        <strong>不止是聊天，更是你的桌面全能管家</strong>
        <p>小伴接入了系统控制与创造引擎，能直接替你写挂件、换壁纸、调特效、设提醒。点击任意示例即可一键填入试试。</p>
      </div>
    </div>

    <NTabs v-model:value="activeTab" type="line" animated class="guide-tabs">
      <NTabPane name="creation" tab="🎨 现场创造">
        <div class="guide-grid">
          <div v-for="item in creationCards" :key="item.id" class="guide-card">
            <div class="card-top">
              <span class="guide-badge">{{ item.badge }}</span>
              <h4>{{ item.title }}</h4>
            </div>
            <p class="card-desc">{{ item.desc }}</p>
            <div class="card-prompt">
              <code>{{ item.prompt }}</code>
            </div>
            <div class="card-actions">
              <NButton size="tiny" secondary @click="copyPrompt(item.prompt, item.id)">
                <template #icon>
                  <Check v-if="copiedIndex === item.id" :size="12" />
                  <Copy v-else :size="12" />
                </template>
                {{ copiedIndex === item.id ? '已复制' : '复制' }}
              </NButton>
              <NButton size="tiny" type="primary" @click="tryPrompt(item.prompt)">
                <template #icon><Wand2 :size="12" /></template>
                一键试试
              </NButton>
            </div>
          </div>
        </div>
      </NTabPane>

      <NTabPane name="control" tab="🪄 桌面控制">
        <div class="guide-grid">
          <div v-for="item in controlCards" :key="item.id" class="guide-card">
            <div class="card-top">
              <span class="guide-badge blue">{{ item.badge }}</span>
              <h4>{{ item.title }}</h4>
            </div>
            <p class="card-desc">{{ item.desc }}</p>
            <div class="card-prompt">
              <code>{{ item.prompt }}</code>
            </div>
            <div class="card-actions">
              <NButton size="tiny" secondary @click="copyPrompt(item.prompt, item.id)">
                <template #icon>
                  <Check v-if="copiedIndex === item.id" :size="12" />
                  <Copy v-else :size="12" />
                </template>
                {{ copiedIndex === item.id ? '已复制' : '复制' }}
              </NButton>
              <NButton size="tiny" type="primary" @click="tryPrompt(item.prompt)">
                <template #icon><Wand2 :size="12" /></template>
                一键试试
              </NButton>
            </div>
          </div>
        </div>
      </NTabPane>

      <NTabPane name="life" tab="🍵 生活关怀">
        <div class="guide-grid">
          <div v-for="item in lifeCards" :key="item.id" class="guide-card">
            <div class="card-top">
              <span class="guide-badge green">{{ item.badge }}</span>
              <h4>{{ item.title }}</h4>
            </div>
            <p class="card-desc">{{ item.desc }}</p>
            <div class="card-prompt">
              <code>{{ item.prompt }}</code>
            </div>
            <div class="card-actions">
              <NButton size="tiny" secondary @click="copyPrompt(item.prompt, item.id)">
                <template #icon>
                  <Check v-if="copiedIndex === item.id" :size="12" />
                  <Copy v-else :size="12" />
                </template>
                {{ copiedIndex === item.id ? '已复制' : '复制' }}
              </NButton>
              <NButton size="tiny" type="primary" @click="tryPrompt(item.prompt)">
                <template #icon><Wand2 :size="12" /></template>
                一键试试
              </NButton>
            </div>
          </div>
        </div>
      </NTabPane>

      <NTabPane name="workspace" tab="📁 助手与多模态">
        <div class="guide-grid">
          <div v-for="item in workspaceCards" :key="item.id" class="guide-card">
            <div class="card-top">
              <span class="guide-badge purple">{{ item.badge }}</span>
              <h4>{{ item.title }}</h4>
            </div>
            <p class="card-desc">{{ item.desc }}</p>
            <div class="card-prompt">
              <code>{{ item.prompt }}</code>
            </div>
            <div class="card-actions">
              <NButton size="tiny" secondary @click="copyPrompt(item.prompt, item.id)">
                <template #icon>
                  <Check v-if="copiedIndex === item.id" :size="12" />
                  <Copy v-else :size="12" />
                </template>
                {{ copiedIndex === item.id ? '已复制' : '复制' }}
              </NButton>
              <NButton size="tiny" type="primary" @click="tryPrompt(item.prompt)">
                <template #icon><Wand2 :size="12" /></template>
                一键试试
              </NButton>
            </div>
          </div>
        </div>
      </NTabPane>

      <NTabPane name="models" tab="⚙️ 模型接入">
        <div class="models-guide">
          <div class="models-box">
            <h4>💡 模型推荐配置</h4>
            <ul>
              <li><strong>DeepSeek (推荐首选)</strong>：极高性价比与出色推理能力，模型填 <code>deepseek-chat</code> 或 <code>deepseek-reasoner</code>。</li>
              <li><strong>智谱 GLM / Kimi / MiniMax</strong>：大厂官方端点支持，长上下文与多模态兼容性良好。</li>
              <li><strong>本地 Ollama (完全离线)</strong>：端点填 <code>http://localhost:11434/v1</code>，密钥留空，数据 100% 留存在本机。</li>
            </ul>
          </div>
          <div class="safety-box">
            <h4>🛡️ 透明安全的权限机制</h4>
            <p>小伴在读取文件时自动执行；而在<strong>写入本地文件、修改系统设置、执行系统命令</strong>时，会先弹出授权卡片。每次操作清晰可见，随时掌控。</p>
          </div>
          <div class="models-action">
            <NButton type="primary" secondary @click="emit('open-settings'); emit('update:show', false);">
              <template #icon><Settings2 :size="14" /></template>
              前往“设置 → AI 模型”配置
            </NButton>
          </div>
        </div>
      </NTabPane>
    </NTabs>

    <template #footer>
      <div class="guide-footer">
        <span class="footer-hint">想了解完整的代码格式与进阶功能？</span>
        <button type="button" class="web-guide-link" @click="openOnlineGuide">
          <span>访问官网完整手册</span>
          <ArrowUpRight :size="14" />
        </button>
      </div>
    </template>
  </NModal>
</template>

<style scoped>
.guide-lead{display:flex;gap:14px;align-items:flex-start;background:var(--ui-surface-alt);border:1px solid var(--ui-border);border-radius:14px;padding:12px 16px;margin-bottom:16px}
.lead-icon{color:var(--ui-primary);background:var(--ui-surface);border-radius:10px;width:34px;height:34px;display:grid;place-items:center;flex-shrink:0;box-shadow:0 2px 6px rgba(0,0,0,.04)}
.lead-copy strong{font-size:14px;color:var(--ui-text-primary);display:block;margin-bottom:3px}
.lead-copy p{font-size:12.5px;color:var(--ui-text-secondary);margin:0;line-height:1.6}
.guide-tabs :deep(.n-tabs-nav){margin-bottom:12px}
.guide-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;max-height:50vh;overflow-y:auto;padding-right:4px}
.guide-card{background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:12px;padding:12px 14px;display:flex;flex-direction:column;gap:8px;transition:border-color .2s}
.guide-card:hover{border-color:var(--ui-primary)}
.card-top{display:flex;align-items:center;gap:8px}
.guide-badge{font-size:11px;padding:1px 6px;border-radius:6px;background:var(--ui-primary-soft);color:var(--ui-primary);font-weight:600}
.guide-badge.blue{background:#e0f2fe;color:#0284c7}
.guide-badge.green{background:#ecfdf5;color:#059669}
.guide-badge.purple{background:#f3e8ff;color:#7c3aed}
.card-top h4{margin:0;font-size:14px;font-weight:600;color:var(--ui-text-primary)}
.card-desc{font-size:12px;color:var(--ui-text-secondary);line-height:1.6;margin:0}
.card-prompt{background:var(--ui-surface-alt);border-radius:8px;padding:8px 10px;margin-top:auto}
.card-prompt code{font-size:12px;color:var(--ui-text-primary);font-family:inherit;line-height:1.5;display:block}
.card-actions{display:flex;justify-content:flex-end;gap:8px;margin-top:4px}
.models-guide{display:grid;gap:14px;font-size:13px;line-height:1.7;color:var(--ui-text-secondary)}
.models-box,.safety-box{background:var(--ui-surface-alt);border:1px solid var(--ui-border);border-radius:12px;padding:12px 16px}
.models-box h4,.safety-box h4{margin:0 0 8px;font-size:14px;color:var(--ui-text-primary)}
.models-box ul{margin:0;padding-left:18px}
.models-box li{margin-bottom:6px}
.models-box code{background:var(--ui-surface);padding:2px 6px;border-radius:4px;font-size:12px;color:var(--ui-primary)}
.safety-box p{margin:0}
.models-action{display:flex;justify-content:center;margin-top:4px}
.guide-footer{display:flex;align-items:center;justify-content:space-between;width:100%;font-size:12.5px}
.footer-hint{color:var(--ui-text-tertiary)}
.web-guide-link{display:inline-flex;align-items:center;gap:4px;background:none;border:0;color:var(--ui-primary);cursor:pointer;font:inherit;font-weight:600;padding:0}
.web-guide-link:hover{text-decoration:underline}
@media(max-width:640px){.guide-grid{grid-template-columns:1fr}}
</style>
