/** Validation for the narrow, user-confirmed official MBTI export bridge. */
export function checkedMbtiResultLink(value:unknown):string | null {
  if(typeof value!=='string' || value.length>512)return null
  try {
    const u=new URL(value),p=u.searchParams
    if(u.origin!=='https://chat.xiangban.xyz'||u.pathname!=='/builtin-games/mbti/share.html'||u.hash||u.username||u.password)return null
    const type=p.get('t')||''
    if([...p.keys()].length!==4||!['t','p','n','d'].every(k=>p.getAll(k).length===1)||!/^[EI][SN][TF][JP]$/.test(type))return null
    const list=(key:string,max:number,min:number)=>{const s=p.get(key)||'';if(!/^\d{1,3}(,\d{1,3}){3}$/.test(s))return null;const values=s.split(',').map(Number);return values.every(n=>n>=min&&n<=max)?values:null}
    const scores=list('p',100,0),totals=list('n',12,1),date=p.get('d')||''
    if(!scores||!totals||!/^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d{3}Z$/.test(date)||!Number.isFinite(Date.parse(date)))return null
    if(scores.some((n,i)=>type[i]!==['EI','SN','TF','JP'][i]![n>=50?0:1]))return null
    return u.href
  }catch{return null}
}

export function decodeMbtiPng(value:unknown):Uint8Array<ArrayBuffer> | null {
  if(typeof value!=='string'||value.length>2_800_000||!value.startsWith('data:image/png;base64,'))return null
  try {
    const encoded=value.slice('data:image/png;base64,'.length)
    if(!/^[A-Za-z0-9+/]*={0,2}$/.test(encoded))return null
    const binary=atob(encoded)
    if(binary.length>2*1024*1024||!binary.startsWith('\x89PNG\r\n\x1a\n'))return null
    return Uint8Array.from(binary,c=>c.charCodeAt(0))
  }catch{return null}
}
