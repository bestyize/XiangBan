/* Independent DC HTML compatibility runtime. Runs only in an opaque-origin sandbox. */
(() => {
  'use strict';
  const send = (type, data = {}) => parent.postMessage({channel:'open-claude-design', token:new URLSearchParams(location.search).get('token'),type,...data}, '*');
  const report = e => send('error', {message:String(e?.message || e).slice(0,1000)});
  addEventListener('error', e => report(e.error || e.message));
  addEventListener('unhandledrejection', e => report(e.reason));
  let editing = true, commenting = false, pins = [], DCLogicBase;
  let requestRender, renderContext;
  const components = new Map(), reactComponents = new Map();
  const camel = name => name.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
  const parseDocument = source => new DOMParser().parseFromString(source.replace(/<(dc-import|x-import)(\s(?:[^>"']|"[^"]*"|'[^']*')*)?\/\s*>/gi, '<$1$2></$1>'), 'text/html');
  const bundleEl = () => document.getElementById('__ocd_bundle__');
  const readBundle = () => {
    try {
      const raw = bundleEl()?.textContent;
      const parsed = raw ? JSON.parse(raw) : {};
      return {
        base: parsed.base || '',
        files: {...(parsed.files || {}), ...(window.__OCD_FILES__ || {})},
        tokens: {...(parsed.tokens || {}), ...(window.__OCD_TOKENS__ || {})},
        assets: parsed.assets || {},
      };
    } catch { return {base:'', files:window.__OCD_FILES__ || {}, tokens:window.__OCD_TOKENS__ || {}}; }
  };
  const resolvePath = (base, rel) => {
    const clean = String(rel || '').trim();
    if (!clean || /[:\\\x00]/.test(clean) || clean.startsWith('//')) return '';
    const parts = clean.startsWith('/') ? [] : base.split('/').slice(0, -1);
    for (const p of clean.split(/[?#]/)[0].split('/')) {
      if (p === '..') { if (!parts.length) return ''; parts.pop(); }
      else if (p && p !== '.') parts.push(p);
    }
    return parts.join('/');
  };
  const evaluate = (text, scope) => Function('scope', `with(scope){return (${text})}`)(scope);
  // Split only at a balanced closing delimiter, so object literals, callbacks,
  // quoted braces and multiline expressions survive the authoring round-trip.
  function expressionParts(text) {
    const parts = []; let pos = 0;
    while (pos < text.length) {
      const start = text.indexOf('{{', pos);
      if (start < 0) { parts.push({text:text.slice(pos)}); break; }
      if (start > pos) parts.push({text:text.slice(pos, start)});
      let depth = 0, quote = '', escape = false, end = -1;
      for (let i = start + 2; i < text.length; i++) {
        const c = text[i];
        if (quote) { if (escape) escape = false; else if (c === '\\') escape = true; else if (c === quote) quote = ''; continue; }
        if (c === '"' || c === "'" || c === '`') { quote = c; continue; }
        if (c === '}' && !depth && text[i + 1] === '}') { end = i; break; }
        if ('{[('.includes(c)) depth++;
        if ('}])'.includes(c)) depth--;
      }
      if (end < 0) { parts.push({text:text.slice(start)}); break; }
      parts.push({code:text.slice(start + 2, end)}); pos = end + 2;
    }
    return parts;
  }
  const expr = (text, scope) => {
    if (!text.includes('{{')) return text;
    const parts = expressionParts(text), expressions = parts.filter(p => p.code !== undefined);
    if (expressions.length === 1 && parts.every(p => p.code !== undefined || !p.text.trim())) return evaluate(expressions[0].code, scope);
    return parts.map(p => p.code === undefined ? p.text : String(evaluate(p.code, scope) ?? '')).join('');
  };
  function assetUrl(url, ctx) {
    const raw = String(url ?? '');
    const path = resolvePath(ctx.file, raw);
    const asset = ctx.assets?.[path];
    return asset ? asset + (raw.includes('#') ? '#' + raw.split('#').slice(1).join('#') : '') : raw;
  }
  function assetCss(css, ctx) {
    return String(css).replace(/url\(\s*(['"]?)([^)'"]+)\1\s*\)/gi, (_, q, path) => `url("${assetUrl(path.trim(), ctx)}")`);
  }
  function nodeIds(root) {
    const used = new Set([...root.querySelectorAll('[data-ocd-node]')].map(el => el.getAttribute('data-ocd-node'))); let n = 0;
    root.querySelectorAll('*').forEach(el => { if (!el.hasAttribute('data-ocd-node')) { while (used.has('n' + n)) n++; el.setAttribute('data-ocd-node', 'n' + n); used.add('n' + n++); } });
  }
  function logic(script, props) {
    if (!script?.textContent.trim()) return new DCLogicBase(props);
    const Comp = Function('DCLogic', 'StreamableLogic', 'React', script.textContent + '\n;return typeof Component!=="undefined"?Component:DCLogic;')(DCLogicBase, DCLogicBase, window.__OCD_REACT__?.React);
    return new Comp(props);
  }
  const bool = v => Boolean(v);
  const pxProps = new Set(['width','height','min-width','min-height','max-width','max-height','font-size','border-radius','border-width','padding','padding-top','padding-right','padding-bottom','padding-left','margin','margin-top','margin-right','margin-bottom','margin-left','gap','top','left','right','bottom']);
  const formatStyle = v => {
    if (Array.isArray(v)) return v.map(formatStyle).filter(Boolean).join(';');
    if (!v || typeof v !== 'object') return String(v ?? '');
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
    const rawSrc = node.getAttribute('src') || node.getAttribute('file');
    const name = node.getAttribute('name') || node.getAttribute('component') || '';
    const rel = rawSrc || (name.endsWith('.html') ? name : name + '.dc.html');
    const resolved = resolvePath(ctx.file, rel);
    const file = Object.hasOwn(ctx.files, resolved) ? resolved : (!rawSrc && Object.hasOwn(ctx.files, rel) ? rel : '');
    const source = ctx.files[file];
    if (!source) throw Error(`dc-import 缺少组件源文件：${rawSrc || name || '未指定组件'}`);
    if (ctx.depth >= 16) throw Error(`dc-import 组件嵌套超过上限：${file}`);
    const doc = parseDocument(source), compRoot = doc.querySelector('x-dc') || doc.body;
    nodeIds(compRoot);
    const script = doc.querySelector('script[data-dc-script]');
    const config = JSON.parse(script?.getAttribute('data-props') || '{}');
    const compProps = Object.fromEntries(Object.entries(config).filter(([k]) => !k.startsWith('$')).map(([k, v]) => [k, v?.default]));
    for (const attr of [...node.attributes]) {
      if (['src','file','name','component','data-ocd-node','hint-size','style'].includes(attr.name)) continue;
      const value = expr(attr.value, scope), key = camel(attr.name.replace(/^sc-camel-/, ''));
      if (key === 'dcProps') { if (value && typeof value === 'object') Object.assign(compProps, value); }
      else compProps[key] = value;
    }
    const key = ctx.key + '/' + node.getAttribute('data-ocd-node') + ':' + file;
    let entry = components.get(key);
    if (!entry) { entry = {instance:logic(script, compProps), mounted:false}; components.set(key, entry); }
    entry.seen = true;
    const inst = entry.instance;
    if (entry.mounted && !entry.previousProps && Object.keys({...inst.props,...compProps}).some(k => inst.props[k] !== compProps[k])) entry.previousProps = inst.props;
    inst.props = compProps;
    // Render slotted nodes in the caller's scope; move them without cloning away
    // their event listeners. The component scope must not re-evaluate them.
    const slotHolder = document.createElement('div');
    slotHolder.append(...[...node.childNodes].map(n => n.cloneNode(true)));
    expandChildren(slotHolder, scope, {...ctx,key:ctx.key + '/slot:' + node.getAttribute('data-ocd-node')});
    compProps.children = [...slotHolder.childNodes];
    const compScope = {$tokens:ctx.tokens,...compProps,...(inst.renderVals?.() || {})};
    const wrapper = document.createElement('div');
    wrapper.append(...[...compRoot.childNodes].map(n => n.cloneNode(true)));
    for (const head of doc.head.querySelectorAll('style,link[rel="stylesheet"]')) wrapper.prepend(head.cloneNode(true));
    expandChildren(wrapper, compScope, {...ctx,file,depth:ctx.depth + 1,key,slots:compProps.children});
    for (const el of wrapper.querySelectorAll('[data-ocd-node]')) el.setAttribute('data-ocd-node','import:' + key + ':' + el.getAttribute('data-ocd-node'));
    for (const el of wrapper.children) el.setAttribute('data-ocd-imported','true');
    const host = document.createElement('div');
    host.style.cssText = 'display:contents;' + formatStyle(expr(node.getAttribute('style') || '', scope));
    host.setAttribute('data-ocd-imported','true');
    host.setAttribute('data-ocd-node',node.getAttribute('data-ocd-node'));
    host.append(...wrapper.childNodes);
    node.replaceWith(host);
  }
  function expandReact(node,scope,ctx) {
    const bridge=window.__OCD_REACT__;
    if (!bridge) throw Error('React 运行时未打包');
    const name=String(expr(node.getAttribute('component-from-global-scope') || node.getAttribute('component') || node.getAttribute('name') || '',scope));
    const from=node.getAttribute('from') || node.getAttribute('src') || node.getAttribute('import') || '';
    if (from.includes('{{')) throw Error('x-import from 必须是静态模块路径');
    const component=bridge.resolve(ctx.file,from,name,ctx);
    const key=ctx.key + '/react:' + node.getAttribute('data-ocd-node');
    let entry=reactComponents.get(key);
    if (!entry) {
      const host=document.createElement('div');host.style.display='contents';host.setAttribute('data-ocd-imported','true');host.setAttribute('data-ocd-node',node.getAttribute('data-ocd-node'));
      entry={host,root:bridge.mount(host,report)};reactComponents.set(key,entry);
    }
    entry.seen=true;
    const props={};
    for (const attr of [...node.attributes]) {
      if (['name','component','component-from-global-scope','from','src','import','hint-size','data-ocd-node'].includes(attr.name)) continue;
      const value=expr(attr.value,scope);
      const key=attr.name.startsWith('data-') || attr.name.startsWith('aria-') ? attr.name : camel(attr.name.replace(/^sc-camel-/,''));
      if (key==='dcProps') { if(value&&typeof value==='object')Object.assign(props,value); }
      else {
        const event=['onClick','onDoubleClick','onChange','onInput','onSubmit','onFocus','onBlur','onKeyDown','onKeyUp','onMouseEnter','onMouseLeave','onMouseDown','onMouseUp','onMouseMove','onPointerDown','onPointerUp','onPointerMove','onScroll','onWheel','onTouchStart','onTouchMove','onTouchEnd'].find(event=>event.toLowerCase()===key);
        if(key==='style'&&typeof value==='string'){const el=document.createElement('div');el.style.cssText=assetCss(value,ctx);props.style=Object.fromEntries([...el.style].map(k=>[k.startsWith('--')?k:camel(k),el.style.getPropertyValue(k)]));}
        else props[event||key]=value;
      }
    }
    if (node.childNodes.length) {
      const holder=document.createElement('div');holder.append(...[...node.childNodes].map(n=>n.cloneNode(true)));
      expandChildren(holder,scope,{...ctx,key:key+'/children'});
      props.children=bridge.slot([...holder.childNodes]);
    }
    entry.component=component;entry.props=props;
    node.replaceWith(entry.host);
  }
  function expand(node, scope, ctx) {
    if (node.nodeType === Node.TEXT_NODE) { if(node.parentElement?.closest('[data-ocd-literal="true"]'))return; const value = expr(node.textContent || '', scope); if (Array.isArray(value) && value.every(n => n instanceof Node)) node.replaceWith(...value); else node.textContent = String(value ?? ''); return; }
    if (!(node instanceof Element)) return;
    if (node.localName === 'x-import') { expandReact(node,scope,ctx); return; }
    if (node.localName === 'slot' || node.localName === 'dc-slot') {
      const name = node.getAttribute('name');
      const nodes = (ctx.slots || []).filter(n => name ? n instanceof Element && n.getAttribute('slot') === name : !(n instanceof Element && n.hasAttribute('slot')));
      if (nodes.length) node.replaceWith(...nodes); else { expandChildren(node, scope, ctx); node.replaceWith(...node.childNodes); }
      return;
    }
    if (node.localName === 'helmet' || node.localName === 'sc-helmet') {
      expandChildren(node, scope, ctx);
      for (const child of [...node.children]) { child.setAttribute('data-ocd-helmet',''); document.head.append(child); }
      node.remove(); return;
    }
    if (node.localName === 'style') { node.textContent = assetCss(expr(node.textContent || '', scope),ctx); return; }
    if (node.localName === 'script') return;
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
        expandChildren(holder, childScope, {...ctx,key:ctx.key + '/for:' + node.getAttribute('data-ocd-node') + ':' + i});
        fragment.append(...holder.childNodes);
      }
      node.replaceWith(fragment); return;
    }
    if (node.localName === 'dc-import') { expandImport(node, scope, ctx); return; }
    // Official standalone exports encode case-sensitive DOM attributes this way.
    // Keep component props above intact; restore DOM names (including SVG viewBox).
    for (const attr of [...node.attributes]) if (attr.name.startsWith('sc-camel-')) {
      node.removeAttribute(attr.name);
      node.setAttribute(camel(attr.name.slice(9)), attr.value);
    }
    for (const attr of [...node.attributes]) {
      if (attr.name.toLowerCase().startsWith('on')) {
        node.removeAttribute(attr.name);
        const handler = expr(attr.value, scope);
        if (typeof handler === 'function') {
          const eventName = attr.name.toLowerCase();
          const capture = eventName.endsWith('capture'), raw = eventName.slice(2).replace(/capture$/, '');
          const event = raw === 'doubleclick' ? 'dblclick' : raw === 'change' && (node.localName === 'textarea' || node.localName === 'input' && !['checkbox','radio','file'].includes(node.type)) ? 'input' : raw;
          node.addEventListener(event, e => { if (!editing) try { handler(e); } catch (error) { report(error); } }, capture);
        }
        continue;
      }
      if (attr.value.includes('{{')) {
        const v = expr(attr.value, scope);
        if (['checked', 'disabled', 'selected', 'hidden', 'readonly', 'multiple', 'required', 'autofocus', 'open', 'muted', 'controls', 'loop'].includes(attr.name)) { if (bool(v)) node.setAttribute(attr.name, ''); else node.removeAttribute(attr.name); }
        else if (attr.name === 'style' || attr.name.startsWith('style-')) node.setAttribute(attr.name, formatStyle(v));
        else if (attr.name === 'class') node.setAttribute('class', formatClass(v));
        else node.setAttribute(attr.name, String(v ?? ''));
      }
    }
    for (const attr of [...node.attributes]) {
      if (['src','poster','href'].includes(attr.name) && !(node.localName === 'a' && attr.name === 'href')) node.setAttribute(attr.name,assetUrl(attr.value,ctx));
      if (attr.name === 'style') node.setAttribute('style',assetCss(attr.value,ctx));
      if (attr.name === 'srcset' && !attr.value.startsWith('data:')) node.setAttribute('srcset',attr.value.split(',').map(part => { const [url,...size] = part.trim().split(/\s+/); return [assetUrl(url,ctx),...size].join(' '); }).join(', '));
    }
    if (node.localName === 'textarea' && node.hasAttribute('value')) node.value = node.getAttribute('value');
    if(node.hasAttribute('data-ocd-style'))node.style.cssText+=';'+node.getAttribute('data-ocd-style');
    if(node.getAttribute('data-ocd-hidden')==='true')node.style.setProperty('display','none','important');
    const pseudo = [...node.attributes].filter(a => a.name.startsWith('style-'));
    if (pseudo.length) {
      const key = 'ocd-pseudo-' + (++renderContext.pseudo);
      node.classList.add(key);
      const style = document.createElement('style'); style.setAttribute('data-ocd-helmet','');
      style.textContent = pseudo.filter(a => /^[a-z-]+$/.test(a.name.slice(6))).map(a => `.${key}:${a.name.slice(6)}{${assetCss(a.value,ctx)}}`).join('');
      document.head.append(style);
    }
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
    let instance, render, mounted = false, rendering = false, pending = false;
    nodeIds(root || document.body);
    window.__OCD_EDITOR_SOURCE__=(root||document.body).cloneNode(true);
    renderContext = {file:bundle.base,files:bundle.files,assets:bundle.assets,tokens:bundle.tokens,depth:0,key:'root',pseudo:0};
    // Static HTML also resolves local media and CSS without gaining filesystem access.
    for (const el of document.querySelectorAll('style,[src],[poster]')) {
      if (el.localName === 'style') el.textContent = assetCss(el.textContent,renderContext);
      else for (const attr of ['src','poster']) if(el.hasAttribute(attr)) el.setAttribute(attr,assetUrl(el.getAttribute(attr),renderContext));
    }
    if (root) {
      try {
        const template = root.cloneNode(true), script = document.querySelector('script[data-dc-script]');
        const config = JSON.parse(script?.getAttribute('data-props') || '{}');
        const props = Object.fromEntries(Object.entries(config).filter(([k]) => !k.startsWith('$')).map(([k,v]) => [k,v?.default]));
        class DCLogic {
          constructor(p) { this.props = p; this.state = {}; this.tokens = bundle.tokens; this.__callbacks = []; }
          setState(value, callback) {
            if(this.__disposed)return;
            const patch = typeof value === 'function' ? value(this.state,this.props) : value;
            if (patch == null) return;
            this.__previousState ??= this.state;
            this.state = {...this.state,...patch};
            if (typeof callback === 'function') this.__callbacks.push(callback);
            requestRender?.();
          }
          forceUpdate(callback) { if (typeof callback === 'function') this.__callbacks.push(callback); requestRender?.(); }
        }
        DCLogicBase = DCLogic;
        instance = logic(script,props);
        render = () => {
          if (rendering) { pending = true; return; }
          rendering = true;
          try {
            const scrollPositions=[...root.querySelectorAll('[data-ocd-node]')].filter(el=>el.scrollTop||el.scrollLeft).map(el=>[el.getAttribute('data-ocd-node'),el.scrollTop,el.scrollLeft]);
            const selection = document.activeElement?.getAttribute('data-ocd-node');
            const rangeStart = document.activeElement?.selectionStart, rangeEnd = document.activeElement?.selectionEnd;
            const clone = template.cloneNode(true), scope = {$tokens:bundle.tokens,...instance.props,...(instance.renderVals?.() || {})};
            for (const entry of components.values()) entry.seen = false;
            for (const entry of reactComponents.values()) entry.seen = false;
            document.querySelectorAll('[data-ocd-helmet]').forEach(n => n.remove());
            renderContext.pseudo = 0;
            expandChildren(clone,scope,renderContext);
            root.replaceChildren(...clone.childNodes);
            for(const [key,entry] of reactComponents){if(!entry.seen){entry.root.unmount();reactComponents.delete(key);}else entry.root.render(entry.component,entry.props);}
            if (selection) { const active = [...root.querySelectorAll('[data-ocd-node]')].find(n => n.getAttribute('data-ocd-node') === selection); active?.focus(); if (typeof active?.setSelectionRange === 'function') try { active.setSelectionRange(rangeStart,rangeEnd); } catch {} }
            for(const [id,top,left] of scrollPositions){const el=[...root.querySelectorAll('[data-ocd-node]')].find(el=>el.getAttribute('data-ocd-node')===id);if(el){el.scrollTop=top;el.scrollLeft=left;}}
            for (const [key,entry] of components) {
              if (!entry.seen) { entry.instance.__disposed=true;entry.instance.componentWillUnmount?.(); components.delete(key); continue; }
              if (!entry.mounted) { entry.mounted = true; entry.instance.componentDidMount?.(); }
              else if (entry.previousProps || entry.instance.__previousState) {
                const previousProps = entry.previousProps || entry.instance.props, previousState = entry.instance.__previousState || entry.instance.state;
                entry.previousProps = undefined; entry.instance.__previousState = undefined;
                entry.instance.componentDidUpdate?.(previousProps,previousState);
              }
              entry.instance.__callbacks.splice(0).forEach(fn => fn.call(entry.instance));
            }
            const oldState=instance.__previousState,oldProps=instance.__previousProps;instance.__previousState=undefined;instance.__previousProps=undefined;
            if(mounted&&(oldState||oldProps))instance.componentDidUpdate?.(oldProps||instance.props,oldState||instance.state);
            instance.__callbacks.splice(0).forEach(fn => fn.call(instance));
            renderPins(); dispatchEvent(new Event('ocd-rendered'));
          } finally { rendering = false; if (pending) { pending = false; queueMicrotask(() => { try { render(); } catch(e) { report(e); } }); } }
        };
        requestRender = render;
        render(); mounted = true; instance.componentDidMount?.();
        addEventListener('unload', () => { for (const entry of components.values()) entry.instance.componentWillUnmount?.(); if (mounted) instance.componentWillUnmount?.(); components.clear(); for(const entry of reactComponents.values())entry.root.unmount();reactComponents.clear(); requestRender = undefined; });
      } catch (e) { report(e); }
    }
    for(const event of ['click','input','change','submit','keydown','keyup','pointerdown','pointerup'])document.addEventListener(event,e=>{if(editing&&e.target?.closest?.('[data-ocd-imported]')&&e.type!=='click'){e.preventDefault();e.stopPropagation();}},true);
    for(const type of ['gesturestart','gesturechange'])document.addEventListener(type,e=>{e.preventDefault();send('canvas-gesture',{start:type==='gesturestart',scale:e.scale,x:e.clientX,y:e.clientY});},{passive:false});
    // Opaque iframes do not bubble input to the canvas host.
    document.addEventListener('wheel', e => {
      if (!editing && !e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      send('canvas-wheel',{dx:e.deltaX,dy:e.deltaY,deltaMode:e.deltaMode,ctrl:e.ctrlKey,meta:e.metaKey,shift:e.shiftKey,x:e.clientX,y:e.clientY});
    },{passive:false});
    for (const type of ['keydown','keyup']) document.addEventListener(type,e=>{
      if(window.__OCD_CANVAS_EDITOR__||!editing||e.target?.closest?.('input,textarea,select,[contenteditable]'))return;
      if([' ','+','=','-','0','1'].includes(e.key)){e.preventDefault();send('canvas-key',{key:e.key,down:type==='keydown'});}
    });
    let pan=null;
    document.addEventListener('pointerdown',e=>{if(e.button===1){e.preventDefault();pan={id:e.pointerId,x:e.clientX,y:e.clientY};e.target?.setPointerCapture?.(e.pointerId);}},true);
    document.addEventListener('pointermove',e=>{if(pan&&pan.id===e.pointerId){send('canvas-pan',{dx:e.clientX-pan.x,dy:e.clientY-pan.y});pan.x=e.clientX;pan.y=e.clientY;}});
    for(const type of ['pointerup','pointercancel'])document.addEventListener(type,()=>pan=null);
    document.addEventListener('click', e => {
      const target = e.target.closest?.('[data-ocd-node]');
      if (editing) {
        if(window.__OCD_CANVAS_EDITOR__)return;
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
        instance.__previousProps=instance.props;instance.props = {...instance.props, ...e.data.props};
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
