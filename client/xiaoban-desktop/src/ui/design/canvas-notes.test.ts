// @vitest-environment happy-dom
import {mount} from '@vue/test-utils';
import {describe,expect,it} from 'vitest';
import CanvasNoteView from './CanvasNote.vue';
import {notesOf,patchNote,addNote} from './canvas-notes.ts';
import {colorHex,contrast} from './colors.ts';
import type {Project} from './model.ts';
describe('Claude canvas notes and property feedback',()=>{
 it('keeps original title sizing and unknown fields when editing and duplicating notes',()=>{
  const p={canvas:{notes:{heading:{kind:'title1',x:0,y:-300,w:240,maxW:6320,text:'原始标题',vendor:{a:1}}}}} as unknown as Project;
  expect(notesOf(p.canvas)[0]).toMatchObject({w:6320,y:-300});patchNote(p,'heading',{text:'修改',w:700});expect(p.canvas.notes.heading).toMatchObject({w:240,maxW:700,vendor:{a:1}});
  const id=addNote(p,0,0,undefined,notesOf(p.canvas)[0]);expect(p.canvas.notes[id]).toMatchObject({x:24,y:-276,text:'修改',vendor:{a:1}});expect(p.canvas.notes.heading.x).toBe(0);
 });
 it('edits notes, handles IME, cancels Escape and commits Tab without changing artboard source',async()=>{
  const note=notesOf({notes:{a:{text:'原文',x:0,y:0,w:320}}})[0];
  const w=mount(CanvasNoteView,{props:{note,selected:true,writable:true,scale:.5,offsetX:0,offsetY:0}});
  await w.trigger('dblclick');await w.get('textarea').setValue('新文');await w.get('textarea').trigger('keydown',{key:'Enter',ctrlKey:true,isComposing:true});expect(w.emitted('patch')).toBeUndefined();
  await w.get('textarea').trigger('keydown',{key:'Tab'});expect(w.emitted('patch')?.[0]).toEqual([{text:'新文'}]);
  await w.trigger('keydown',{key:'ArrowRight',shiftKey:true});expect(w.emitted('patch')?.at(-1)).toEqual([{x:10,y:0}]);w.unmount();
 });
 it('converts computed colors to valid color inputs and reports low contrast accurately',()=>{
  expect(colorHex('rgb(217, 78, 96)')).toBe('#d94e60');expect(colorHex('#abc')).toBe('#aabbcc');expect(contrast('#000','#fff')).toBeCloseTo(21);expect(contrast('#aaa','#fff')).toBeLessThan(4.5);expect(contrast('rgba(0,0,0,0)','#fff')).toBeNull();
 });
});
