<script setup lang="ts">
import { onMounted,onUnmounted,ref } from 'vue';
import { desktop,invoke } from '../../../bridge.ts';
interface Client {id:string;configured:boolean;lastCall:number|null;lastHandshake?:number|null;configPath:string|null}
interface Status {running:boolean;enabled:boolean;url:string;error:string|null;clients:Client[];tunnelCommand?:string}
const state=ref<Status|null>(null),busy=ref(''),error=ref(''),notice=ref(''),web=ref(false),command=ref('');
const clients=[
  {id:'codex',name:'Codex / ChatGPT 桌面',hint:'写入当前 Codex 主机的 MCP 配置。ChatGPT 桌面仅在共用该主机配置的版本中可用；网页使用下方隧道入口。'},
  {id:'claude-code',name:'Claude Code',hint:'写入用户级 MCP 配置，对所有项目可用。新开会话或通过 /mcp 重连。'},
  {id:'claude-desktop',name:'Claude Desktop',hint:'配置本地 stdio MCP，兼容标准和第三方网关模式的配置目录。完成后退出并重新打开 Claude Desktop。'},
  {id:'chatgpt-web',name:'ChatGPT 网页',hint:'准备本地连接，再通过 OpenAI Secure MCP Tunnel 连接；需在账号中完成隧道创建和授权。'},
];
const find=(id:string)=>state.value?.clients.find(c=>c.id===id);
function clientStatus(id:string){
  const c=find(id);
  if(!c?.configured)return '未配置';
  if(!state.value?.running)return '已配置 · 服务已停止';
  if(c.lastCall)return `最近调用 ${new Date(c.lastCall).toLocaleTimeString()}`;
  return c.lastHandshake?'握手成功 · 尚未调用':'已配置 · 等待握手';
}
async function refresh(){if(!desktop)return;try{state.value=await invoke<Status>('design_mcp_status');}catch(e){error.value=String(e)}}
async function run(id:string,action:()=>Promise<void>){if(busy.value)return;busy.value=id;error.value='';notice.value='';try{await action()}catch(e){error.value=String(e)}finally{busy.value=''}}
async function configure(id:string,remove=false){await run(id,async()=>{
  const result=await invoke<Status>('design_mcp_configure',{client:id,remove});state.value=result;
  if(id==='chatgpt-web'){web.value=!remove;command.value=result.tunnelCommand??'';}
  notice.value=remove?'已撤销访问权限并移除小伴连接。':id==='chatgpt-web'?'本机已准备好，请完成下方网页连接步骤。':id==='claude-desktop'?'配置已保存，请退出并重新打开 Claude Desktop。':'配置已保存，请在客户端新开会话或重新连接 MCP。';
});}
async function toggle(){await run('service',async()=>{state.value=await invoke<Status>('design_mcp_set_enabled',{enabled:!state.value?.running});});}
async function copy(){await run('copy',async()=>{await navigator.clipboard.writeText(command.value);notice.value='已复制本地 MCP 启动命令。';});}
async function open(url:string){await run('link',async()=>{await invoke('open_download_url',{url});});}
let timer:ReturnType<typeof setInterval>|undefined;
onMounted(()=>{void refresh();timer=setInterval(()=>{if(!busy.value)void refresh()},5000)});onUnmounted(()=>clearInterval(timer));
</script>
<template>
<section class="design-mcp" aria-label="设计画板协作" data-testid="design-mcp-settings">
  <header><h3>设计画板协作</h3><span class="state" role="status">{{state?.running?'本机服务运行中':'本机服务未启动'}}</span><button class="xb-btn is-sm" :disabled="!desktop||!!busy" @click="toggle">{{state?.running?'停止服务':'启动服务'}}</button></header>
  <p class="scope">连接后，该客户端可读取和编辑本机资料库中的设计稿。</p>
  <div v-for="c in clients" :key="c.id" class="client" :title="c.hint" :aria-description="c.hint">
    <b>{{c.name}}</b><span class="state">{{clientStatus(c.id)}}</span>
    <button class="xb-btn is-sm" :disabled="!desktop||!!busy" :data-testid="`design-connect-${c.id}`" @click="configure(c.id)">{{busy===c.id?'处理中…':c.id==='chatgpt-web'?'连接引导':find(c.id)?.configured?'更新配置':'启动并连接'}}</button>
    <button v-if="find(c.id)?.configured" class="xb-btn is-sm" :disabled="!!busy" :aria-label="`移除 ${c.name} 连接`" @click="configure(c.id,true)">移除</button>
  </div>
  <div v-if="web" class="web-guide" data-testid="design-web-guide">
    <h4>ChatGPT 网页连接</h4>
    <ol>
      <li><button class="text-link" @click="open('https://platform.openai.com/settings/organization/tunnels')">打开 OpenAI 隧道设置</button>，创建隧道并关联你的 ChatGPT 工作区。</li>
      <li>按<button class="text-link" @click="open('https://developers.openai.com/api/docs/guides/secure-mcp-tunnels')">官方步骤</button>安装并启动 tunnel-client，将下方命令用作本地 stdio MCP。运行密钥只在本机输入。</li>
      <li><button class="text-link" @click="open('https://chatgpt.com/plugins')">打开 ChatGPT 插件</button>，添加自定义 MCP，连接方式选择 Tunnel，并选择该隧道。</li>
    </ol>
    <label>本地 MCP 启动命令<textarea :value="command" readonly aria-label="本地 MCP 启动命令" /></label><button class="xb-btn is-sm" :disabled="!!busy" @click="copy">复制命令</button>
    <p class="scope">完成账户授权并产生工具调用后，才表示连接已验证。使用期间保持小伴和隧道运行。</p>
  </div>
  <p v-if="notice" role="status">{{notice}}</p><p v-if="error||state?.error" role="alert" class="error">{{error||state?.error}}</p>
  <p v-if="!desktop" role="status">请在小伴桌面中配置协作连接。</p>
</section>
</template>
<style scoped>
.design-mcp{margin-bottom:24px;color:var(--ui-text-primary);font-size:13px}.design-mcp header{display:flex;align-items:center;gap:10px;flex-wrap:wrap}.design-mcp h3{font-size:15px;margin:0 auto 0 0}.client{display:flex;align-items:center;gap:8px;padding:10px 0;flex-wrap:wrap}.client b{font-weight:500;min-width:170px}.state{font-size:12px;color:var(--ui-text-secondary);margin-right:auto}.scope{font-size:12px;color:var(--ui-text-secondary);line-height:1.6}.web-guide{padding:12px;border-radius:10px;background:var(--ui-control)}.web-guide h4{margin:0 0 8px}.web-guide ol{padding-left:20px;line-height:1.8}.web-guide textarea{display:block;width:100%;box-sizing:border-box;min-height:76px;resize:vertical;padding:8px;margin:8px 0;background:var(--ui-surface);color:inherit;border:0;border-radius:6px;font:12px/1.6 monospace}.text-link{border:0;background:none;color:var(--ui-ink);cursor:pointer;font:inherit;padding:0 3px;text-decoration:underline}.error{color:var(--ui-danger)}
</style>
