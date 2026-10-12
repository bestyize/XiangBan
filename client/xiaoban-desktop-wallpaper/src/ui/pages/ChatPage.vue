<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { NButton, NInput } from 'naive-ui';
import { ArrowDown, Check, FolderOpen, LoaderCircle, MessageSquarePlus, SendHorizontal, Shield, ShieldAlert, Square, Trash2, X, Zap } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import type { ControlStore } from '../control-types.ts';
import MarkdownView from '../chat/MarkdownView.ts';
import { CONTINUE_TEXT } from '../chat/chat-store.ts';
import { approvalPreview, clip, formatAgo, isImeEnter, isNearBottom, prettyArguments, riskVerb, toolLabel } from '../chat/format.ts';
import { describeCall } from '../chat/view.ts';
import type { ApprovalDecision, ApprovalRequest, QuestionRequest } from '../chat/types.ts';

const props = defineProps<{ store: ControlStore }>();
const chat = props.store.chat;

const draft = ref('');
const composing = ref(false);
const scroller = ref<HTMLElement | null>(null);
const stick = ref(true);
const confirmingId = ref('');
const answerText = ref('');
const examples = ['帮我看看工作目录里有什么，整理一份清单', '把壁纸换成一张安静的风景', '每天下午三点提醒我喝水', '查一下今天是几号，再告诉我这周还剩几天'];

const view = computed(() => chat.current());
const running = computed(() => !!view.value?.running);
const title = computed(() => (chat.currentId && chat.sessions.find(s => s.id === chat.currentId)?.title) || '新对话');
const canSend = computed(() => chat.configured && !running.value && draft.value.trim() !== '');
const workspaceName = computed(() => chat.workspace.split(/[\\/]/).filter(Boolean).slice(-2).join('/') || chat.workspace);
const now = () => Math.floor(Date.now() / 1000);
const waiting = computed(() => !!view.value && (view.value.approvals.length > 0 || view.value.questions.length > 0));
const thinking = computed(() => {
  const v = view.value;
  if (!v?.running || waiting.value) return false;
  const last = v.items[v.items.length - 1];
  return !(last?.kind === 'assistant' && last.streaming);
});
const statusLine = computed(() => view.value?.status || (waiting.value ? '等你确认后继续' : thinking.value ? '思考中…' : ''));

onMounted(async () => {
  const first = !chat.ready;
  await chat.init();
  if (!first) await chat.refresh();
});

function toBottom() {
  const el = scroller.value;
  if (el) el.scrollTop = el.scrollHeight;
}
function onScroll() {
  if (scroller.value) stick.value = isNearBottom(scroller.value);
}
const signature = computed(() => {
  const v = view.value;
  if (!v) return '';
  const last = v.items[v.items.length - 1];
  return [v.id, v.items.length, last && 'text' in last ? last.text.length : 0, last && 'output' in last ? last.output.length : 0, v.approvals.length, v.questions.length].join('|');
});
watch(signature, async (value, before) => {
  if (value.split('|')[0] !== (before ?? '').split('|')[0]) stick.value = true;
  await nextTick();
  if (stick.value) toBottom();
});

async function submit() {
  const text = draft.value.trim();
  if (!text || !canSend.value) return;
  draft.value = '';
  stick.value = true;
  if (!(await chat.send(text)) && !draft.value) draft.value = text;
  await nextTick();
  toBottom();
}
function onKeydown(event: KeyboardEvent) {
  if (event.key !== 'Enter' || event.shiftKey || isImeEnter(event, composing.value)) return;
  event.preventDefault();
  void submit();
}
const useExample = (text: string) => { draft.value = text; };
const continueAnswer = () => { stick.value = true; void chat.send(CONTINUE_TEXT); };

const choose = (id: string | null) => { confirmingId.value = ''; void chat.select(id); };
async function remove(id: string) { confirmingId.value = ''; await chat.remove(id); }
function openSettings() { props.store.settingsTab = 'ai'; props.store.navigate('settings'); }

const decide = (request: ApprovalRequest, decision: ApprovalDecision) => chat.answerApproval(request, decision);
function answer(request: QuestionRequest, value: string | null) {
  answerText.value = '';
  return chat.answerQuestion(request, value);
}

