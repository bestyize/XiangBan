import type { CountdownData, NoteData, TodoItem, ReminderItem, ItemRepeat } from './model.ts';
import { dateKey } from './calendar.ts';

export const emptyNote = (): NoteData => ({title:'待办事项',mode:'todo',text:'',items:[]});
export const emptyCountdown = (): CountdownData => ({title:'值得期待的日子',date:dateKey(new Date()),mode:'countdown',repeat:'none'});
export function validDate(value: unknown): value is string {
  if(typeof value!=='string'||!/^\d{4}-\d{2}-\d{2}$/.test(value)||value<'1900-01-01'||value>'2100-12-31')return false;
  const [year,month,day]=value.split('-').map(Number), date=new Date(year,month-1,day,12);
  return dateKey(date)===value;
}
export const validTime=(s:unknown):s is string=>typeof s==='string'&&/^([01]\d|2[0-3]):[0-5]\d$/.test(s);
export const repeats:{value:ItemRepeat;label:string}[]=[{value:'none',label:'不重复'},{value:'daily',label:'每天'},{value:'weekdays',label:'工作日（周一至周五）'},{value:'weekly',label:'每周'}];
export function validateTodo(item:TodoItem){
 if(!item||!/^[a-z0-9]{1,32}$/.test(item.id)||typeof item.text!=='string'||!item.text.trim()||item.text.length>120||typeof item.done!=='boolean'||item.time!==undefined&&item.time!==''&&!validTime(item.time)||item.date!==undefined&&item.date!==''&&!validDate(item.date))throw Error('待办内容或日期时间无效');
 validateSchedule(item,item.date??'',item.time??'',false);
}
function validateSchedule(item:{repeat?:ItemRepeat;notify?:boolean;completedOn?:string},date:string,time:string,defaultNotify:boolean){
 if(item.repeat!==undefined&&!repeats.some(r=>r.value===item.repeat)||item.notify!==undefined&&typeof item.notify!=='boolean'||item.completedOn!==undefined&&item.completedOn!==''&&!validDate(item.completedOn))throw Error('提醒配置无效');
 if((item.notify??defaultNotify)||item.repeat&&item.repeat!=='none'){if(!validDate(date)||!validTime(time))throw Error('开启提醒或重复时需要完整日期和时间');}
}
export function validateReminder(item:ReminderItem){
 if(!item||!/^[a-z0-9]{1,32}$/.test(item.id)||typeof item.title!=='string'||!item.title.trim()||item.title.length>60||typeof item.at!=='string'||item.at.length!==16||item.at[10]!=='T'||!validDate(item.at.slice(0,10))||!validTime(item.at.slice(11))||typeof item.done!=='boolean'||item.end!==undefined&&item.end!==''&&(!validTime(item.end)||item.end<=item.at.slice(11))||item.location!==undefined&&(typeof item.location!=='string'||item.location.length>80))throw Error('日程内容无效，请检查起止时间');
 validateSchedule(item,item.at.slice(0,10),item.at.slice(11),true);
}
export function validateNote(note:NoteData){
 if(!note||typeof note.title!=='string'||!note.title.trim()||note.title.length>40||note.mode!=='todo'||typeof note.text!=='string'||note.text.length>1200||!Array.isArray(note.items)||note.items.length>12)throw Error('待办内容无效');
 const ids=new Set<string>();for(const item of note.items){validateTodo(item);if(ids.has(item.id))throw Error('待办标识重复');ids.add(item.id);}
}
export function occursOn(at:string,repeat:ItemRepeat|undefined,day:Date){
 const key=dateKey(day),anchor=at.slice(0,10);if(key<anchor)return false;
 if(!repeat||repeat==='none')return key===anchor;
 if(repeat==='daily')return true;
 if(repeat==='weekdays')return day.getDay()>0&&day.getDay()<6;
 return new Date(anchor+'T12:00').getDay()===day.getDay();
}
export function itemDone(item:{done:boolean;repeat?:ItemRepeat;completedOn?:string},day=new Date()){return !item.repeat||item.repeat==='none'?item.done:item.completedOn===dateKey(day);}
export function toggleItem<T extends {done:boolean;repeat?:ItemRepeat;completedOn?:string}>(item:T,day=new Date()):T{return item.repeat&&item.repeat!=='none'?{...item,done:false,completedOn:itemDone(item,day)?'':dateKey(day)}:{...item,done:!item.done};}
export function scheduleLabel(date:string,time:string,repeat?:ItemRepeat,day=new Date()){
 const label=repeat&&repeat!=='none'?repeats.find(r=>r.value===repeat)?.label.replace('（周一至周五）',''):date===dateKey(day)?'今天':date?(date.slice(0,4)!==String(day.getFullYear())?date.slice(0,4)+'年':'')+date.slice(5).replace('-','月')+'日':'';
 return [label,time].filter(Boolean).join(' ');
}
export function validateCountdown(data: CountdownData) {
  if(!data||typeof data.title!=='string'||!data.title.trim()||data.title.length>40||!validDate(data.date)||!['countdown','elapsed'].includes(data.mode)||!['none','yearly'].includes(data.repeat)||(data.mode==='elapsed'&&data.repeat!=='none'))throw new Error('纪念日配置无效');
}
const dayNumber=(date:Date)=>Date.UTC(date.getFullYear(),date.getMonth(),date.getDate())/86400000;
export function countdownValue(data: CountdownData, now = new Date()) {
  const [year,month,day]=data.date.split('-').map(Number);
  const occurrence=(y:number)=>new Date(y,month-1,Math.min(day,new Date(y,month,0).getDate()),12);
  let target=occurrence(year);
  if(data.mode==='countdown'&&data.repeat==='yearly'){
    target=occurrence(Math.max(year,now.getFullYear()));
    if(dayNumber(target)<dayNumber(now))target=occurrence(target.getFullYear()+1);
  }
  const diff=dayNumber(target)-dayNumber(now);
  if(data.mode==='elapsed')return {days:Math.abs(diff),label:diff>0?'还有':diff===0?'就是今天':'已经',date:dateKey(target)};
  return {days:Math.abs(diff),label:diff<0?'已过去':diff===0?'就是今天':'还有',date:dateKey(target)};
}
