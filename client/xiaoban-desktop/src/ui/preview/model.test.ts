import {describe,it,expect} from 'vitest';
import {assetForLink,previewKind,previewTab} from './model.ts';
import type {ConversationAsset} from '../chat/assets.ts';
const asset=(name='demo.html',patch:Partial<ConversationAsset>={}):ConversationAsset=>({key:'local:a',id:'a',name,type:'file',updatedMs:4,local:{id:'a',name,path:`/work/${name}`,kind:'file',exists:true,imported:false,bytes:10,updatedMs:4},...patch});
describe('preview selection',()=>{
 it('limits executable previews to HTML, keeps code as text and unsupported formats external',()=>{expect(previewKind('DEMO.HTML')).toBe('html');expect(previewKind('app.tsx')).toBe('text');expect(previewKind('chart.svg')).toBe('image');for(const name of ['report.pdf','deck.pptx','app.exe','README'])expect(previewKind(name)).toBeNull();});
 it('keeps folder descendants bound to the original opaque root identity',()=>{const row=asset('nested/index.html',{relative:'nested/index.html'});expect(previewTab(row,'session')?.target).toEqual({sessionId:'session',id:'a',relative:'nested/index.html'});expect(previewTab({...row,local:{...row.local!,exists:false}},'session')).toBeNull();expect(previewTab({...row,local:{...row.local!,kind:'folder'}},'session')).toBeNull();});
 it('matches delivered local links without capturing remote URLs or ambiguous basenames',()=>{const a=asset('中文.html');expect(assetForLink([a],'file:///work/%E4%B8%AD%E6%96%87.html')).toBe(a);expect(assetForLink([a],'中文.html')).toBe(a);expect(assetForLink([a,a],'中文.html')).toBeUndefined();for(const link of ['https://site/中文.html','file://remote/work/中文.html','javascript:alert(1)'])expect(assetForLink([a],link)).toBeUndefined();});
 it('normalizes Windows file URLs',()=>{const a=asset();a.local!.path='C:\\work\\demo.html';expect(assetForLink([a],'file:///C:/work/demo.html')).toBe(a);});
});
