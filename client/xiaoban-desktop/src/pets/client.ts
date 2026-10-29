import {ref} from 'vue';
import {desktop,invoke} from '../bridge.ts';
import {OFFICIAL_THEMES,validateTheme,validateLayout,definitionOf,type PetCatalog,type PetInstance,type PetTheme} from './model.ts';
const layoutKey='xiaoban.pets.layout.v1',themesKey='xiaoban.pets.themes.v1';
export function createPetClient(){
 const catalog=ref<PetCatalog>({themes:[],layout:{version:1,instances:[]},errors:[]}),ready=ref(false),busy=ref(false),error=ref('');
 let queue=Promise.resolve();
 function accept(value:PetCatalog){value.themes.forEach(validateTheme);validateLayout(value.layout);catalog.value=value;ready.value=!value.errors.length;error.value=value.errors.join('；');}
 async function load(){try{if(desktop)accept(await invoke<PetCatalog>('get_pet_catalog'));else{const custom=(JSON.parse(localStorage.getItem(themesKey)||'[]')as PetTheme[]).map(validateTheme);accept({themes:[...OFFICIAL_THEMES.map(t=>({...t,source:'builtin' as const})),...custom.map(t=>({...t,source:'custom' as const}))],layout:validateLayout(JSON.parse(localStorage.getItem(layoutKey)||'{"version":1,"instances":[]}')),errors:[]});}}catch(e){error.value=String(e);ready.value=false;}}
 function change(action:()=>Promise<void>){const pending=queue.catch(()=>{}).then(async()=>{if(!ready.value)throw Error('宠物库尚未载入');busy.value=true;error.value='';try{await action();await load();}catch(e){error.value=String(e);throw e;}finally{busy.value=false;}});queue=pending;return pending;}
 function saveLayout(instances:PetInstance[]){const next=validateLayout({version:1,instances});localStorage.setItem(layoutKey,JSON.stringify(next));}
 async function add(themeId:string,petId:string,layer:PetInstance['layer']='topmost'){
  let id='';await change(async()=>{const pet=catalog.value.themes.find(t=>t.id===themeId)?.pets.find(p=>p.id===petId);if(!pet)throw Error('没有找到这个宠物');
   if(desktop){id=(await invoke<{id:string}>('add_pet',{themeId,petId,layer})).id;return;}
   id=crypto.randomUUID().replaceAll('-','').slice(0,16);const count=catalog.value.layout.instances.length;
   saveLayout([...catalog.value.layout.instances,{id,themeId,petId,name:pet.name,size:210,x:.1+(count%4)*.23,y:Math.min(.85,.15+Math.floor(count/4)*.18),layer,enabled:true,expression:'idle',links:true}]);
  });return id;
 }
 function update(id:string,patch:Partial<Omit<PetInstance,'id'>>){return change(async()=>{if(desktop){await invoke('update_pet',{id,patch});return;}const instances=catalog.value.layout.instances.map(p=>p.id===id?{...p,...patch}:p);if(!catalog.value.layout.instances.some(p=>p.id===id))throw Error('宠物已被移除');const changed=instances.find(p=>p.id===id)!;if(!definitionOf(catalog.value.themes,changed))throw Error('没有找到这个宠物作品');saveLayout(instances);});}
 function remove(id:string){return change(async()=>{if(desktop)await invoke('remove_pet',{id});else saveLayout(catalog.value.layout.instances.filter(p=>p.id!==id));});}
 function saveTheme(value:unknown,overwrite=false){return change(async()=>{const theme=validateTheme(value);if(OFFICIAL_THEMES.some(t=>t.id===theme.id))throw Error('不能覆盖官方主题，请使用新的主题 id');if(desktop){await invoke('save_pet_theme',{theme,overwrite});return;}
  const custom=catalog.value.themes.filter(t=>t.source==='custom');if(custom.some(t=>t.id===theme.id)&&!overwrite)throw Error('主题已存在');if(catalog.value.layout.instances.some(p=>p.themeId===theme.id&&!theme.pets.some(d=>d.id===p.petId)))throw Error('请先移除仍在桌面上的宠物');localStorage.setItem(themesKey,JSON.stringify([...custom.filter(t=>t.id!==theme.id).map(({source,...t})=>t),theme]));
 });}
 function deleteTheme(id:string){return change(async()=>{if(OFFICIAL_THEMES.some(t=>t.id===id))throw Error('不能删除官方主题');if(catalog.value.layout.instances.some(p=>p.themeId===id))throw Error('请先移除该主题已添加的桌面宠物');if(desktop)await invoke('delete_pet_theme',{id});else localStorage.setItem(themesKey,JSON.stringify(catalog.value.themes.filter(t=>t.source==='custom'&&t.id!==id).map(({source,...t})=>t)));});}
 async function exportTheme(id:string){if(desktop)return invoke('export_pet_theme',{id});const theme=catalog.value.themes.find(t=>t.id===id);if(!theme)throw Error('主题不存在');const {source,...data}=theme;const url=URL.createObjectURL(new Blob([JSON.stringify(data,null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download=`${id}.json`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);}
 function changed(){void load();}window.addEventListener('pets-changed',changed);window.addEventListener('focus',changed);
 return {catalog,ready,busy,error,load,add,update,remove,saveTheme,deleteTheme,exportTheme,dispose(){window.removeEventListener('pets-changed',changed);window.removeEventListener('focus',changed);}};
}
