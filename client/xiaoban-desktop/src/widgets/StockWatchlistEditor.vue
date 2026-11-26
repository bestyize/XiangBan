<script setup lang="ts">
import {computed,ref} from 'vue';
import {validStockSymbol,type LivingData,type WidgetContent} from './model.ts';
const props=defineProps<{living?:LivingData;save:(c:WidgetContent)=>Promise<void>}>();
const base=computed(()=>props.living??{symbol:'116.00700',count:0,reminders:[]}),list=computed(()=>base.value.symbols?.length?base.value.symbols:[base.value.symbol]);
const symbol=ref(''),busy=ref(false),error=ref('');
const directory=[
  {symbol:'116.00700',name:'腾讯控股',keys:'txkg tencent 00700'},
  {symbol:'1.000001',name:'上证指数',keys:'szzs shanghai 000001'},
  {symbol:'0.399001',name:'深证成指',keys:'szcz shenzhen 399001'},
  {symbol:'0.399006',name:'创业板指',keys:'cybz 399006'},
  {symbol:'1.600519',name:'贵州茅台',keys:'gzmt maotai 600519'},
  {symbol:'116.09988',name:'阿里巴巴',keys:'albb baba 09988'},
  {symbol:'116.03690',name:'美团',keys:'mt meituan 03690'},
  {symbol:'116.01810',name:'小米集团',keys:'xm xiaomi 01810'},
  {symbol:'0.300750',name:'宁德时代',keys:'ndsd catl 300750'},
  {symbol:'0.002594',name:'比亚迪',keys:'byd 002594'},
];
const presets=directory.slice(0,4);
const suggestions=computed(()=>{
  const q=symbol.value.trim().toLowerCase();
  const pool=q?directory.filter(p=>!list.value.includes(p.symbol)&&(p.name.toLowerCase().includes(q)||p.symbol.toLowerCase().includes(q)||p.keys.includes(q))):presets.filter(p=>!list.value.includes(p.symbol));
  return pool.slice(0,5);
});
const name=(s:string)=>directory.find(p=>p.symbol===s)?.name||s;
async function update(symbols:string[],selected=base.value.symbol){busy.value=true;error.value='';try{await props.save({living:{...base.value,symbols,symbol:selected}});return true;}catch(e){error.value=String(e);return false;}finally{busy.value=false;}}
async function add(value=symbol.value.trim()){
  const matched=directory.find(p=>p.name===value);
  const code=matched?matched.symbol:value;
  if(!validStockSymbol(code)){error.value='请输入有效代码，如 116.00700、1.600519、0.000001';return;}
  if(list.value.includes(code)){error.value='这只证券已在自选中';return;}
  if(list.value.length>=8){error.value='最多关注 8 只证券';return;}
  if(await update([...list.value,code]))symbol.value='';
}
function remove(s:string){const rest=list.value.filter(x=>x!==s);if(rest.length)void update(rest,s===base.value.symbol?rest[0]:base.value.symbol);}
</script>
<template>
<section class="watchlist-editor">
 <header>自选列表 <small>{{ list.length }} / 8</small></header>
 <label v-for="s in list" :key="s" class="watchlist-row">
  <input type="radio" name="selected-stock" :checked="base.symbol===s" :disabled="busy" :aria-label="`关注${name(s)}`" @change="update(list,s)"/>
  <span>{{ name(s) }}<small>{{ s }}</small></span>
  <button type="button" class="remove-btn" :disabled="busy||list.length<=1" :aria-label="`移除${name(s)}`" @click="remove(s)">移除</button>
 </label>
 <form @submit.prevent="add()">
  <input v-model="symbol" placeholder="搜名称/拼音或输代码，如 茅台" aria-label="自选证券代码" :disabled="busy"/>
  <button type="submit" class="add-btn" :disabled="busy||list.length>=8">添加</button>
 </form>
 <div v-if="suggestions.length" class="watchlist-presets">
  <button v-for="p in suggestions" :key="p.symbol" type="button" :disabled="busy||list.length>=8" @click="add(p.symbol)">+ {{ p.name }}</button>
 </div>
 <p>沪市 1.600519 · 深市 0.000001 · 港股 116.00700。点击组件上的“自选”可切换行情。</p>
 <p v-if="error" role="alert">{{ error }}</p>
</section>
</template>
<style scoped>
.watchlist-editor{display:grid;gap:10px;font-size:13px}
.watchlist-editor header{display:flex;justify-content:space-between;font-weight:600}
.watchlist-editor small,.watchlist-editor p{font-size:11px;color:var(--ui-text-tertiary,#8e8e93)}
.watchlist-row{display:flex;gap:10px;align-items:center;padding:6px 0;border-bottom:1px solid var(--ui-line,rgba(0,0,0,.07))}
.watchlist-row>span{flex:1;display:grid;gap:2px}
.watchlist-row input{accent-color:var(--ui-btn,#a8640c)}
.remove-btn{border:0;border-radius:7px;padding:4px 8px;font:inherit;font-size:12px;background:var(--ui-control,#f2f2f4);color:var(--ui-text-secondary,#6e6e73);cursor:pointer}
.remove-btn:hover:not(:disabled){color:var(--ui-danger,#d93a44)}
.remove-btn:disabled{opacity:.4;cursor:default}
.watchlist-presets{display:flex;flex-wrap:wrap;gap:6px}
.watchlist-presets button{border:0;border-radius:7px;padding:5px 9px;font:inherit;font-size:12px;background:var(--ui-control,#f2f2f4);color:inherit;cursor:pointer}
.watchlist-presets button:hover:not(:disabled){background:var(--ui-soft,#faeedb);color:var(--ui-ink,#87500a)}
.watchlist-editor form{display:flex;gap:8px}
.watchlist-editor form input{flex:1;min-width:0;background:var(--ui-control,#f2f2f4);color:inherit;border:0;border-radius:8px;padding:7px 10px;font:inherit}
.add-btn{border:0;border-radius:8px;padding:6px 12px;font:inherit;font-weight:500;background:var(--ui-btn,#a8640c);color:#fff;cursor:pointer}
.add-btn:disabled{opacity:.4;cursor:default}
.watchlist-editor p{line-height:1.6;margin:0}
.watchlist-editor p[role=alert]{color:var(--ui-danger,#d93a44)}
</style>
