<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
const props=defineProps<{modelValue:string;disabled?:boolean}>();
const emit=defineEmits<{ 'update:modelValue':[value:string]; valid:[value:boolean] }>();
const template=ref<Record<string,any>>({}), address=ref(''), command=ref(''), mode=ref('auto'), clientId=ref(''), scope=ref(''), token=ref(''), parseError=ref('');
const values=reactive<Record<string,string>>({});
let ownValue='', loading=false;
const remote=computed(()=>typeof template.value.url==='string'||typeof template.value.baseUrl==='string');
const placeholders=(text:string)=>[...new Set(text.match(/\$\{[^{}]+\}|YOUR_[A-Z0-9_]+|<[^<>\n]+>/g)||[])];
const isSecret=(name:string)=>/key|token|secret|password|authorization|credential/i.test(name);
const label=(name:string)=>name.replace(/^\$\{|\}$|^<|>$/g,'').replace(/^YOUR_/,'');
const fields=computed(()=>{
  const result:{id:string;label:string;secret:boolean;kind:'env'|'headers'|'variable';key:string;prefix:string;placeholder?:string}[]=[];
  for(const kind of ['headers','env'] as const){
    for(const [key,raw] of Object.entries(template.value[kind]||{})){
      if(typeof raw!=='string'||(kind==='headers'&&['oauth','none'].includes(mode.value)&&key.toLowerCase()==='authorization'))continue;
      const prefix=kind==='headers'&&/^Bearer\s+/i.test(raw)?raw.match(/^Bearer\s+/i)![0]:'';
      result.push({id:kind+'.'+key,label:placeholders(raw).length===1?label(placeholders(raw)[0]):key,secret:isSecret(key),kind,key,prefix});
    }
  }
  const text=(remote.value?address.value:'')+'\n'+JSON.stringify(template.value.args||[]);
  for(const value of placeholders(text))result.push({id:'variable.'+value,label:label(value),secret:!/(url|host|path|endpoint)/i.test(value),kind:'variable',key:value,prefix:'',placeholder:value});
  return result;
});
const manualToken=computed(()=>remote.value&&mode.value==='token'&&!fields.value.some(f=>f.secret));
function fill(text:string,url=false){for(const field of fields.value.filter(f=>f.kind==='variable')){const value=values[field.id];text=text.split(field.key).join(value?(url&&field.secret?encodeURIComponent(value):value):field.key);}return text;}
function publish(){
  if(loading||parseError.value)return;
  const config=JSON.parse(JSON.stringify(template.value));
  if(remote.value){config.url=fill(address.value.trim(),true);delete config.baseUrl;config.authMode=mode.value;config.clientId=clientId.value.trim();config.scope=scope.value.trim();}
  else config.command=command.value.trim();
  for(const kind of ['headers','env']){
    if(config[kind])config[kind]={...config[kind]};
  }
  for(const field of fields.value){if(field.kind!=='variable')config[field.kind][field.key]=field.prefix+(values[field.id]||'');}
  if(['oauth','none'].includes(mode.value)&&config.headers){for(const key of Object.keys(config.headers))if(key.toLowerCase()==='authorization')delete config.headers[key];}
  if(manualToken.value)config.headers={...config.headers,Authorization:'Bearer '+token.value.trim()};
  if(Array.isArray(config.args))config.args=config.args.map((s:unknown)=>typeof s==='string'?fill(s):s);
  // A new user edit must never import an old hidden OAuth session reference.
  delete config.oauthSession;
  ownValue=JSON.stringify(config,null,2);emit('update:modelValue',ownValue);
  const complete=fields.value.every(f=>!!values[f.id]?.trim())&&(!manualToken.value||!!token.value.trim())&&(remote.value?!!config.url:!!config.command);
  emit('valid',complete);
}
watch(()=>props.modelValue,value=>{
  if(value===ownValue)return;
  loading=true;parseError.value='';
  try{
    let parsed=JSON.parse(value);if(parsed?.mcpServers){const servers=Object.values(parsed.mcpServers);if(servers.length!==1)throw Error();parsed=servers[0];}
    if(!parsed||typeof parsed!=='object'||Array.isArray(parsed))throw Error();
    template.value=parsed;address.value=parsed.url||parsed.baseUrl||'';command.value=parsed.command||'';
    mode.value=parsed.authMode||((Object.keys(parsed.headers||{}).some(isSecret)||placeholders(address.value).length)?'token':'auto');
    clientId.value=parsed.clientId||'';scope.value=parsed.scope||'';token.value='';Object.keys(values).forEach(k=>delete values[k]);
    for(const field of fields.value){
      const raw=field.kind==='variable'?'':String(parsed[field.kind]?.[field.key]||'');
      values[field.id]=placeholders(raw).length?'':raw.slice(field.prefix.length);
    }
  }catch{parseError.value='配置 JSON 无效，请在高级配置中检查格式。';emit('valid',false);}
  loading=false;
  if(!parseError.value)publish();
},{immediate:true});
watch([address,command,mode,clientId,scope,token,values],publish,{deep:true});
function editRaw(event:Event){const text=(event.target as HTMLTextAreaElement).value;ownValue='';emit('update:modelValue',text);}
</script>
<template>
  <div class="connector-fields">
    <template v-if="!parseError">
      <label v-if="remote">服务地址<input v-model="address" aria-label="服务地址" type="url" :disabled="disabled" spellcheck="false" autocomplete="off" /></label>
      <label v-else>启动程序<input v-model="command" aria-label="启动程序" :disabled="disabled" spellcheck="false" autocomplete="off" /></label>
      <label v-if="remote">连接方式<select v-model="mode" aria-label="认证方式" :disabled="disabled"><option value="auto">自动识别（推荐）</option><option value="oauth">网页登录授权</option><option value="token">填写密钥</option><option value="none">无需授权</option></select></label>
      <p v-if="remote&&(mode==='auto'||mode==='oauth')" class="form-note">需要登录时将打开服务商的授权页面，完成后自动返回连接。</p>
      <label v-for="field in fields" :key="field.id">{{field.label}}<input v-model="values[field.id]" :aria-label="field.label" :type="field.secret?'password':'text'" :disabled="disabled" :placeholder="'请输入 '+field.label" spellcheck="false" autocomplete="off" /></label>
      <label v-if="manualToken">API Key / Token<input v-model="token" aria-label="API Key / Token" type="password" :disabled="disabled" placeholder="填写服务商提供的密钥" autocomplete="off" /></label>
      <details v-if="remote&&(mode==='oauth'||mode==='auto')"><summary>服务商要求的额外授权参数</summary><label>客户端 ID（可选）<input v-model="clientId" aria-label="OAuth 客户端 ID" :disabled="disabled" placeholder="通常自动注册；服务商要求时填写" autocomplete="off" /></label><label>授权范围（可选）<input v-model="scope" aria-label="OAuth 授权范围" :disabled="disabled" placeholder="由服务商提供；通常无需填写" autocomplete="off" /></label></details>
    </template>
    <p v-if="parseError" role="alert" class="form-error">{{parseError}}</p>
    <details class="advanced-config"><summary>高级配置 · JSON</summary><textarea :value="modelValue" aria-label="MCP 连接配置" :disabled="disabled" spellcheck="false" autocomplete="off" @input="editRaw" /></details>
  </div>
