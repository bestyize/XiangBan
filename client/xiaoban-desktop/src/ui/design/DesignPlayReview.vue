<script setup lang="ts">
import {computed,onBeforeUnmount,onMounted,ref,watch} from 'vue';
import {invoke,listen} from '../../bridge.ts';
import {playSteps,type Project} from './model.ts';

const props=defineProps<{project:Project;playing:boolean;writable:boolean}>();
const emit=defineEmits<{play:[];add:[step:string]}>();
const steps=computed(()=>playSteps(props.project));
const passed=ref<number[]>([]), pending=ref(false), draft=ref(''), error=ref('');
type Review={revision:number;passed:number[]};
let generation=0, off:(()=>void)|undefined, disposed=false;
async function refresh(){
  const request=++generation;
  if(!steps.value.length)return;
  const {id,revision}=props.project;
  try{
    const review=await invoke<Review>('design_play_review',{id,revision});
    if(request===generation&&review?.revision===revision){passed.value=review.passed;error.value='';}
  }catch(e){if(request===generation)error.value=String(e);}
}
watch(()=>[props.project.id,props.project.revision],()=>{
  passed.value=[];draft.value='';error.value='';void refresh();
},{immediate:true});
onMounted(async()=>{
  const stop=await listen<string>('design-play-reviewed',id=>{if(id===props.project.id)void refresh();});
  if(disposed)stop();else off=stop;
});
onBeforeUnmount(()=>{disposed=true;++generation;off?.();});
async function mark(index:number,checked:boolean){
  if(!props.playing||!props.writable||pending.value)return;
  const {id,revision}=props.project;
  pending.value=true;
  try{
    await invoke('design_play_review',{id,revision,step:index,passed:checked});
    if(id===props.project.id&&revision===props.project.revision)await refresh();
  }catch(e){if(id===props.project.id&&revision===props.project.revision)error.value=String(e);}
  finally{pending.value=false;}
}
function add(){
  const step=draft.value.trim();
  if(props.writable&&step&&!steps.value.includes(step))emit('add',step);
}
</script>
<template>
  <details class="play-review">
    <summary>Play 验收{{ steps.length ? ' '+passed.length+'/'+steps.length : '' }}</summary>
    <div class="review-menu xb-menu">
      <button v-if="!playing" class="xb-menu-item" @click="emit('play')">进入 Play 模式</button>
      <label v-for="(step,index) in steps" :key="step">
        <input type="checkbox" :checked="passed.includes(index)" :disabled="!playing||!writable||pending"
          @click.prevent="mark(index,!passed.includes(index))"> {{ step }}
      </label>
      <form @submit.prevent="add">
        <input v-model="draft" aria-label="交互验收项" placeholder="操作及预期结果" maxlength="240">
        <button type="submit" :disabled="!writable||!draft.trim()||steps.includes(draft.trim())">添加</button>
      </form>
      <small v-if="error" role="alert">{{error}}</small>
    </div>
  </details>
</template>
<style scoped>
.play-review{position:relative;flex:none}.play-review summary{cursor:pointer;white-space:nowrap}
.review-menu{position:absolute;top:100%;right:0;z-index:30;min-width:225px;max-width:min(300px,75vw);max-height:50vh;overflow:auto}
.review-menu label,.review-menu form{display:flex;gap:6px;align-items:baseline;padding:5px 8px}.review-menu form input{min-width:0;flex:1}
.review-menu small{color:var(--ui-danger)}
</style>
