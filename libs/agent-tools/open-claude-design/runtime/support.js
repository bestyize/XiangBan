/* Independent DC HTML compatibility runtime. Runs only in an opaque-origin sandbox. */
(() => {
  'use strict';
  const send = (type, data = {}) => parent.postMessage({channel:'open-claude-design', token:new URLSearchParams(location.search).get('token'),type,...data}, '*');
  const report = e => send('error', {message:String(e?.message || e).slice(0,1000)});
  addEventListener('error', e => report(e.error || e.message));
  addEventListener('unhandledrejection', e => report(e.reason));
  let editing = true, commenting = false, pins = [], DCLogicBase;
  const bundleEl = () => document.getElementById('__ocd_bundle__');
  const readBundle = () => {
    try {
      const raw = bundleEl()?.textContent;
      const parsed = raw ? JSON.parse(raw) : {};
      return {
        base: parsed.base || '',
        files: {...(parsed.files || {}), ...(window.__OCD_FILES__ || {})},
        tokens: {...(parsed.tokens || {}), ...(window.__OCD_TOKENS__ || {})},
      };
    } catch { return {base:'', files:window.__OCD_FILES__ || {}, tokens:window.__OCD_TOKENS__ || {}}; }
  };
  const resolvePath = (base, rel) => {
    if (!rel) return '';
    const clean = String(rel).trim().replace(/^\.\//, '');
    if (!base || !base.includes('/')) {
      const out = [];
      for (const p of clean.split('/')) { if (p === '..') out.pop(); else if (p && p !== '.') out.push(p); }
      return out.join('/');
    }
    const parts = base.split('/'); parts.pop();
    for (const p of String(rel).trim().split('/')) { if (p === '..') parts.pop(); else if (p && p !== '.') parts.push(p); }
    return parts.join('/');
  };
  const evaluate = (text, scope) => Function('scope', `with(scope){return (${text})}`)(scope);
  const expr = (text, scope) => {
    const match = /^\s*{{([^{}]+)}}\s*$/.exec(text);
    if (match) return evaluate(match[1], scope);
    if (text.includes('{{')) return text.replace(/{{([^{}]+)}}/g, (_, code) => String(evaluate(code, scope) ?? ''));
    return text;
  };
  const bool = v => v !== 'false' && Boolean(v);
  const pxProps = new Set(['width','height','min-width','min-height','max-width','max-height','font-size','border-radius','border-width','padding','padding-top','padding-right','padding-bottom','padding-left','margin','margin-top','margin-right','margin-bottom','margin-left','gap','top','left','right','bottom']);
  const formatStyle = v => {
    if (!v || typeof v !== 'object' || Array.isArray(v)) return String(v ?? '');
    return Object.entries(v).filter(([,val]) => val !== undefined && val !== null && val !== false && val !== '').map(([k,val]) => {
      const key = k.replace(/[A-Z]/g, m => '-' + m.toLowerCase());
      const str = typeof val === 'number' && pxProps.has(key) ? `${val}px` : String(val);
      return `${key}:${str}`;
    }).join(';');
  };
  const formatClass = v => {
    if (Array.isArray(v)) return v.flat().filter(Boolean).join(' ');
    if (v && typeof v === 'object') return Object.entries(v).filter(([,on]) => bool(on)).map(([k]) => k).join(' ');
    return String(v ?? '');
  };
  const parseStyleMap = css => {
    const out = {};
    for (const part of String(css || '').split(';')) {
      const idx = part.indexOf(':');
      if (idx > 0) {
        const k = part.slice(0, idx).trim().toLowerCase(), v = part.slice(idx + 1).trim();
        if (k && v) out[k] = v;
      }
    }
    return out;
  };
  const nodePath = el => {
    const parts = [];
    let cur = el;
    while (cur && cur instanceof Element && cur.localName !== 'x-dc' && cur.localName !== 'body' && cur.localName !== 'html' && parts.length < 6) {
      let label = cur.localName;
      if (cur.id) label += `#${cur.id}`;
      else if (cur.classList?.length) label += '.' + [...cur.classList].slice(0, 2).join('.');
      parts.unshift(label);
      cur = cur.parentElement;
    }
    return parts.join(' > ') || el?.localName || '';
  };
  function expandChildren(parent, scope, ctx) {
    let ifMatched = false;
    for (const node of [...parent.childNodes]) {
      if (node instanceof Element) {
        if (node.localName === 'sc-if') {
          ifMatched = bool(expr(node.getAttribute('value') || '', scope));
          if (!ifMatched) { node.remove(); continue; }
          expandChildren(node, scope, ctx);
          node.replaceWith(...node.childNodes);
          continue;
        }
        if (node.localName === 'sc-else-if') {
          if (ifMatched || !bool(expr(node.getAttribute('value') || '', scope))) { node.remove(); continue; }
          ifMatched = true;
          expandChildren(node, scope, ctx);
          node.replaceWith(...node.childNodes);
          continue;
        }
        if (node.localName === 'sc-else') {
          if (ifMatched) { node.remove(); continue; }
          ifMatched = true;
          expandChildren(node, scope, ctx);
          node.replaceWith(...node.childNodes);
          continue;
        }
        ifMatched = false;
      }
      expand(node, scope, ctx);
    }
  }
  function expandImport(node, scope, ctx) {
    const rawSrc = node.getAttribute('src') || node.getAttribute('file') || node.getAttribute('component') || '';
    const resolved = resolvePath(ctx.file, rawSrc);
    const source = ctx.files[resolved] ?? ctx.files[rawSrc];
    if (!source) throw Error(`此画板包含尚未支持的 dc-import 组件或缺少源文件：${rawSrc || '未指定 src'}`);
    if (ctx.depth >= 16) throw Error(`dc-import 组件嵌套超过上限：${rawSrc}`);
    const doc = new DOMParser().parseFromString(source, 'text/html');
    const compRoot = doc.querySelector('x-dc') || doc.body;
    const script = doc.querySelector('script[data-dc-script]');
    let config = {};
    if (script?.getAttribute('data-props')) { try { config = JSON.parse(script.getAttribute('data-props')); } catch {} }
    const compProps = Object.fromEntries(Object.entries(config).filter(([k]) => k !== '$preview').map(([k, v]) => [k, v?.default]));
    for (const attr of [...node.attributes]) {
      if (['src', 'file', 'component', 'data-ocd-node'].includes(attr.name)) continue;
      compProps[attr.name] = attr.value.includes('{{') ? expr(attr.value, scope) : attr.value;
    }
    let compScope = {$tokens: ctx.tokens, ...compProps};
    if (script && DCLogicBase) {
      const Comp = Function('DCLogic', script.textContent + '\n;return typeof Component!=="undefined"?Component:DCLogic;')(DCLogicBase);
      const inst = new Comp(compProps);
      if (inst.renderVals) compScope = {$tokens: ctx.tokens, ...compProps, ...inst.renderVals()};
    }
    const slotNodes = [...node.childNodes].map(n => n.cloneNode(true));
    slotNodes.forEach(n => expand(n, scope, ctx));
    for (const slot of [...compRoot.querySelectorAll('slot,dc-slot')]) {
      const name = slot.getAttribute('name');
      const matched = name
        ? slotNodes.filter(n => n instanceof Element && n.getAttribute('slot') === name)
        : slotNodes.filter(n => !(n instanceof Element && n.hasAttribute('slot')));
      if (matched.length) slot.replaceWith(...matched.map(n => n.cloneNode(true)));
      else slot.replaceWith(...slot.childNodes);
    }
    for (const h of [...compRoot.querySelectorAll('helmet')]) {
      for (const n of [...h.children]) { n.setAttribute('data-ocd-helmet', ''); document.head.append(n.cloneNode(true)); }
      h.remove();
    }
    const wrapper = document.createElement('div');
    wrapper.append(...[...compRoot.childNodes].map(n => n.cloneNode(true)));
    expandChildren(wrapper, compScope, {...ctx, file: resolved || rawSrc, depth: ctx.depth + 1});
    if (node.hasAttribute('data-ocd-node') && wrapper.firstElementChild && !wrapper.firstElementChild.hasAttribute('data-ocd-node')) {
      wrapper.firstElementChild.setAttribute('data-ocd-node', node.getAttribute('data-ocd-node'));
    }
    node.replaceWith(...wrapper.childNodes);
  }
  function expand(node, scope, ctx) {
    if (node.nodeType === Node.TEXT_NODE) { node.textContent = String(expr(node.textContent || '', scope) ?? ''); return; }
    if (!(node instanceof Element)) return;
    if (node.localName === 'sc-if' || node.localName === 'sc-else-if' || node.localName === 'sc-else') {
      if (node.localName !== 'sc-else' && !bool(expr(node.getAttribute('value') || '', scope))) { node.remove(); return; }
      expandChildren(node, scope, ctx); node.replaceWith(...node.childNodes); return;
    }
    if (node.localName === 'sc-for') {
      const list = expr(node.getAttribute('list') || '', scope);
      if (!Array.isArray(list)) throw Error('sc-for list must be an array');
      const fragment = document.createDocumentFragment(), name = node.getAttribute('as') || 'item', idxName = node.getAttribute('index');
      for (let i = 0; i < Math.min(list.length, 1000); i++) {
        const childScope = {...scope, [name]: list[i], $index: i, ...(idxName ? {[idxName]: i} : {})};
        const holder = document.createElement('div');
        for (const source of [...node.childNodes]) holder.append(source.cloneNode(true));
        expandChildren(holder, childScope, ctx);
        fragment.append(...holder.childNodes);
      }
      node.replaceWith(fragment); return;
    }
    if (node.localName === 'dc-import') { expandImport(node, scope, ctx); return; }
    for (const attr of [...node.attributes]) {
      if (attr.name.startsWith('on')) {
        node.removeAttribute(attr.name);
        const handler = expr(attr.value, scope);
        if (typeof handler === 'function') node.addEventListener(attr.name.slice(2).toLowerCase(), e => { if (!editing) try { handler(e); } catch (error) { report(error); } });
        continue;
      }
      if (attr.value.includes('{{')) {
        const v = expr(attr.value, scope);
        if (['checked', 'disabled', 'selected', 'hidden'].includes(attr.name)) { if (bool(v)) node.setAttribute(attr.name, ''); else node.removeAttribute(attr.name); }
        else if (attr.name === 'style') node.setAttribute('style', formatStyle(v));
        else if (attr.name === 'class') node.setAttribute('class', formatClass(v));
        else node.setAttribute(attr.name, String(v ?? ''));
      }
    }
    const hover = node.getAttribute('style-hover');
    if (hover) { const original = node.getAttribute('style') || ''; node.addEventListener('mouseenter', () => { node.style.cssText = original + ';' + hover; }); node.addEventListener('mouseleave', () => { node.style.cssText = original; }); }
    const active = node.getAttribute('style-active');
    if (active) { const original = node.getAttribute('style') || ''; node.addEventListener('mousedown', () => { node.style.cssText = original + ';' + active; }); node.addEventListener('mouseup', () => { node.style.cssText = original; }); }
    expandChildren(node, scope, ctx);
  }
  function renderPins() {
    document.querySelectorAll('[data-ocd-pin-layer]').forEach(n => n.remove());
    if (!editing || !Array.isArray(pins) || !pins.length) return;
    const layer = document.createElement('div');
    layer.setAttribute('data-ocd-pin-layer', '');
    layer.style.cssText = 'position:fixed;inset:0;pointer-events:none;z-index:2147483646';
    for (const pin of pins) {
      const target = pin.node ? document.querySelector(`[data-ocd-node="${CSS.escape ? CSS.escape(pin.node) : pin.node}"]`) : null;
      const rect = target?.getBoundingClientRect?.();
      const badge = document.createElement('span');
      const left = rect ? Math.max(4, Math.min(innerWidth - 24, rect.right - 10)) : 12;
      const top = rect ? Math.max(4, Math.min(innerHeight - 24, rect.top - 8)) : 12 + (Number(pin.num || 1) - 1) * 24;
      badge.textContent = String(pin.num || '•');
      badge.title = String(pin.text || '');
      badge.style.cssText = `position:fixed;left:${left}px;top:${top}px;min-width:18px;height:18px;padding:0 4px;border-radius:999px;background:#d94e60;color:#fff;font:600 10px/18px system-ui;text-align:center;box-shadow:0 2px 6px rgba(0,0,0,.22)`;
      layer.append(badge);
    }
    document.body.append(layer);
  }
  addEventListener('DOMContentLoaded', () => {
    const root = document.querySelector('x-dc');
    const bundle = readBundle();
    let instance, render, mounted = false;
    {
      const host = root || document.body, used = new Set([...host.querySelectorAll('[data-ocd-node]')].map(el => el.getAttribute('data-ocd-node'))); let n = 0;
      host.querySelectorAll('*').forEach(el => { if (!el.hasAttribute('data-ocd-node')) { while (used.has('n' + n)) n++; el.setAttribute('data-ocd-node', 'n' + n); used.add('n' + n++); } });
    }
    if (root) {
      try {
        const template = root.cloneNode(true), script = document.querySelector('script[data-dc-script]');
        let config = {};
        if (script?.getAttribute('data-props')) config = JSON.parse(script.getAttribute('data-props'));
        const props = Object.fromEntries(Object.entries(config).filter(([k]) => k !== '$preview').map(([k, v]) => [k, v?.default]));
        class DCLogic {
          constructor(p) { this.props = p; this.state = {}; this.tokens = bundle.tokens; }
          setState(value) { this.state = {...this.state, ...(typeof value === 'function' ? value(this.state, this.props) : value)}; render?.(); }
        }
        DCLogicBase = DCLogic;
        if (script) {
          const Component = Function('DCLogic', script.textContent + '\n;return Component;')(DCLogic);
          instance = new Component(props);
        } else instance = new DCLogic(props);
        render = () => {
          const selection = document.activeElement?.getAttribute('data-ocd-node');
          const rangeStart = document.activeElement?.selectionStart, rangeEnd = document.activeElement?.selectionEnd;
          const clone = template.cloneNode(true), scope = {$tokens: bundle.tokens, ...instance.props, ...(instance.renderVals ? instance.renderVals() : {})};
          expandChildren(clone, scope, {file: bundle.base, files: bundle.files, tokens: bundle.tokens, depth: 0});
          document.querySelectorAll('[data-ocd-helmet]').forEach(n => n.remove());
          clone.querySelectorAll('helmet').forEach(h => { for (const n of [...h.children]) { n.setAttribute('data-ocd-helmet', ''); document.head.append(n); } h.remove(); });
          root.replaceChildren(...clone.childNodes);
          if (selection) { const active = [...root.querySelectorAll('[data-ocd-node]')].find(n => n.getAttribute('data-ocd-node') === selection); active?.focus(); if (typeof active?.setSelectionRange === 'function') try { active.setSelectionRange(rangeStart, rangeEnd); } catch {} }
          renderPins();
        };
        render(); mounted = true; instance.componentDidMount?.();
        addEventListener('unload', () => { if (mounted) instance.componentWillUnmount?.(); });
      } catch (e) { report(e); }
    }
    for(const type of ['gesturestart','gesturechange'])document.addEventListener(type,e=>{e.preventDefault();send('canvas-gesture',{start:type==='gesturestart',scale:e.scale,x:e.clientX,y:e.clientY});},{passive:false});
    // Opaque iframes do not bubble input to the canvas host.
    document.addEventListener('wheel', e => {
      if (!editing && !e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      send('canvas-wheel',{dx:e.deltaX,dy:e.deltaY,deltaMode:e.deltaMode,ctrl:e.ctrlKey,meta:e.metaKey,shift:e.shiftKey,x:e.clientX,y:e.clientY});
    },{passive:false});
    for (const type of ['keydown','keyup']) document.addEventListener(type,e=>{
      if(!editing||e.target?.closest?.('input,textarea,select,[contenteditable]'))return;
      if([' ','+','=','-','0','1'].includes(e.key)){e.preventDefault();send('canvas-key',{key:e.key,down:type==='keydown'});}
    });
    let pan=null;
    document.addEventListener('pointerdown',e=>{if(e.button===1){e.preventDefault();pan={id:e.pointerId,x:e.clientX,y:e.clientY};e.target?.setPointerCapture?.(e.pointerId);}},true);
    document.addEventListener('pointermove',e=>{if(pan&&pan.id===e.pointerId){send('canvas-pan',{dx:e.clientX-pan.x,dy:e.clientY-pan.y});pan.x=e.clientX;pan.y=e.clientY;}});
    for(const type of ['pointerup','pointercancel'])document.addEventListener(type,()=>pan=null);
    document.addEventListener('click', e => {
      const target = e.target.closest?.('[data-ocd-node]');
      if (editing) {
        e.preventDefault(); e.stopImmediatePropagation();
        if (target) {
          const style = target.getAttribute('style') || '';
          const r = target.getBoundingClientRect?.() || {left:0, top:0, width:0, height:0};
          const selText = String(window.getSelection?.()?.toString?.() || '').trim();
          const quote = (selText || target.textContent || '').trim().slice(0, 160);
          send('select', {
            node: target.getAttribute('data-ocd-node'),
            tag: target.tagName,
            text: target.children.length ? '' : target.textContent,
            quote,
            path: nodePath(target),
            style,
            styles: parseStyleMap(style),
            rect: {x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height)},
            commenting,
          });
        }
        return;
      }
      const a = e.target.closest?.('a[href]');
      if (a) { e.preventDefault(); const href = a.getAttribute('href'); if (href.startsWith('#')) { document.getElementById(href.slice(1))?.scrollIntoView(); } else send('navigate', {file: href}); }
    }, true);
    addEventListener('message', e => {
      if (e.source !== parent || e.data?.channel !== 'open-claude-design') return;
      if (e.data.type === 'mode') { editing = Boolean(e.data.editing); commenting = Boolean(e.data.commenting); renderPins(); }
      if (e.data.type === 'props' && instance && e.data.props && typeof e.data.props === 'object') {
        instance.props = {...instance.props, ...e.data.props};
        try { render?.(); } catch (err) { report(err); }
      }
      if (e.data.type === 'theme' && (e.data.theme === 'light' || e.data.theme === 'dark')) {
        document.documentElement.dataset.theme = e.data.theme;
        document.documentElement.style.colorScheme = e.data.theme;
      }
      if (e.data.type === 'pins') {
        pins = Array.isArray(e.data.pins) ? e.data.pins : [];
        renderPins();
      }
    });
    send('ready');
  });
})();
