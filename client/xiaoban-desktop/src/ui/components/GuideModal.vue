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
    badge: '现场写壁纸',
    title: '暗黑星轨 Canvas 壁纸',
    desc: '小伴现场编写单文件 HTML 壁纸（Canvas / WebGL / CSS 动效），自动存进壁纸库并应用。',
    prompt: '帮我写一个暗黑流动星轨的动态壁纸，带上微光粒子，做完直接应用到桌面',
  },
  {
    id: 'c2',
    badge: '现场造挂件',
    title: '自创单文件组件',
    desc: '让小伴现场编写 HTML/CSS/JS 小卡片（如极简时钟、番茄钟），自动存入自定义组件并放置到桌面。',
    prompt: '帮我写一个极简赛博朋克风格的时钟小组件，放在桌面上',
  },
  {
    id: 'c3',
    badge: '粒子特效',
    title: '春日微风落樱特效',
    desc: '按特效包格式写一个粒子层，保存后就能像内置特效一样选用。',
    prompt: '帮我做一个春日微风带着粉色樱花瓣飘落的桌面特效，并切换到它',
  },
  {
    id: 'c4',
    badge: '现场造游戏',
    title: '自创一款离线小游戏',
    desc: '用 HTML/CSS/JS 写进游戏文件夹，支持存档、成绩和桌面层玩法，内置七款之外还能继续加。',
    prompt: '帮我做一个接星星的小游戏，接住掉下来的星星会加分，玩的时候可以存档',
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
    title: '外壳与表情调整',
    desc: '两款盆栽外壳（绿叶、白色小雏菊）与 25 种表情可换，也能让小伴先藏起来。',
    prompt: '把小伴换成白色小雏菊外壳，做一个开心的表情',
  },
  {
    id: 'd3',
    badge: '快捷游戏',
    title: '摸鱼放松片刻',
    desc: '一句话唤起水墨剑客、俄罗斯方块、贪吃蛇、2048、暮色滑翔等离线小游戏。',
    prompt: '打开水墨剑客小游戏，我想放松一下',
  },
  {
    id: 'd4',
    badge: '窗口摆放',
    title: '把桌面收拾一下',
    desc: '查询小伴自己的窗口与显示器，按比例移动盆栽、组件；也能收起超级小伴。',
    prompt: '把小伴和桌面组件的位置整理一下，别挡着右边的工作区',
  },
];

const lifeCards: GuideCard[] = [
  {
    id: 'l1',
    badge: '健康作息',
    title: '定时饮水吃水果',
    desc: '自动录入关怀提醒规则，到点以温和气泡通知，免打扰时段自动避让。',
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
    badge: '下班提醒',
    title: '准时下班',
    desc: '按工作日或指定星期提醒收尾，给一天一个明确的结束。',
    prompt: '周一到周五下午六点半提醒我：准备下班，今天辛苦啦',
  },
  {
    id: 'l4',
    badge: '一次提醒',
    title: '临时事项不忘',
    desc: '一次性的提醒同样能安排，例如会议、吃药或某个具体日期的约定。',
    prompt: '明天上午十点提醒我给客户回电话',
  },
];

