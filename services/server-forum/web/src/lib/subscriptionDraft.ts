import type { Schedule } from '../api/subscriptions'
/** Conservative convenience parser. A visible form is always confirmed before persistence. */
export function parseSubscriptionDraft(text:string):{recipient?:string;schedule?:Schedule} {
 const recipient=text.match(/[A-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[A-Z0-9.-]+\.[A-Z]{2,}/i)?.[0]
 const match=text.match(/(?:每天|每日|每周[一二三四五六日天])\s*(上午|早上|早晨|下午|晚上)?\s*(\d{1,2})(?::(\d{2})|点(半|\d{1,2}分?)?)/)
 if(!match)return {recipient}
 let hour=Number(match[2]); const minute=match[4]==='半'?30:Number(match[3]||match[4]?.replace('分','')||0)
 if(['下午','晚上'].includes(match[1]||'')&&hour<12)hour+=12
 if(hour>23||minute>59)return {recipient}
 const time=`${String(hour).padStart(2,'0')}:${String(minute).padStart(2,'0')}`
 const weekly=text.match(/每周([一二三四五六日天])/)
 return {recipient,schedule:weekly?{kind:'weekly',weekday:Math.min('一二三四五六日天'.indexOf(weekly[1]),6),time}:{kind:'daily',time}}
}