</template>
<style scoped>
.connector-fields{display:grid;gap:12px;margin:14px 0;padding:14px;background:var(--ui-surface);border-radius:12px;box-shadow:0 0 0 1px var(--ui-border)}
.connector-fields label{display:grid;gap:6px;font-size:12px;font-weight:500;color:var(--ui-text-secondary)}
input,select,textarea{box-sizing:border-box;width:100%;padding:8px 11px;border:0;border-radius:9px;background:var(--ui-control);box-shadow:inset 0 0 0 1px transparent;color:var(--ui-text-primary);font:inherit;font-size:13px;line-height:1.6;outline:none;transition:box-shadow .15s,background .15s}
input:focus,select:focus,textarea:focus{background:var(--ui-surface);box-shadow:inset 0 0 0 1.5px var(--ui-btn)}
input::placeholder,textarea::placeholder{color:var(--ui-text-tertiary)}
.form-note{margin:0;font-size:12px;color:var(--ui-text-secondary)}
details{font-size:12px}summary{cursor:pointer;color:var(--ui-text-secondary)}details label{margin-top:12px}
textarea{height:200px;font-family:var(--ui-font-mono,monospace);font-size:12px;resize:vertical;margin-top:10px;background:var(--ui-surface-sunken)}
.form-error{margin:0;font-size:12px;color:var(--ui-danger)}
</style>
