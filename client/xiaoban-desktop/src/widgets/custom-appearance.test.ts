// @vitest-environment happy-dom
import {describe,it,expect} from 'vitest';
import {Window} from 'happy-dom';
import {installCustomAppearance} from './custom-appearance.ts';

describe('custom widget shadow limits',()=>{
  it('lays out at the requested resolution while keeping the template dimensions and its DOM state',async()=>{
    const win=new Window();
    win.document.body.innerHTML='<button data-count="5">Kitty</button>';
    new Function('document','window',`(${installCustomAppearance.toString()})();`)(win.document,win);
    win.document.dispatchEvent(new win.Event('DOMContentLoaded'));
    const button=win.document.querySelector('button');
    const resize=(scale:number,source=win)=>win.dispatchEvent(new win.MessageEvent('message',{source,data:{type:'xiaoban-widget-layout',scale,width:280,height:240}}));
    resize(1.3);
    expect(win.document.body.style.zoom).toBe('1.3');
    expect(win.document.body.style.width).toBe('280px');
    expect(win.document.body.style.height).toBe('240px');
    resize(.75);
    expect(win.document.body.style.zoom).toBe('0.75');
    expect(win.document.querySelector('button')).toBe(button);
    expect(button!.dataset.count).toBe('5');
    resize(NaN);resize(-1);
    expect(win.document.body.style.zoom).toBe('0.75');
    const other=new Window();resize(1.5,other);
    expect(win.document.body.style.zoom).toBe('0.75');
    await other.happyDOM.close();await win.happyDOM.close();
  });
  it('softens stylesheet, hover, nested and inline shadows while preserving inset and unrelated filters',async()=>{
    const win=new Window();
    win.document.head.innerHTML=`<style>
      .card{box-shadow:0 14px 34px rgba(255,140,175,.22),0 3px 10px rgba(255,180,205,.15),inset 0 1px 2px white;background:pink}
      .card:hover{box-shadow:0 6px 14px red!important}
      @media (min-width:1px){.card{filter:brightness(1.1) drop-shadow(0 4px 10px rgba(255,130,170,.25)) contrast(1.2)}}
      @keyframes pulse{to{box-shadow:0 10px 30px red}}
      .inset{box-shadow:inset 0 1px 2px white}
    </style>`;
    win.document.body.innerHTML='<div style="box-shadow:0 20px 40px red;filter:drop-shadow(0 10px 20px red) blur(1px)">Kitty</div>';
    new Function('document','window',`(${installCustomAppearance.toString()})();`)(win.document,win);
    win.document.dispatchEvent(new win.Event('DOMContentLoaded'));
    const rules=win.document.styleSheets[0]!.cssRules;
    expect(rules[0]!.cssText).toContain('0 2px 5px -3px rgba(0,0,0,0.12),inset 0 1px 2px white');
    expect(rules[0]!.cssText).toContain('background: pink');
    expect(rules[1]!.cssText).toContain('!important');
    expect(rules[1]!.cssText).not.toContain('14px');
    expect(rules[2]!.cssText).toContain('brightness(1.1) drop-shadow(0 1px 2px rgba(0,0,0,0.08)) contrast(1.2)');
    expect(rules[3]!.cssText).not.toContain('30px');
    expect(rules[4]!.cssText).toContain('inset 0 1px 2px white');
    expect(win.document.body.firstElementChild!.getAttribute('style')).toContain('drop-shadow(0 1px 2px rgba(0,0,0,0.08)) blur(1px)');
    await win.happyDOM.close();
  });
});
