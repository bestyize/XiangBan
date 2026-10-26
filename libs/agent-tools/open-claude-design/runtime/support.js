/* Independent DC HTML compatibility runtime. Runs only in an opaque-origin sandbox. */
(() => {
  'use strict';
  const send = (type, data = {}) => parent.postMessage({channel:'open-claude-design', token:new URLSearchParams(location.search).get('token'),type,...data}, '*');
  const report = e => send('error', {message:String(e?.message || e).slice(0,1000)});
  addEventListener('error', e => report(e.error || e.message));
  addEventListener('unhandledrejection', e => report(e.reason));
  let editing = true;
  const evaluate = (text, scope) => Function('scope', `with(scope){return (${text})}`)(scope);
  const expr = (text, scope) => {
    const match = /^\s*{{([\s\S]*?)}}\s*$/.exec(text);
    if (match) return evaluate(match[1], scope);
    return text.replace(/{{([\s\S]*?)}}/g, (_, code) => String(evaluate(code, scope) ?? ''));
  };
  const bool = v => !(v === false || v == null || v === 'false' || v === '');
  function expand(node, scope) {
    if (node.nodeType === Node.TEXT_NODE) { node.textContent = String(expr(node.textContent || '',scope) ?? ''); return; }
    if (!(node instanceof Element)) return;
    if (node.localName === 'sc-if') {
      if (!bool(expr(node.getAttribute('value') || '',scope))) { node.remove(); return; }
      const children = [...node.childNodes]; children.forEach(n=>expand(n,scope)); node.replaceWith(...node.childNodes); return;
    }
    if (node.localName === 'sc-for') {
      const list = expr(node.getAttribute('list') || '',scope);
      if (!Array.isArray(list)) throw Error('sc-for list must be an array');
      const fragment = document.createDocumentFragment(), name = node.getAttribute('as') || 'item';
      for (let i=0;i<Math.min(list.length,1000);i++) {
        const childScope = {...scope,[name]:list[i],$index:i};
        for (const source of [...node.childNodes]) { const copy=source.cloneNode(true); fragment.append(copy); expand(copy,childScope); }
      }
      node.replaceWith(fragment); return;
    }
    if (node.localName === 'dc-import') throw Error('此画板包含尚未支持的 dc-import 组件，请让小伴展开为 HTML');
    for (const attr of [...node.attributes]) {
      if (attr.name.startsWith('on')) {
        node.removeAttribute(attr.name);
        const handler=expr(attr.value,scope);
        if (typeof handler==='function') node.addEventListener(attr.name.slice(2).toLowerCase(), e=>{if(!editing)try{handler(e);}catch(error){report(error);}});
        continue;
      }
      if (attr.value.includes('{{')) {
        const v=expr(attr.value,scope);
        if (['checked','disabled','selected','hidden'].includes(attr.name)) { if(bool(v))node.setAttribute(attr.name,'');else node.removeAttribute(attr.name); }
        else node.setAttribute(attr.name, String(v ?? ''));
      }
    }
    const hover=node.getAttribute('style-hover');
    if(hover){const original=node.getAttribute('style')||'';node.addEventListener('mouseenter',()=>{node.style.cssText=original+';'+hover;});node.addEventListener('mouseleave',()=>{node.style.cssText=original;});}
    [...node.childNodes].forEach(n=>expand(n,scope));
  }
  addEventListener('DOMContentLoaded', () => {
    const root=document.querySelector('x-dc');
    let instance, render, mounted=false;
    {
      const host=root||document.body,used=new Set([...host.querySelectorAll('[data-ocd-node]')].map(el=>el.getAttribute('data-ocd-node')));let n=0;
      host.querySelectorAll('*').forEach(el=>{if(!el.hasAttribute('data-ocd-node')){while(used.has('n'+n))n++;el.setAttribute('data-ocd-node','n'+n);used.add('n'+n++);}});
    }
    if(root) {
      try {
        const template=root.cloneNode(true), script=document.querySelector('script[data-dc-script]');
        let config={};
        if(script?.getAttribute('data-props')) config=JSON.parse(script.getAttribute('data-props'));
        const props=Object.fromEntries(Object.entries(config).filter(([k])=>k!=='$preview').map(([k,v])=>[k,v?.default]));
        class DCLogic {
          constructor(p){this.props=p;this.state={};}
          setState(value){this.state={...this.state,...(typeof value==='function'?value(this.state,this.props):value)};render?.();}
        }
        if(script) {
          const Component=Function('DCLogic',script.textContent+'\n;return Component;')(DCLogic);
          instance=new Component(props);
        } else instance=new DCLogic(props);
        render=()=>{
          const selection=document.activeElement?.getAttribute('data-ocd-node');
          const rangeStart=document.activeElement?.selectionStart,rangeEnd=document.activeElement?.selectionEnd;
          const clone=template.cloneNode(true), scope=instance.renderVals?instance.renderVals():{};
          expand(clone,scope);
          document.querySelectorAll('[data-ocd-helmet]').forEach(n=>n.remove());
          clone.querySelectorAll('helmet').forEach(h=>{for(const n of [...h.children]){n.setAttribute('data-ocd-helmet','');document.head.append(n);}h.remove();});
          root.replaceChildren(...clone.childNodes);
          if(selection){const active=[...root.querySelectorAll('[data-ocd-node]')].find(n=>n.getAttribute('data-ocd-node')===selection);active?.focus();if(typeof active?.setSelectionRange==='function')try{active.setSelectionRange(rangeStart,rangeEnd);}catch{}}
        };
        render(); mounted=true;instance.componentDidMount?.();
        addEventListener('unload',()=>{if(mounted)instance.componentWillUnmount?.();});
      } catch(e){report(e);}
    }
    document.addEventListener('click', e=>{
      const target=e.target.closest?.('[data-ocd-node]');
      if(editing){e.preventDefault();e.stopImmediatePropagation();if(target)send('select',{node:target.getAttribute('data-ocd-node'),tag:target.tagName,text:target.children.length?'':target.textContent,style:target.getAttribute('style')||''});return;}
      const a=e.target.closest?.('a[href]');
      if(a){e.preventDefault();const href=a.getAttribute('href');if(href.startsWith('#')){document.getElementById(href.slice(1))?.scrollIntoView();}else send('navigate',{file:href});}
    },true);
    addEventListener('message',e=>{if(e.source!==parent || e.data?.channel!=='open-claude-design')return;
      if(e.data.type==='mode')editing=e.data.editing;
    });
    send('ready');
  });
})();
