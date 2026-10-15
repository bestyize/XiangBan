<script setup lang="ts">
import { ref } from 'vue'
import Icon from '@/components/Icon.vue'

const copied = ref<string | null>(null)

interface Spell {
  id: string
  badge: string
  title: string
  desc: string
  prompt: string
}

const creationSpells: Spell[] = [
  {
    id: 's1',
    badge: '现场造挂件',
    title: '赛博朋克极简时钟',
    desc: '让小伴现场手写代码并生成挂件，几秒内自动钉在桌面上，支持多屏拖动与缩放。',
    prompt: '帮我写一个极简赛博朋克风格的时钟挂件，放在桌面上',
  },
  {
    id: 's2',
    badge: '现场造挂件',
    title: '下班倒计时便签',
    desc: '自定义下班时间与温馨寄语，到点伴随微光提醒。',
    prompt: '帮我写一个下班倒计时便签挂件，写上“下班不慌，慢慢来”，放到桌面',
  },
  {
    id: 's3',
    badge: '现场写壁纸',
    title: '暗黑星轨 Canvas 壁纸',
    desc: '利用前端 Canvas 生成流动的星轨与光影，一键替换为当前桌面背景。',
    prompt: '帮我写一个暗黑流动星轨的动态壁纸，带上微光粒子，做完直接应用到桌面',
  },
  {
    id: 's4',
    badge: '粒子特效',
    title: '春日微风落樱特效',
    desc: '定制全屏悬浮粒子动画包，支持调节密度、飘动方向与速度。',
    prompt: '帮我做一个春日微风带着粉色樱花瓣飘落的桌面特效，并切换到它',
  },
]

const controlSpells: Spell[] = [
  {
    id: 'c1',
    badge: '氛围联动',
    title: '冬日雪山氛围一键换装',
    desc: '智能联动壁纸库与桌面特效系统，同步切换背景与降雪粒子。',
    prompt: '换一张安静的雪山风景壁纸，并开启落雪特效',
  },
  {
    id: 'c2',
    badge: '外观调整',
    title: '换肤与表情互动',
    desc: '更换桌面小伴的外观外壳，设置开心、眨眼或专注表情。',
    prompt: '把小伴换成毛绒小猫皮肤，做一个开心的表情',
  },
  {
    id: 'c3',
    badge: '启动游戏',
    title: '离线小游戏快捷唤起',
    desc: '无需翻找菜单，直接唤出水墨剑客、俄罗斯方块、贪吃蛇或 2048。',
    prompt: '打开水墨剑客小游戏，我想放松一下',
  },
]

const careSpells: Spell[] = [
  {
    id: 'l1',
    badge: '健康作息',
    title: '定时饮水提醒',
    desc: '自动生成关怀规则，到点以温和气泡通知，免打扰时段自动避让。',
    prompt: '每天下午三点半提醒我喝水吃点水果',
  },
  {
    id: 'l2',
    badge: '久坐关怀',
    title: '工作间隔休息',
    desc: '每隔指定时长提醒起身活动，呵护眼睛与颈椎。',
    prompt: '工作每隔 45 分钟提醒我闭眼休息，活动活动颈椎',
  },
  {
    id: 'l3',
    badge: '下班提醒',
    title: '下班倒计时与总结',
    desc: '提醒收尾工作、准时下班，给予下班仪式感。',
    prompt: '每天下午六点半提醒我：准备下班，今天辛苦啦',
  },
]

const helperSpells: Spell[] = [
  {
    id: 'h1',
    badge: '工作区分析',
    title: '整理目录与文档清单',
    desc: '扫描指定工作目录，提取文件大纲与笔记关键摘要。',
    prompt: '帮我看看当前工作目录里有什么文件，整理一份清晰的清单',
  },
  {
    id: 'h2',
    badge: '截图多模态',
    title: '粘贴截图智能解答',
    desc: '聊天框直接 Ctrl+V / Cmd+V 粘贴错误截图或设计图，多图并发解析。',
    prompt: '看下我刚粘贴的截图，分析一下报错的原因和解决办法',
  },
  {
    id: 'h3',
    badge: '语音朗读',
    title: 'Edge-TTS 拟真音色',
    desc: '采用高质量微软 Edge-TTS 神经网络语音，点击即可朗读小伴回答。',
    prompt: '念一段温柔的晚安寄语给我听吧',
  },
]

async function copyText(text: string, id: string) {
  try {
    await navigator.clipboard.writeText(text)
    copied.value = id
    setTimeout(() => {
      if (copied.value === id) copied.value = null
    }, 2000)
  } catch {
    /* ignore */
  }
}
</script>