async function openLink(href: string) {
  const url = href.split('#')[0];
  try {
    if (!props.store.desktop) { window.open(url, '_blank', 'noopener,noreferrer'); return; }
    await invoke('open_download_url', { url });
  } catch {
    try { await navigator.clipboard.writeText(href); props.store.notify('这个地址不能直接打开，已复制到剪贴板', 'info'); } catch { props.store.notify('这个地址不能直接打开', 'error'); }
  }
}
</script>

<template>
  <section class="chat-page" data-testid="chat-page">
    <aside class="chat-side" aria-label="对话列表">
      <div class="side-head">
        <NButton type="primary" block data-testid="chat-new" :disabled="!chat.currentId" @click="choose(null)"><template #icon><MessageSquarePlus :size="16" /></template>新对话</NButton>
      </div>
      <ul class="session-list">
        <li v-for="s in chat.sessions" :key="s.id" :class="{ current: s.id === chat.currentId }">
          <template v-if="confirmingId === s.id">
            <div class="confirm-row" data-testid="chat-delete-confirm"><span>删除这段对话？</span><button type="button" class="link danger" :data-testid="`chat-delete-yes-${s.id}`" @click="remove(s.id)">删除</button><button type="button" class="link" @click="confirmingId = ''">取消</button></div>
          </template>
          <template v-else>
            <button type="button" class="session" :data-testid="`chat-session-${s.id}`" :aria-current="s.id === chat.currentId ? 'true' : undefined" @click="choose(s.id)">
              <span class="session-title">{{ s.title }}</span>
              <span class="session-meta">
                <LoaderCircle v-if="chat.views[s.id]?.running" class="spin" :size="12" aria-label="正在回答" />
                <i v-if="chat.views[s.id] && (chat.views[s.id].approvals.length || chat.views[s.id].questions.length)" class="attention-dot" :data-testid="`chat-attention-${s.id}`" aria-label="需要你确认"></i>
                {{ formatAgo(s.updatedAt, now()) }}
              </span>
            </button>
            <button type="button" class="session-delete" :data-testid="`chat-delete-${s.id}`" :aria-label="`删除 ${s.title}`" :disabled="chat.views[s.id]?.running" @click="confirmingId = s.id"><Trash2 :size="14" /></button>
          </template>
        </li>
        <li v-if="chat.ready && !chat.sessions.length" class="empty-list">还没有对话</li>
      </ul>
    </aside>

    <div class="chat-main">
      <header class="chat-head">
        <h2 data-testid="chat-title">{{ title }}</h2>
        <div class="head-actions">
          <div class="permission-toggle" role="group" aria-label="权限模式" data-testid="chat-permission-toggle">
            <button
              type="button"
              class="perm-btn"
              :class="{ active: chat.permissionMode === 'ask' }"
              data-testid="perm-ask"
              title="默认权限：读操作自动执行，写文件、运行命令与修改设置前需确认"
              @click="chat.setPermissionMode('ask')"
            >
              <Shield :size="12" />
              <span>默认权限</span>
            </button>
            <button
              type="button"
              class="perm-btn"
              :class="{ active: chat.permissionMode === 'auto' }"
              data-testid="perm-auto"
              title="完全信任：自动执行所有操作（文件修改、命令等无需逐一确认）"
              @click="chat.setPermissionMode('auto')"
            >
              <Zap :size="12" />
              <span>完全信任</span>
            </button>
          </div>
          <button v-if="chat.workspace" type="button" class="workspace-chip" data-testid="chat-workspace" :title="`工作目录：${chat.workspace}\n点击到设置里更改`" @click="openSettings"><FolderOpen :size="14" />{{ workspaceName }}</button>
        </div>
      </header>

      <div ref="scroller" class="chat-scroll" data-testid="chat-messages" @scroll.passive="onScroll">
        <div class="chat-column">
          <p v-if="!chat.ready" class="muted state-line">正在读取对话…</p>
          <div v-else-if="!store.desktop" class="info-card" data-testid="chat-preview-only"><b>请在桌面应用中使用聊天</b><p>浏览器预览里只能看到界面。</p></div>
          <div v-else-if="!chat.configured && !view?.items.length" class="info-card" data-testid="chat-unconfigured">
            <b>先给小伴接上一个模型</b>
            <p>在“设置 → AI 模型”里填写接口地址、模型名，需要的话再填密钥。填好后回到这里就能聊天。</p>
            <NButton type="primary" data-testid="chat-go-settings" @click="openSettings">去设置</NButton>
          </div>
          <div v-else-if="!view?.items.length" class="welcome" data-testid="chat-welcome">
            <img src="/xiaoban.svg" alt="" width="56" height="56" />
            <h3>有什么可以帮你？</h3>
            <p>我能回答问题，也能在你选的工作目录里读写文件、运行命令，还能替你调整壁纸、特效和提醒。改动之前我会先问你。</p>
            <div class="examples"><button v-for="(text, n) in examples" :key="text" type="button" :data-testid="`chat-example-${n}`" @click="useExample(text)">{{ text }}</button></div>
          </div>

          <template v-if="view">
            <template v-for="item in view.items" :key="item.id">
              <div v-if="item.kind === 'user'" class="msg user" data-testid="msg-user"><div class="bubble">{{ item.text }}<small v-if="item.images" class="muted">（含 {{ item.images }} 张图片）</small></div></div>
              <div v-else-if="item.kind === 'assistant'" class="msg assistant" data-testid="msg-assistant">
                <img class="avatar" src="/xiaoban.svg" alt="" width="28" height="28" />
                <div class="answer"><MarkdownView :source="item.text" @open="openLink" /><span v-if="item.streaming" class="cursor" aria-hidden="true"></span></div>
              </div>
              <template v-else-if="item.kind === 'tool'">
                <details v-if="!chat.hideToolCalls" class="tool" :class="item.status" :data-testid="`tool-${item.id}`">
                  <summary>
                    <LoaderCircle v-if="item.status === 'running'" class="spin" :size="14" /><Check v-else-if="item.status === 'done'" :size="14" /><X v-else :size="14" />
                    <b>{{ toolLabel(item.name) }}</b><span class="tool-sub">{{ describeCall(item.name, item.arguments) }}</span>
                  </summary>
                  <div class="tool-body">
                    <div v-if="item.arguments" class="tool-part"><small>内容</small><pre>{{ prettyArguments(item.arguments) }}</pre></div>
                    <div v-if="item.output" class="tool-part"><small>结果</small><pre>{{ clip(item.output) }}</pre></div>
                  </div>
                </details>
              </template>
              <div v-else-if="item.kind === 'notice'" class="notice" :class="item.tone" :data-testid="`notice-${item.tone}`" :role="item.tone === 'error' ? 'alert' : 'status'">
                {{ item.text }}
                <button v-if="item.canContinue && !view.running" type="button" class="link" data-testid="chat-continue" @click="continueAnswer">继续</button>
              </div>
            </template>

            <section v-for="request in view.approvals" :key="request.id" class="card approval" :class="{ exec: request.risk === 'exec' }" data-testid="chat-approval" role="alertdialog" :aria-label="`需要你确认：${toolLabel(request.tool)}`">
              <header><ShieldAlert :size="16" /><span>小伴想要{{ riskVerb(request.risk) }}：<b>{{ toolLabel(request.tool) }}</b></span></header>
              <p v-if="request.tainted" class="caution" data-testid="approval-tainted">这段对话读过网页等外部内容，其中可能夹带诱导，所以这次每一步都要你确认。</p>
              <pre class="preview" data-testid="approval-preview">{{ approvalPreview(request) }}</pre>
              <footer>
                <NButton type="primary" size="small" data-testid="approval-once" @click="decide(request, 'once')">允许一次</NButton>
                <NButton v-if="request.canRemember" size="small" data-testid="approval-always" @click="decide(request, 'always')">这段对话都允许</NButton>
                <NButton size="small" quaternary data-testid="approval-deny" @click="decide(request, 'deny')">拒绝</NButton>
              </footer>
            </section>

            <section v-for="request in view.questions" :key="request.id" class="card question" data-testid="chat-question" role="alertdialog" aria-label="小伴想问你">
              <header><span>{{ request.question }}</span></header>
              <div v-if="request.options.length" class="options"><NButton v-for="(option, n) in request.options" :key="option" size="small" :data-testid="`question-option-${n}`" @click="answer(request, option)">{{ option }}</NButton></div>
              <footer>
                <NInput v-model:value="answerText" size="small" placeholder="或者自己回答" data-testid="question-input" @keydown.enter="!isImeEnter($event, composing) && answerText.trim() && answer(request, answerText)" />
                <NButton type="primary" size="small" :disabled="!answerText.trim()" data-testid="question-submit" @click="answer(request, answerText)">回答</NButton>
                <NButton size="small" quaternary data-testid="question-skip" @click="answer(request, null)">跳过</NButton>
              </footer>
            </section>

            <p v-if="statusLine" class="status-line" data-testid="chat-status"><LoaderCircle class="spin" :size="14" />{{ statusLine }}</p>
          </template>
        </div>
      </div>

      <button v-if="!stick && view?.items.length" type="button" class="to-bottom" data-testid="chat-bottom" aria-label="回到最新" @click="stick = true; toBottom()"><ArrowDown :size="16" /></button>

      <footer class="composer" @compositionstart="composing = true" @compositionend="composing = false">
        <div class="composer-box">
          <NInput v-model:value="draft" type="textarea" :autosize="{ minRows: 1, maxRows: 8 }" :disabled="!store.desktop || !chat.configured" :placeholder="chat.configured ? '问点什么，或者交给小伴去办（Enter 发送，Shift+Enter 换行）' : '先到设置里配置模型'" data-testid="chat-input" @keydown="onKeydown" />
          <NButton v-if="running" type="error" circle data-testid="chat-stop" aria-label="停止" @click="chat.cancel()"><template #icon><Square :size="14" /></template></NButton>
          <NButton v-else type="primary" circle data-testid="chat-send" aria-label="发送" :disabled="!canSend" @click="submit"><template #icon><SendHorizontal :size="16" /></template></NButton>
        </div>
      </footer>
    </div>
  </section>
