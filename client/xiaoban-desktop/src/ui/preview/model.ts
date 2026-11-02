import type {ConversationAsset} from '../chat/assets.ts';
export type PreviewTarget={sessionId:string;id:string;relative:string};
export type PreviewInfo={name:string;kind:'html'|'image'|'text';source:string|null;url:string;revision:string;token:string};
export type PreviewTab={key:string;name:string;target:PreviewTarget;path:string;version:string};
export function previewKind(name:string):PreviewInfo['kind']|null{
 const ext=name.split('.').at(-1)?.toLowerCase();
 if(ext==='html'||ext==='htm')return 'html';
 if(['png','jpg','jpeg','webp','gif','bmp','svg'].includes(ext??''))return 'image';
 if(['txt','md','markdown','json','csv','tsv','js','mjs','css','ts','tsx','jsx','xml','log','yaml','yml','toml'].includes(ext??''))return 'text';
 return null;
}
export function previewTab(asset:ConversationAsset,sessionId:string):PreviewTab|null{
 if(!asset.local||asset.local.kind==='folder'||!asset.local.exists||!previewKind(asset.name))return null;
 return {key:`${asset.id}:${asset.relative??''}`,name:asset.name,target:{sessionId,id:asset.id,relative:asset.relative??''},path:asset.local.path,version:`${asset.updatedMs}:${asset.local.bytes}:${asset.local.toolCallId??''}`};
}
export function assetForLink(assets:ConversationAsset[],href:string):ConversationAsset|undefined{
 let path=href;
 if(/^file:\/\//i.test(href)){try{const url=new URL(href);if(url.hostname&&url.hostname!=='localhost')return;path=decodeURIComponent(url.pathname);if(/^\/[a-z]:\//i.test(path))path=path.slice(1);}catch{return;}}
 else if(/^[a-z][a-z0-9+.-]*:/i.test(href)&&!/^\w:[\\/]/.test(href))return;
 const normalize=(p:string)=>p.replace(/\\/g,'/');
 const exact=assets.find(a=>a.local&&normalize(a.local.path)===normalize(path));if(exact)return exact;
 if(!path.includes('/')&&!path.includes('\\')){const same=assets.filter(a=>a.local&&a.name===path);if(same.length===1)return same[0];}
}
