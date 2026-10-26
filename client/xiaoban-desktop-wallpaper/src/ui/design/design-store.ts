import {reactive} from 'vue';
import {invoke,listen} from '../../bridge.ts';
import {clone,type Project,type Payload,type Summary} from './model.ts';
export function createDesignStore(notify:(message:unknown,type?:'success'|'error'|'info')=>void){
  let generation=0,disposed=false,off:(()=>void)|undefined;
  const state=reactive({project:null as Project|null,origin:'',summaries:[] as Summary[],busy:false,dirty:false,error:'',conflict:false,
    async list(session:string){const items=await invoke<Summary[]>('design_list',{session});state.summaries=Array.isArray(items)?items:[];return state.summaries;},
    async open(id:string){const g=++generation;const p=await invoke<Payload>('design_read',{id});if(g!==generation)return;state.project=p.draft??p.project;state.origin=p.origin;state.dirty=!!p.draft;state.error=p.draft?'已恢复未保存草稿，请重试保存或导出后读取新版':'';state.conflict=!!p.draft&&p.draft.revision!==p.project.revision;},
    close(){generation++;state.project=null;state.dirty=false;state.error='';},
    async commit(edit:(p:Project)=>void){if(!state.project||state.busy)return false;if(state.conflict){notify('请先处理版本冲突','error');return false;}
      try{const draft=clone(state.project);edit(draft);state.project=draft;state.dirty=true;return await state.save();}catch(e){notify(e,'error');return false;}},
    async save(){if(!state.project||state.busy)return false;state.busy=true;state.error='';const draft=clone(state.project),g=generation;
      try{try{await invoke('design_draft_save',{project:draft});}catch(e){notify('恢复草稿备份失败：'+String(e),'error');}const result=await invoke<Payload>('design_save',{project:draft,expectedRevision:draft.revision});if(g===generation){state.project=result.project;state.origin=result.origin;state.dirty=false;state.conflict=false;}void invoke('design_draft_clear',{project:draft}).catch(()=>{});return true;
      }catch(e){if(g===generation){state.error=String(e);state.conflict=/已更新|版本/.test(String(e));}notify(e,'error');return false;}finally{state.busy=false;}},
    async discard(){if(!state.project||state.busy)return;const draft=clone(state.project);try{await invoke('design_draft_clear',{project:draft});await state.open(draft.id);}catch(e){notify(e,'error');}},
    async refresh(){if(!state.project)return;if(state.dirty){state.conflict=true;state.error='设计稿有新版本，当前草稿尚未保存，请先导出草稿再读取新版';return;}await state.open(state.project.id);},
    async export(format:'html'|'zip'){if(!state.project)return;try{const path=await invoke<string|null>('design_export',{id:state.project.id,format,project:state.dirty?clone(state.project):null});if(path)notify('设计稿已导出','success');}catch(e){notify(e,'error');}},
    async init(onChange?:(summary:Summary)=>void){if(off||disposed)return;const stop=await listen<Summary>('design-changed',summary=>{onChange?.(summary);if(summary.id===state.project?.id&&summary.revision>state.project.revision&&!state.busy)void state.refresh().catch(e=>notify(e,'error'));});if(disposed)stop();else off=stop;},
    dispose(){disposed=true;generation++;off?.();off=undefined;},
  });return state;
}
export type DesignStore=ReturnType<typeof createDesignStore>;