<template>
  <main class="guide-page">
    <section class="guide-hero">
      <div class="inner">
        <span class="eyebrow"><Icon name="sparkles" :size="16" /> 官方玩法手册</span>
        <h1>小伴桌面 · 玩法与功能指南</h1>
        <p class="subtitle">
          小伴桌面不仅能提供安静流动的动态壁纸、天气特效与桌面组件；更内置了一个深度整合桌面系统与创作引擎的 AI Agent 伴侣。
          在这里，你可以通过一句话直接造挂件、换壁纸、设作息，让桌面真正灵动起来。
        </p>
      </div>
    </section>

    <div class="guide-content">
      <!-- 模块 1：AI 现场创造 -->
      <section class="guide-section">
        <div class="section-title">
          <span class="num">01</span>
          <div>
            <h2>AI 现场创造 · 挂件与动态壁纸</h2>
            <p>这是小伴最与众不同的能力：它能现场编写前端代码，直接挂载到你的桌面上。</p>
          </div>
        </div>
        <div class="cards-grid">
          <article v-for="s in creationSpells" :key="s.id" class="spell-card">
            <div class="spell-head">
              <span class="badge red">{{ s.badge }}</span>
              <h3>{{ s.title }}</h3>
            </div>
            <p class="spell-desc">{{ s.desc }}</p>
            <div class="spell-code">
              <code>{{ s.prompt }}</code>
              <button type="button" class="copy-btn" :aria-label="`复制咒语：${s.title}`" @click="copyText(s.prompt, s.id)">
                <Icon v-if="copied === s.id" name="check" :size="14" />
                <span v-else>复制</span>
              </button>
            </div>
          </article>
        </div>
      </section>

      <!-- 模块 2：桌面控制与联动 -->
      <section class="guide-section">
        <div class="section-title">
          <span class="num">02</span>
          <div>
            <h2>桌面控制 · 氛围一键联动</h2>
            <p>动口不动手，将壁纸、特效、小伴表情与离线小游戏融为一体。</p>
          </div>
        </div>
        <div class="cards-grid">
          <article v-for="s in controlSpells" :key="s.id" class="spell-card">
            <div class="spell-head">
              <span class="badge blue">{{ s.badge }}</span>
              <h3>{{ s.title }}</h3>
            </div>
            <p class="spell-desc">{{ s.desc }}</p>
            <div class="spell-code">
              <code>{{ s.prompt }}</code>
              <button type="button" class="copy-btn" :aria-label="`复制咒语：${s.title}`" @click="copyText(s.prompt, s.id)">
                <Icon v-if="copied === s.id" name="check" :size="14" />
                <span v-else>复制</span>
              </button>
            </div>
          </article>
        </div>
      </section>

      <!-- 模块 3：生活关怀与日常作息 -->
      <section class="guide-section">
        <div class="section-title">
          <span class="num">03</span>
          <div>
            <h2>日常关怀 · 温和作息管家</h2>
            <p>基于系统级关怀引擎，准时用微风般的气泡提醒你生活中的小事。</p>
          </div>
        </div>
        <div class="cards-grid">
          <article v-for="s in careSpells" :key="s.id" class="spell-card">
            <div class="spell-head">
              <span class="badge green">{{ s.badge }}</span>
              <h3>{{ s.title }}</h3>
            </div>
            <p class="spell-desc">{{ s.desc }}</p>
            <div class="spell-code">
              <code>{{ s.prompt }}</code>
              <button type="button" class="copy-btn" :aria-label="`复制咒语：${s.title}`" @click="copyText(s.prompt, s.id)">
                <Icon v-if="copied === s.id" name="check" :size="14" />
                <span v-else>复制</span>
              </button>
            </div>
          </article>
        </div>
      </section>

      <!-- 模块 4：本地文件与多模态 -->
      <section class="guide-section">
        <div class="section-title">
          <span class="num">04</span>
          <div>
            <h2>本地助手 · 工作区分析与识图</h2>
            <p>连接你选定的工作区，支持截图直接粘贴多模态分析，语音实时朗读。</p>
          </div>
        </div>
        <div class="cards-grid">
          <article v-for="s in helperSpells" :key="s.id" class="spell-card">
            <div class="spell-head">
              <span class="badge purple">{{ s.badge }}</span>
              <h3>{{ s.title }}</h3>
            </div>
            <p class="spell-desc">{{ s.desc }}</p>
            <div class="spell-code">
              <code>{{ s.prompt }}</code>
              <button type="button" class="copy-btn" :aria-label="`复制咒语：${s.title}`" @click="copyText(s.prompt, s.id)">
                <Icon v-if="copied === s.id" name="check" :size="14" />
                <span v-else>复制</span>
              </button>
            </div>
          </article>
        </div>
      </section>

      <!-- 模块 5：模型配置与安全机制 -->
      <section class="guide-section config-section">
        <div class="section-title">
          <span class="num">05</span>
          <div>
            <h2>模型接入与安全机制</h2>
            <p>小伴把数据安全与选择权完全交给你。</p>
          </div>
        </div>
        <div class="info-blocks">
          <div class="info-box">
            <h3>🤖 推荐大模型配置</h3>
            <ul>
              <li><strong>DeepSeek (高性价比首选)</strong>：推荐填写 <code>deepseek-chat</code>（日常对话）或 <code>deepseek-reasoner</code>（强逻辑推理），响应快且极度实惠。</li>
              <li><strong>智谱 GLM / 月之暗面 Kimi / MiniMax</strong>：大厂官方服务商一键接入，多模态与长文档表现出色。</li>
              <li><strong>本地 Ollama (100% 离线隐私)</strong>：在“设置 → AI 模型”添加自定义端点，地址填 <code>http://localhost:11434/v1</code>，模型填 <code>qwen2.5:7b</code> 等，无需联网即可使用。</li>
            </ul>
          </div>
          <div class="info-box">
            <h3>🛡️ 透明安全的权限把控</h3>
            <p>
              小伴默认采用<strong>按需授权机制</strong>：在读取本地文件时自动完成；但在<strong>写入文件、运行命令、修改系统配置</strong>前，都会先在聊天界面弹出一张卡片询问你。
              你可以选择“允许一次”、“这段对话都允许”或“拒绝”，高风险操作永远二次确认，杜绝任何意外操作。
            </p>
          </div>
        </div>
      </section>

      <div class="guide-cta">
        <h3>现在就去桌面体验吧</h3>
        <p>打开小伴桌面的「聊天」界面，点击右上角【玩法指南】或者直接发送一句试试看。</p>
        <RouterLink to="/" class="cta-btn">返回小伴桌面首页</RouterLink>
      </div>
    </div>
  </main>
