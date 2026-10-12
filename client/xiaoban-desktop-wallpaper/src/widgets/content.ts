import type { CountdownData, NoteData } from './model.ts';
import { dateKey } from './calendar.ts';

export const emptyNote = (): NoteData => ({title:'随手记',mode:'note',text:'',items:[]});
export const emptyCountdown = (): CountdownData => ({title:'值得期待的日子',date:dateKey(new Date()),mode:'countdown',repeat:'none'});
export function validDate(value: unknown): value is string {
  if(typeof value!=='string'||!/^\d{4}-\d{2}-\d{2}$/.test(value)||value<'1900-01-01'||value>'2100-12-31')return false;
  const [year,month,day]=value.split('-').map(Number), date=new Date(year,month-1,day,12);
  return dateKey(date)===value;
}
export function validateNote(note: NoteData) {
  if(!note||typeof note.title!=='string'||!note.title.trim()||note.title.length>40||!['note','todo'].includes(note.mode)||typeof note.text!=='string'||note.text.length>1200||!Array.isArray(note.items)||note.items.length>12)throw new Error('便签内容无效或超过长度限制');
  const ids=new Set<string>();
  for(const item of note.items){if(!item||typeof item.id!=='string'||!/^[a-z0-9]{1,32}$/.test(item.id)||ids.has(item.id)||typeof item.text!=='string'||!item.text.trim()||item.text.length>120||typeof item.done!=='boolean')throw new Error('待办内容无效');ids.add(item.id);}
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
