// All module compilation and execution stays in the opaque preview frame.
import React from 'react';
import * as ReactDOM from 'react-dom';
import {createRoot} from 'react-dom/client';
import * as jsxRuntime from 'react/jsx-runtime';
import {transform} from 'sucrase';
const cache = new Map();
function moduleFile(base, specifier, files) {
  if (/[:\\\0]/.test(specifier) || specifier.startsWith('//')) throw Error(`不支持外部模块：${specifier}；请将依赖放入源包`);
  const parts = specifier.startsWith('/') ? [] : base.split('/').slice(0,-1);
  for (const part of specifier.split('/')) {
    if (part === '..') { if (!parts.length) throw Error('模块路径越界'); parts.pop(); }
    else if (part && part !== '.') parts.push(part);
  }
  const path = parts.join('/');
  const result = [path,...['.js','.jsx','.ts','.tsx','.mjs','/index.js','/index.jsx','/index.ts','/index.tsx'].map(ext=>path+ext)].find(p=>Object.hasOwn(files,p));
  if (!result) throw Error(`缺少本地模块：${specifier}`);
  return result;
}
function load(file, ctx) {
  if (cache.has(file)) return cache.get(file).exports;
  if (cache.size >= 300) throw Error('模块数量超过上限');
  const module = {exports:{}};cache.set(file,module);
  const require = specifier => {
    if (specifier === 'react') return React;
    if (specifier === 'react-dom') return ReactDOM;
    if (specifier === 'react-dom/client') return {createRoot};
    if (specifier === 'react/jsx-runtime') return jsxRuntime;
    const target = moduleFile(file,specifier,{...ctx.files,...ctx.assets});
    if (ctx.assets?.[target]) return ctx.assets[target];
    if (target.endsWith('.json')) return JSON.parse(ctx.files[target]);
    if (target.endsWith('.css')) {
      let style=document.querySelector(`style[data-ocd-module-css="${CSS.escape(target)}"]`);
      if (!style) { style=document.createElement('style');style.dataset.ocdModuleCss=target;style.textContent=ctx.files[target];document.head.append(style); }
      return {};
    }
    return load(target,ctx);
  };
  try {
    const code=transform(ctx.files[file],{transforms:['typescript','jsx','imports'],filePath:file,production:true}).code;
    Function('require','module','exports','React',code)(require,module,module.exports,React);
    return module.exports;
  } catch(e) { cache.delete(file); throw Error(`${file}: ${e.message}`); }
}
window.React ??= React;
window.__OCD_REACT__ = {
  React,
  slot(nodes) {
    function Slot(){const ref=React.useRef(null);React.useLayoutEffect(()=>{ref.current?.replaceChildren(...nodes);},[nodes]);return React.createElement('span',{ref,style:{display:'contents'}});}
    return React.createElement(Slot);
  },
  resolve(base,from,name,ctx) {
    const modules=String(from).trim().split(/\s+/).filter(Boolean);
    if (!modules.length) {
      let value=window;for(const part of name.split('.'))value=value?.[part];
      if (!value) throw Error(`找不到全局组件：${name}`);return value;
    }
    let result;
    for (const specifier of modules) result=load(moduleFile(base,specifier,ctx.files),ctx);
    let value=name?name.split('.').reduce((obj,key)=>obj?.[key],result):result.default;
    if (!value && name === 'default') value=result.default;
    if (!value) throw Error(`模块 ${from} 没有导出 ${name||'default'}`);
    return value;
  },
  mount(container,onError) {
    class Boundary extends React.Component {
      state={error:null};
      static getDerivedStateFromError(error){return {error};}
      componentDidCatch(error){onError(error);}
      render(){return this.state.error?React.createElement('p',{role:'alert'},String(this.state.error.message)):this.props.children;}
    }
    const root=createRoot(container);
    return {
      render(component,props){ReactDOM.flushSync(()=>root.render(React.createElement(Boundary,null,React.createElement(component,props))));},
      unmount(){ReactDOM.flushSync(()=>root.unmount());},
    };
  },
};