</template>

<style scoped>
.chat-page{flex:1;min-height:0;display:grid;grid-template-columns:268px minmax(0,1fr);background:var(--ui-app-bg)}
.chat-side{border-right:1px solid var(--ui-border);display:flex;flex-direction:column;min-height:0;background:var(--ui-surface)}
.side-head{padding:16px 14px 10px}
.session-list{list-style:none;margin:0;padding:4px 8px 12px;overflow:auto;flex:1;scrollbar-width:thin}
.session-list li{position:relative;display:flex;align-items:center;border-radius:10px}
.session-list li:hover,.session-list li.current{background:var(--ui-surface-alt)}
.session{flex:1;min-width:0;text-align:left;background:none;border:0;color:var(--ui-text-primary);cursor:pointer;padding:9px 10px;display:grid;gap:3px;font:inherit}
.session-title{font-size:13.5px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.session-meta{font-size:12px;color:var(--ui-text-tertiary);display:flex;gap:6px;align-items:center}
.session-delete{opacity:0;border:0;background:none;color:var(--ui-text-tertiary);cursor:pointer;padding:6px;margin-right:4px;border-radius:6px}
.session-list li:hover .session-delete,.session-delete:focus-visible,.session-list li.current .session-delete{opacity:1}
.session-delete:hover:not(:disabled){color:var(--ui-primary)}
.session-delete:disabled{cursor:not-allowed;opacity:.3}
.confirm-row{flex:1;display:flex;gap:10px;align-items:center;padding:9px 10px;font-size:13px}
.empty-list{justify-content:center;padding:24px 0;color:var(--ui-text-tertiary);font-size:13px}
.attention-dot{width:7px;height:7px;border-radius:50%;background:var(--ui-primary);display:inline-block}
.link{border:0;background:none;color:var(--ui-primary);cursor:pointer;font:inherit;padding:0}
.link.danger{color:var(--ui-primary-pressed);font-weight:650}
.chat-main{display:flex;flex-direction:column;min-height:0;min-width:0;position:relative}
.chat-head{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:14px 24px;border-bottom:1px solid var(--ui-border)}
.chat-head h2{font-size:15px;margin:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.head-actions{display:flex;align-items:center;gap:10px;flex-shrink:0}
.permission-toggle{display:inline-flex;padding:2px;background:var(--ui-surface-alt);border:1px solid var(--ui-line);border-radius:999px;gap:2px}
.perm-btn{border:0;background:none;color:var(--ui-text-tertiary);border-radius:999px;padding:3px 9px;font-size:12px;cursor:pointer;display:inline-flex;gap:4px;align-items:center;transition:all .15s}
.perm-btn:hover{color:var(--ui-text-primary)}
.perm-btn.active{background:var(--ui-surface);color:var(--ui-text-primary);box-shadow:0 1px 3px rgba(0,0,0,.08);font-weight:600}
.perm-btn.active svg{color:var(--ui-primary)}
.workspace-chip{border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-secondary);border-radius:999px;padding:4px 10px;font-size:12px;display:inline-flex;gap:6px;align-items:center;cursor:pointer;max-width:260px;overflow:hidden;white-space:nowrap}
.workspace-chip:hover{color:var(--ui-text-primary)}
.chat-scroll{flex:1;min-height:0;overflow-y:auto;overflow-x:hidden;scrollbar-width:thin;scrollbar-color:var(--ui-scrollbar) transparent;overscroll-behavior:contain}
.chat-column{width:100%;max-width:820px;margin:0 auto;padding:22px 24px 12px;display:grid;gap:14px;box-sizing:border-box;min-width:0}
.state-line{text-align:center;padding:40px 0}
.info-card,.welcome{text-align:center;padding:48px 16px;display:grid;gap:10px;justify-items:center}
.info-card{background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:16px;margin-top:40px}
.info-card p,.welcome p{color:var(--ui-text-secondary);max-width:520px;line-height:1.8;margin:0;font-size:14px}
.welcome h3{margin:4px 0 0;font-size:20px}
.examples{display:grid;gap:8px;margin-top:14px;width:min(520px,100%)}
.examples button{text-align:left;border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-primary);border-radius:12px;padding:11px 14px;cursor:pointer;font:inherit;font-size:13.5px}
.examples button:hover{border-color:var(--ui-primary);background:var(--ui-primary-soft)}
.msg.user{display:flex;justify-content:flex-end;width:100%;max-width:100%;min-width:0;box-sizing:border-box}
.bubble{background:var(--ui-primary-soft);color:var(--ui-text-primary);border-radius:16px 16px 4px 16px;padding:10px 14px;max-width:min(82%,100%);white-space:pre-wrap;word-break:break-word;overflow-wrap:anywhere;line-height:1.7;font-size:14.5px;min-width:0;box-sizing:border-box}
.msg.assistant{display:grid;grid-template-columns:28px minmax(0,1fr);gap:12px;align-items:start;width:100%;max-width:100%;min-width:0;box-sizing:border-box}
.avatar{border-radius:8px;margin-top:2px}
.answer{min-width:0;font-size:14.5px;line-height:1.8;color:var(--ui-text-primary);overflow-wrap:anywhere;word-break:break-word;box-sizing:border-box}
.cursor{display:inline-block;width:7px;height:15px;background:var(--ui-primary);border-radius:2px;margin-left:2px;vertical-align:text-bottom;animation:blink 1s steps(2,start) infinite}
@keyframes blink{to{visibility:hidden}}
.tool{margin-left:40px;border:1px solid var(--ui-line);border-radius:10px;background:var(--ui-surface);font-size:13px}
.tool summary{cursor:pointer;padding:7px 12px;display:flex;gap:8px;align-items:center;color:var(--ui-text-secondary);list-style:none}
.tool summary::-webkit-details-marker{display:none}
.tool summary b{color:var(--ui-text-primary);font-weight:600;white-space:nowrap}
.tool-sub{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;min-width:0;font-family:ui-monospace,Menlo,Consolas,monospace;font-size:12px}
.tool.error summary svg{color:var(--ui-primary)}
.tool.done summary svg{color:#1fb871}
.tool-body{padding:2px 12px 10px;display:grid;gap:8px}
.tool-part small{color:var(--ui-text-tertiary)}
.tool-part pre,.preview{margin:4px 0 0;padding:9px 11px;background:var(--ui-surface-alt);border-radius:8px;overflow:auto;max-height:260px;font-size:12px;line-height:1.6;white-space:pre-wrap;overflow-wrap:anywhere;font-family:ui-monospace,Menlo,Consolas,monospace}
.notice{margin-left:40px;font-size:13px;padding:8px 12px;border-radius:10px;background:var(--ui-surface-alt);color:var(--ui-text-secondary);display:flex;gap:12px;align-items:center}
.notice.error{background:var(--ui-primary-soft);color:var(--ui-primary-pressed)}
.notice.warning{background:rgba(245,158,11,.14)}
.card{margin-left:40px;border:1px solid var(--ui-border);border-radius:14px;background:var(--ui-surface);padding:14px 16px;display:grid;gap:10px;box-shadow:var(--ui-shadow-pop)}
.card.exec{border-color:var(--ui-primary)}
.card header{display:flex;gap:8px;align-items:center;font-size:14px}
.card header svg{color:var(--ui-primary);flex:none}
.card footer{display:flex;gap:8px;align-items:center;flex-wrap:wrap}
.caution{margin:0;font-size:12.5px;color:var(--ui-text-secondary);background:rgba(245,158,11,.14);border-radius:8px;padding:7px 10px;line-height:1.6}
.options{display:flex;gap:8px;flex-wrap:wrap}
.status-line{margin:0 0 0 40px;font-size:13px;color:var(--ui-text-tertiary);display:flex;gap:8px;align-items:center}
.spin{animation:spin 1s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
.to-bottom{position:absolute;right:28px;bottom:96px;width:34px;height:34px;border-radius:50%;border:1px solid var(--ui-border);background:var(--ui-surface);color:var(--ui-text-primary);cursor:pointer;display:grid;place-items:center;box-shadow:var(--ui-shadow-pop)}
.composer{padding:10px 24px 18px}
.composer-box{max-width:820px;margin:0 auto;display:flex;gap:10px;align-items:flex-end;background:var(--ui-surface);border:1px solid var(--ui-border);border-radius:18px;padding:8px 10px 8px 14px}
.composer-box :deep(.n-input){--n-border:none;--n-border-hover:none;--n-border-focus:none;--n-box-shadow-focus:none;background:transparent}
.composer-box :deep(.n-input .n-input__border),.composer-box :deep(.n-input .n-input__state-border){display:none}
.composer-box :deep(.n-input-wrapper){padding-left:0}
.chat-page :deep(.markdown>*:first-child){margin-top:0}
.chat-page :deep(.markdown>*:last-child){margin-bottom:0}
.chat-page :deep(.markdown p){margin:0 0 .75em}
.chat-page :deep(.markdown .md-heading){margin:1.1em 0 .5em;line-height:1.4}
.chat-page :deep(.markdown h3){font-size:1.25em}.chat-page :deep(.markdown h4){font-size:1.12em}.chat-page :deep(.markdown h5),.chat-page :deep(.markdown h6){font-size:1em}
.chat-page :deep(.markdown ul),.chat-page :deep(.markdown ol){margin:0 0 .75em;padding-left:1.5em}
.chat-page :deep(.markdown li>p){margin:.3em 0}
.chat-page :deep(.markdown blockquote){margin:0 0 .75em;padding:2px 14px;border-left:3px solid var(--ui-border);color:var(--ui-text-secondary)}
.chat-page :deep(.markdown hr){border:0;border-top:1px solid var(--ui-line);margin:1em 0}
.chat-page :deep(.markdown .md-code){background:var(--ui-surface-alt);border-radius:5px;padding:1px 6px;font-size:.88em;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.chat-page :deep(.markdown .md-link){color:var(--ui-primary);cursor:pointer;text-decoration:underline;text-underline-offset:2px;overflow-wrap:anywhere}
.chat-page :deep(.markdown .md-table){overflow:auto;margin:0 0 .75em}
.chat-page :deep(.markdown table){border-collapse:collapse;font-size:13.5px}
.chat-page :deep(.markdown th),.chat-page :deep(.markdown td){border:1px solid var(--ui-line);padding:6px 12px;text-align:left}
.chat-page :deep(.markdown th){background:var(--ui-surface-alt);font-weight:600}
@media(max-width:900px){
  .chat-page{grid-template-columns:190px minmax(0,1fr)}
  .chat-column{padding:16px 14px 12px}
  .bubble{max-width:88%}
  .tool,.notice,.card,.status-line{margin-left:0}
}
@media(max-width:680px){
  .chat-page{grid-template-columns:1fr}
  .chat-side{display:none}
  .bubble{max-width:92%}
}
@media(prefers-reduced-motion:reduce){.spin,.cursor{animation:none}}
</style>
