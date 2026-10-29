import holidays from './holidays.json';

export function dateKey(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth()+1).padStart(2,'0')}-${String(date.getDate()).padStart(2,'0')}`;
}
const lunarFormatter = new Intl.DateTimeFormat('zh-CN-u-ca-chinese', { month:'long', day:'numeric' });
const lunarDays = ['初一','初二','初三','初四','初五','初六','初七','初八','初九','初十','十一','十二','十三','十四','十五','十六','十七','十八','十九','二十','廿一','廿二','廿三','廿四','廿五','廿六','廿七','廿八','廿九','三十'];
const festivals: Record<string,string> = {'正月-1':'春节','正月-15':'元宵','二月-2':'龙抬头','五月-5':'端午','七月-7':'七夕','八月-15':'中秋','九月-9':'重阳','腊月-8':'腊八'};
export function lunarDate(date: Date) {
  const parts = lunarFormatter.formatToParts(new Date(date.getFullYear(), date.getMonth(), date.getDate(), 12));
  const month = parts.find(part=>part.type==='month')?.value ?? '';
  const day = Number(parts.find(part=>part.type==='day')?.value);
  const text = lunarDays[day-1] ?? '';
  return { month, day, text, full:`${month}${text}`, festival:festivals[`${month}-${day}`] ?? '' };
}
export function holidayInfo(date: Date) {
  const key=dateKey(date), year=holidays.find(item=>item.year===date.getFullYear());
  const holiday=year?.holidays.find(item=>key>=item.start&&key<=item.end);
  const makeup=year?.holidays.find(item=>item.makeup.includes(key));
  return { covered:!!year, holiday:holiday?.name??'', makeup:makeup?.name??'', badge:holiday?'休':makeup?'班':'' };
}
export function calendarLabel(date: Date) {
  const lunar=lunarDate(date), key=dateKey(date).slice(5);
  const solar:Record<string,string>={'01-01':'元旦','05-01':'劳动节','10-01':'国庆节'};
  return solar[key] || lunar.festival || (lunar.day===1?lunar.month:lunar.text);
}