const workspaceCards: GuideCard[] = [
  {
    id: 'w1',
    badge: '本地工作区',
    title: '工作区文件整理',
    desc: '在设置里选定工作目录后，读取、搜索、写入文件或运行命令，范围之外不碰。',
    prompt: '帮我看看当前工作目录里有什么文件，整理一份清晰的清单',
  },
  {
    id: 'w2',
    badge: '视觉识图',
    title: '粘贴截图一键分析',
    desc: '直接 Ctrl+V / Cmd+V 粘贴错误截图或 UI 设计稿，一条消息最多 8 张图。',
    prompt: '看下我刚粘贴的截图，分析一下报错的原因和解决办法',
  },
  {
    id: 'w3',
    badge: '本机语音',
    title: '语音提问与自然朗读',
    desc: 'SenseVoice Small 在本机识别语音，转成文字后再由你确认发送；回答可用 Edge-TTS 音色朗读。',
    prompt: '念一段温柔的晚安寄语给我听吧',
  },
  {
    id: 'w4',
    badge: '电脑控制',
    title: '操作其他应用',
    desc: '在“设置 → AI 模型”开启电脑屏幕与应用控制后，小伴可以读取窗口、点击和输入文字。',
    prompt: '帮我把浏览器里当前这个页面保存成 PDF，放到工作目录',
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
        <p>小伴接入了桌面装扮与创作引擎，能直接替你写壁纸、调特效、放组件、设提醒，也能读写工作区、操作其他应用。点击任意示例即可一键填入试试。</p>
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

      <NTabPane name="control" tab="🪄 桌面装扮">
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
            <h4>💡 两种用起来的方式</h4>
            <ul>
              <li><strong>小伴 AI（最省事）</strong>：在“AI 模型”里登录小伴账号就能直接用，不用自己申请密钥；新用户有试用额度，可用兑换码充值，模型档位由小伴后台维护。</li>
              <li><strong>自己的服务商</strong>：内置 DeepSeek、智谱 BigModel、Kimi、MiniMax 预设，填 API 密钥即可；也可以选“自定义供应商”，接第三方端点或本地部署。</li>
              <li><strong>本地 Ollama（完全离线）</strong>：自定义供应商地址填 <code>http://localhost:11434/v1</code>，密钥留空，协议选 OpenAI 兼容（Chat Completions）。</li>
            </ul>
          </div>
          <div class="safety-box">
            <h4>🛡️ 透明的权限机制</h4>
            <p>聊天顶栏有两种模式。<strong>默认权限</strong>下，读操作和小伴自己的操作（换壁纸、放组件、移动小伴窗口、改表情、开游戏等）直接执行，写文件、运行命令、修改设置前先弹出确认卡片；<strong>完全信任</strong>下不再逐一询问，只在提权、批量删除、<code>git push --force</code> 这类高风险操作时停下来。完全信任下可访问工作区外的文件，并以当前用户权限执行命令，支持全局 npm、brew 等操作；不会自动获得管理员或系统隐私权限。设置中的命令沙箱默认关闭，手动开启后才限制命令的写入目录。每个对话各自记住自己的模式。</p>
          </div>
          <div class="safety-box">
            <h4>🧠 上下文与压缩</h4>
            <p>输入框旁的圆环显示当前对话的上下文占用，可以调节上限（256K 到 1M）。超出时内核会自动省略较早内容；“智能压缩”让模型把旧对话写成摘要，自动压缩在回答结束后用量达到上限 90% 时触发。</p>
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
/* NModal 注入的卡片/标签内部节点不带 scoped 属性，只能用 :global 限定在 .guide-modal 下 */
:global(.guide-modal > .n-card-content){flex:1 1 auto;min-height:0;overflow:hidden;display:flex;flex-direction:column}
:global(.guide-modal > .n-card-header),:global(.guide-modal > .n-card__footer){flex:none}
:global(.guide-modal > .n-card__footer){padding-top:0}
.guide-tabs{flex:1 1 auto;min-height:0;display:flex;flex-direction:column}
:global(.guide-modal .n-tabs-nav){flex:none;margin-bottom:12px}
:global(.guide-modal .n-tabs-pane-wrapper){flex:1 1 auto;min-height:0;overflow:hidden;display:flex;flex-direction:column}
:global(.guide-modal .n-tab-pane){flex:1 1 auto;min-height:0;overflow-y:auto;overscroll-behavior:contain;padding-right:4px}
.guide-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:12px;align-content:start}
.guide-card{background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:12px;padding:12px 14px;display:flex;flex-direction:column;gap:8px;transition:border-color .2s}
.guide-card:hover{border-color:var(--ui-primary)}
.card-top{display:flex;align-items:center;gap:8px}
.guide-badge{font-size:11px;padding:1px 6px;border-radius:6px;background:var(--ui-primary-soft);color:var(--ui-primary);font-weight:600}
.guide-badge.blue{background:#e0f2fe;color:#0284c7}
.guide-badge.green{background:#ecfdf5;color:#059669}
.guide-badge.purple{background:#f3e8ff;color:#7c3aed}
:root[data-theme=dark] .guide-badge.blue{background:#1E2F44;color:#8FC1F2}
:root[data-theme=dark] .guide-badge.green{background:#1D3328;color:#72D6A2}
:root[data-theme=dark] .guide-badge.purple{background:#2A2342;color:#BBA8FF}
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
@media(max-width:640px){.guide-grid{grid-template-columns:1fr}.guide-tabs{--n-tab-gap:18px}.guide-tabs :deep(.n-tabs-tab){padding-left:0;padding-right:0}}
</style>
