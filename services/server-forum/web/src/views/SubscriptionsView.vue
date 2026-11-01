<script setup lang="ts">
import { onMounted, onBeforeUnmount, reactive, ref, watch } from 'vue'
import { subscriptions, type Subscription, type SubscriptionInput, type Edition } from '@/api/subscriptions'
import { parseSubscriptionDraft } from '@/lib/subscriptionDraft'
import { useSession } from '@/stores/session'
import { useHead } from '@/lib/head'
useHead(()=>({title:'资讯订阅'}))
const session=useSession(), rows=ref<Subscription[]>([]), history=ref<Edition[]>([]), selected=ref<Edition|null>(null)
const busy=ref(false), error=ref(''), message=ref(''), editor=ref(false), confirmed=ref(false), configured=ref(false), enabled=ref(false), code=ref(''), verifyId=ref(''), more=ref(false)
let generation=0,disposed=false
const empty=():SubscriptionInput=>({id:crypto.randomUUID(),revision:0,title:'',interests:'',recipient:'',schedule:{kind:'daily',time:'08:00'},timezone:Intl.DateTimeFormat().resolvedOptions().timeZone||'UTC',active:true,confirmed:false})
const draft=reactive(empty()), kind=ref<'daily'|'weekly'|'once'>('daily'), time=ref('08:00'), weekday=ref(0), once=ref('')
const labels:Record<string,string>={active:'运行中',unverified:'待验证邮箱',paused:'已暂停',quota:'额度不足',completed:'已完成',queued:'等待整理',building:'正在整理',retry:'等待重试',sent:'已发送',uncertain:'发送结果待核实',cancelled:'已取消',failed:'失败',sending:'正在发送'}
const stamp=(v:string|null)=>v?new Date(v).toLocaleString():'—'
function edit(row?:Subscription){Object.assign(draft,row?{...row,active:row.status==='active',confirmed:false}:empty());kind.value=draft.schedule.kind;time.value=draft.schedule.kind==='once'?'08:00':draft.schedule.time;weekday.value=draft.schedule.kind==='weekly'?draft.schedule.weekday:0;once.value=draft.schedule.kind==='once'?draft.schedule.at:'';confirmed.value=false;editor.value=true}
function parse(){const p=parseSubscriptionDraft(draft.interests);if(p.recipient)draft.recipient=p.recipient;if(p.schedule){kind.value=p.schedule.kind;time.value=p.schedule.kind==='once'?'08:00':p.schedule.time;weekday.value=p.schedule.kind==='weekly'?p.schedule.weekday:0}confirmed.value=false;message.value=p.schedule?'已填入识别的时间，请核对时区和收件人。':'请在下面选择发送周期与时间。'}
async function load(){const mine=++generation;const [a,b]=await Promise.all([subscriptions.list(),subscriptions.history()]);if(disposed||mine!==generation)return;rows.value=a.items;configured.value=a.delivery_configured;enabled.value=a.worker_enabled;history.value=b.items;more.value=b.items.length===30}
async function act(fn:()=>Promise<void>){if(busy.value)return;busy.value=true;error.value='';message.value='';try{await fn()}catch(e){error.value=e instanceof Error?e.message:String(e)}finally{busy.value=false}}
async function save(){if(!confirmed.value)return;await act(async()=>{draft.schedule=kind.value==='once'?{kind:'once',at:once.value}:kind.value==='weekly'?{kind:'weekly',time:time.value,weekday:weekday.value}:{kind:'daily',time:time.value};const saved=await subscriptions.save({...draft,confirmed:true});editor.value=false;await load();message.value=saved.verified_at?'订阅已保存。':'订阅已保存，请验证收件邮箱后启动。'})}
async function toggle(row:Subscription){await act(async()=>{await subscriptions.save({...row,active:row.status!=='active',confirmed:true});await load()})}
async function remove(row:Subscription){if(!window.confirm(`删除「${row.title}」？停止后续发送，历史期刊仍保留。`))return;await act(async()=>{await subscriptions.remove(row);await load()})}
async function requestCode(row:Subscription){await act(async()=>{await subscriptions.verification(row.id);verifyId.value=row.id;code.value='';message.value='确认邮件已提交，请在邮箱中复制确认码。'})}
async function verify(){await act(async()=>{await subscriptions.verify(verifyId.value,code.value);verifyId.value='';code.value='';await load();message.value='邮箱已确认，订阅已启动。'})}
async function openEdition(id:string){await act(async()=>{const mine=generation;const result=await subscriptions.edition(id);if(mine===generation&&!disposed)selected.value=result})}
async function nextPage(){await act(async()=>{const mine=generation;const page=await subscriptions.history(history.value.at(-1)?.created_at);if(mine===generation){history.value.push(...page.items);more.value=page.items.length===30}})}
function download(){if(!selected.value)return;const url=URL.createObjectURL(new Blob([selected.value.plaintext||''],{type:'text/plain;charset=utf-8'}));const a=document.createElement('a');a.href=url;a.download=`xiaoban-edition-${selected.value.id}.txt`;a.click();URL.revokeObjectURL(url)}
watch(()=>session.user?.uid,()=>{++generation;rows.value=[];history.value=[];selected.value=null;editor.value=false;code.value='';verifyId.value='';if(session.loggedIn)void act(load)})
onMounted(()=>void act(load));onBeforeUnmount(()=>{disposed=true;++generation})
</script>
<template>
 <main class="subscriptions">
  <header><div><p class="eyebrow">我的资料</p><h1>资讯订阅</h1></div><button class="btn btn-primary" :disabled="busy" @click="edit()">新建订阅</button></header>
  <p class="intro">把你关注的事，整理成一封信。订阅与历史期刊保存在账号中，桌面、手机和浏览器均可管理。</p>
  <p v-if="!configured||!enabled" class="notice">服务尚未开放发送。可以保存订阅，后台启用并验证邮箱后才能投递。</p>
  <p v-if="error" role="alert" class="notice error">{{error}}</p><p v-if="message" role="status" class="notice">{{message}}</p>
  <form v-if="editor" class="panel editor" @submit.prevent="save">
   <h2>{{draft.revision?'编辑订阅':'想了解什么？'}}</h2>
   <label>关注内容<textarea v-model="draft.interests" maxlength="2000" required placeholder="例如：每周一早上8点，总结城市骑行、户外装备和附近周末活动，重点关注杭州。" @input="confirmed=false" /></label>
   <button class="btn" type="button" @click="parse">从描述填写时间与邮箱</button>
   <label>订阅名称<input v-model="draft.title" maxlength="100" required @input="confirmed=false"></label>
   <label>收件邮箱<input v-model="draft.recipient" type="email" maxlength="254" required autocomplete="email" @input="confirmed=false"></label>
   <div class="grid"><label>周期<select v-model="kind" @change="confirmed=false"><option value="daily">每天</option><option value="weekly">每周</option><option value="once">仅一次</option></select></label>
    <label v-if="kind==='weekly'">星期<select v-model="weekday" @change="confirmed=false"><option v-for="(day,i) in ['一','二','三','四','五','六','日']" :key="i" :value="i">周{{day}}</option></select></label>
    <label v-if="kind!=='once'">发送时间<input v-model="time" type="time" required @input="confirmed=false"></label>
    <label v-else>发送时刻（含时区偏移）<input v-model="once" required placeholder="2026-12-01T08:00:00+08:00" @input="confirmed=false"></label>
    <label>时区<input v-model="draft.timezone" required placeholder="Asia/Shanghai" @input="confirmed=false"></label>
   </div>
   <p class="hint">夏令时不存在的时刻跳过，重复时刻只发送一次。收件邮箱仅用于投递，不读取收件箱。使用平台服务，不上传你的 API Key。</p>
   <label class="check"><input v-model="confirmed" type="checkbox" required>我确认以上关注内容、收件邮箱、周期、时间和时区，并同意收到这些邮件。</label>
   <div class="actions"><button class="btn btn-primary" :disabled="busy||!confirmed">确认保存</button><button type="button" class="btn" @click="editor=false">取消</button></div>
  </form>
  <p v-if="!rows.length&&!busy" class="empty">还没有订阅。任何你关心的主题，都可以从这里开始。</p>
  <article v-for="row in rows" :key="row.id" class="panel card">
   <header><h2>{{row.title}}</h2><span class="status">{{labels[row.status]||row.status}}</span></header><p>{{row.interests}}</p>
   <small>{{row.recipient}} · {{row.timezone}} · 下次 {{stamp(row.next_run)}}</small>
   <div class="actions"><button class="btn btn-sm" :disabled="busy" @click="edit(row)">编辑</button><template v-if="!row.verified_at"><button class="btn btn-sm" :disabled="busy||!configured" @click="requestCode(row)">发送确认邮件</button><button class="btn btn-sm" @click="verifyId=row.id;code=''">输入确认码</button></template><button v-else-if="row.status!=='completed'" class="btn btn-sm" :disabled="busy" @click="toggle(row)">{{row.status==='active'?'暂停':'恢复'}}</button><button class="btn btn-sm" :disabled="busy" @click="remove(row)">删除</button></div>
   <form v-if="verifyId===row.id" class="actions" @submit.prevent="verify"><label>邮箱确认码<input v-model="code" required maxlength="32" autocomplete="one-time-code"></label><button class="btn btn-primary" :disabled="busy">验证并启动</button></form>
  </article>
  <header class="archive"><h2>历史期刊</h2><button class="btn" :disabled="busy" @click="act(load)">刷新</button></header>
  <p class="hint">发送失败与等待重试也会保留记录。结果待核实的邮件不会自动重发。</p>
  <button v-for="item in history" :key="item.id" class="edition" @click="openEdition(item.id)"><span>{{item.title}}<small>{{stamp(item.created_at)}}<template v-if="item.error"> · {{item.error}}</template><template v-if="item.accounting_status==='review'"> · 额度结算待核实</template></small></span><span>{{labels[item.status]||item.status}}</span></button>
  <button v-if="more" class="btn" :disabled="busy" @click="nextPage">更早期刊</button>
  <section v-if="selected" class="panel reader"><header><h2>{{selected.title}}</h2><div><button class="btn" @click="download">导出文本</button><button class="btn" @click="selected=null">关闭</button></div></header><pre>{{selected.plaintext||'本期尚未生成内容。'}}</pre><a v-for="source in selected.sources||[]" :key="source.url" :href="source.url" target="_blank" rel="noopener noreferrer">{{source.title}}</a></section>
 </main>