</template>

<style scoped>
.guide-page {
  padding-bottom: 80px;
}

.guide-hero {
  padding: 64px var(--gutter) 40px;
  background: radial-gradient(ellipse at 50% 0%, rgba(200, 160, 100, 0.12), transparent 70%);
  border-bottom: 1px solid var(--line);
}

.guide-hero .inner {
  max-width: 960px;
  margin: 0 auto;
}

.eyebrow {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  letter-spacing: 1px;
  color: #d4a373;
  font-weight: 600;
  margin-bottom: 12px;
}

.guide-hero h1 {
  font-family: var(--font-display);
  font-size: 42px;
  font-weight: 700;
  line-height: 1.25;
  color: var(--text);
  margin-bottom: 16px;
}

.subtitle {
  font-size: 16px;
  line-height: 1.8;
  color: var(--text-2);
}

.guide-content {
  max-width: 1080px;
  margin: 0 auto;
  padding: 48px var(--gutter) 0;
  display: grid;
  gap: 56px;
}

.guide-section {
  display: grid;
  gap: 24px;
}

.section-title {
  display: flex;
  align-items: baseline;
  gap: 16px;
}

.section-title .num {
  font-family: var(--font-display);
  font-size: 32px;
  font-weight: 700;
  color: #d4a373;
  opacity: 0.8;
}

.section-title h2 {
  font-size: 22px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 4px;
}

.section-title p {
  font-size: 14px;
  color: var(--text-3);
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.spell-card {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--r-tile);
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  transition: transform 0.2s, border-color 0.2s;
}

.spell-card:hover {
  transform: translateY(-2px);
  border-color: var(--line-strong);
}

.spell-head {
  display: flex;
  align-items: center;
  gap: 10px;
}

.badge {
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 6px;
  font-weight: 600;
}

.badge.red {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

.badge.blue {
  background: rgba(14, 165, 233, 0.15);
  color: #38bdf8;
}

.badge.green {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
}

.badge.purple {
  background: rgba(168, 85, 247, 0.15);
  color: #c084fc;
}

.spell-head h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.spell-desc {
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-2);
  margin: 0;
}

.spell-code {
  background: var(--surface-2);
  border-radius: var(--r-ui);
  padding: 10px 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-top: auto;
}

.spell-code code {
  font-family: inherit;
  font-size: 13px;
  color: #fef08a;
  line-height: 1.5;
  word-break: break-all;
}

.copy-btn {
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid var(--line);
  color: var(--text);
  border-radius: 6px;
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.16);
}

.info-blocks {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.info-box {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--r-tile);
  padding: 24px;
  font-size: 14px;
  line-height: 1.8;
  color: var(--text-2);
}

.info-box h3 {
  font-size: 17px;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 12px;
}

.info-box ul {
  padding-left: 18px;
  margin: 0;
}

.info-box li {
  margin-bottom: 8px;
}

.info-box code {
  background: var(--surface-2);
  padding: 2px 6px;
  border-radius: 4px;
  color: #38bdf8;
  font-size: 13px;
}

.guide-cta {
  text-align: center;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--r-tile);
  padding: 48px 24px;
  margin-top: 20px;
}

.guide-cta h3 {
  font-size: 24px;
  color: var(--text);
  margin-bottom: 8px;
}

.guide-cta p {
  color: var(--text-2);
  font-size: 14px;
  margin-bottom: 24px;
}

.cta-btn {
  display: inline-block;
  background: #d4a373;
  color: #1a1a1e;
  font-weight: 600;
  padding: 10px 24px;
  border-radius: 999px;
  text-decoration: none;
  transition: opacity 0.2s;
}

.cta-btn:hover {
  opacity: 0.9;
}

@media (max-width: 768px) {
  .cards-grid,
  .info-blocks {
    grid-template-columns: 1fr;
  }
  .guide-hero h1 {
    font-size: 32px;
  }
}
</style>
