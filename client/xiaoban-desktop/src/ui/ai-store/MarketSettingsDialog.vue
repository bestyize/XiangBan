<script setup lang="ts">
// 插件来源：市场地址随应用内置，不能修改；这里只管 SkillHub 是否作为补充来源、是否检查更新。
import { ref, watch } from 'vue';
import { NModal } from 'naive-ui';
import { Check, RefreshCw } from 'lucide-vue-next';
import { invoke } from '../../bridge.ts';
import './store-cards.css';

const props = defineProps<{ show: boolean }>();
const skillhub = defineModel<boolean>('skillhub', { required: true });
const autoUpdates = defineModel<boolean>('autoUpdates', { required: true });
const emit = defineEmits<{ close: [] }>();
const origin = ref(''), status = ref<'idle' | 'checking' | 'ok' | 'failed'>('idle'), detail = ref('');
let token = 0;

async function check() {
  const id = ++token;
  status.value = 'checking'; detail.value = '';
  try {
    const info = await invoke<{ origin: string }>('ai_market_info');
    origin.value = info.origin;
    const page = await invoke<{ total: number | null }>('ai_market_browse', { kind: 'skills', keyword: '', page: 1, category: '', source: '', sortBy: '', variant: '' });
    if (id !== token) return;
    status.value = 'ok'; detail.value = page.total === null ? '' : `技能 ${page.total.toLocaleString('zh-CN')} 项`;
  } catch (e) { if (id === token) { status.value = 'failed'; detail.value = String(e); } }
}
watch(() => props.show, shown => { if (shown) void check(); else ++token; }, { immediate: true });
</script>

<template>
  <NModal :show="show" preset="card" title="插件来源" class="store-dialog" style="width:640px;max-width:calc(100vw - 36px)" @update:show="!$event && emit('close')">
    <p class="sub">AI 插件页从哪里获取专家、技能和连接器。</p>
    <div class="source-row">
      <div class="text"><b>小伴市场</b><span>默认来源，内容经过整理和审核，可按分类与关键词浏览。</span>
        <code v-if="origin" class="origin">{{ origin }}</code>
        <span v-if="status === 'checking'" class="state"><RefreshCw :size="12" />正在检查连接…</span>
        <span v-else-if="status === 'ok'" class="state ok"><Check :size="12" :stroke-width="3" />已连接{{ detail ? ' · ' + detail : '' }}</span>
        <span v-else-if="status === 'failed'" class="state bad">无法连接：{{ detail }} <button class="text-action" @click="check">重试</button></span>
      </div>
    </div>
    <label class="source-row"><div class="text"><b>SkillHub 直连</b><span>作为补充来源。小伴市场不可用或没有想要的内容时，可在列表右上角切换。</span></div><input v-model="skillhub" type="checkbox" role="switch" aria-label="启用 SkillHub 直连" /></label>
    <label class="source-row"><div class="text"><b>自动检查更新</b><span>打开 AI 插件页时检查已安装的插件是否有新版本。只提示，不会自动更新。</span></div><input v-model="autoUpdates" type="checkbox" role="switch" aria-label="自动检查更新" /></label>
    <div class="source-row"><div class="text"><b>网页一键安装</b><span>「小伴市场」网页上的「在小伴中安装」会唤起本应用，并始终先弹出确认，不会静默安装。</span></div></div>
    <div class="dialog-actions"><button class="store-primary" @click="emit('close')">完成</button></div>
  </NModal>
</template>

<style scoped>
.sub{margin:0 0 6px}
.source-row{display:flex;align-items:center;gap:16px;padding:14px 0;border-bottom:1px solid var(--ui-line)}
.source-row:last-of-type{border-bottom:0}
.text{flex:1;display:grid;gap:3px}
.text b{font-size:13.5px;font-weight:500;color:var(--ui-text-primary)}
.text span{font-size:12px;line-height:1.6;color:var(--ui-text-secondary)}
.origin{font-size:12px;color:var(--ui-text-primary);overflow-wrap:anywhere}
.state{display:inline-flex;align-items:center;gap:5px}
.state.ok{color:var(--ui-success-text)}
.state.bad{color:var(--ui-danger)}
input[type=checkbox]{appearance:none;flex:none;width:36px;height:22px;margin:0;border-radius:999px;background:var(--ui-control);position:relative;cursor:pointer;transition:background .15s}
input[type=checkbox]::after{content:'';position:absolute;top:2px;left:2px;width:18px;height:18px;border-radius:50%;background:#fff;box-shadow:0 1px 2px rgba(0,0,0,.25);transition:transform .15s}
input[type=checkbox]:checked{background:var(--ui-btn)}
input[type=checkbox]:checked::after{transform:translateX(14px)}
input[type=checkbox]:focus-visible{outline:2px solid var(--ui-btn);outline-offset:2px}
</style>