</template>
<style scoped>
.subscriptions{max-width:820px;margin:auto;padding:16px 0 40px}header,.actions{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap}h1{font-size:24px;margin:0}h2{font-size:17px;margin:0}.eyebrow,.intro,.hint,small{color:var(--ink-3);font-size:12px;line-height:1.7}.eyebrow{margin:0 0 4px}.intro{font-size:14px;margin:20px 0}.notice{padding:12px 16px;background:var(--surface-2);border-radius:10px;font-size:13px}.error{color:var(--danger)}.editor,.card,.reader{padding:20px;margin:16px 0;background:var(--surface);border-radius:14px}.editor{display:grid;gap:16px}.editor>button{justify-self:start}label{display:grid;gap:7px;font-size:13px}input,textarea,select{width:100%;background:var(--surface-2);color:var(--ink);border:0;border-radius:8px;padding:11px;font:inherit}textarea{min-height:110px;resize:vertical}.grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px}.check{display:flex;align-items:flex-start;line-height:1.6}.check input{width:auto;margin-top:4px}.actions{justify-content:flex-start;margin-top:12px}.status{color:var(--brand);font-size:12px}.card p{font-size:14px;white-space:pre-wrap;overflow-wrap:anywhere}.card small{display:block;overflow-wrap:anywhere}.archive{margin-top:32px}.edition{display:flex;justify-content:space-between;gap:16px;width:100%;padding:16px 0;background:none;border:0;border-bottom:1px solid var(--line);text-align:left;color:var(--ink);font:inherit;font-size:13px}.edition small{display:block}.edition>span:last-child{white-space:nowrap;color:var(--ink-3)}.reader pre{white-space:pre-wrap;overflow-wrap:anywhere;font:inherit;font-size:14px;line-height:1.9}.reader a{display:block;color:var(--brand);margin:8px 0}.empty{padding:24px;color:var(--ink-3)}@media(max-width:520px){.grid{grid-template-columns:1fr}.editor,.card,.reader{padding:16px}.edition{gap:8px}}
</style>
