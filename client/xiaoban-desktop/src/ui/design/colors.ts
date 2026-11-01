/** Normalize computed CSS colors before passing them to a native color control. */
export function rgb(value:string):number[]|null{
 const hex=/^#([\da-f]{3}|[\da-f]{6})$/i.exec(value.trim());
 if(hex){const raw=hex[1].length===3?[...hex[1]].map(c=>c+c).join(''):hex[1];return [0,2,4].map(i=>parseInt(raw.slice(i,i+2),16));}
 const match=/^rgba?\(\s*([\d.]+)[,\s]+([\d.]+)[,\s]+([\d.]+)(?:\s*[,/]\s*([\d.]+))?\s*\)$/i.exec(value.trim());
 if(!match||match[4]!==undefined&&Number(match[4])!==1)return null;
 return match.slice(1,4).map(v=>Math.min(255,Math.max(0,Number(v))));
}
export function colorHex(value:string,fallback='#000000'):string{const c=rgb(value);return c?'#'+c.map(v=>Math.round(v).toString(16).padStart(2,'0')).join(''):fallback;}
export function contrast(foreground:string,background:string):number|null{
 const colors=[rgb(foreground),rgb(background)];if(colors.some(c=>!c))return null;
 const [a,b]=colors.map(c=>c!.map(v=>v/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4).reduce((s,v,i)=>s+v*[.2126,.7152,.0722][i],0));return (Math.max(a,b)+.05)/(Math.min(a,b)+.05);
}
