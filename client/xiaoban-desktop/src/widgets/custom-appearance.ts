// Runs inside the sandbox, where both user stylesheets and inline styles are accessible.
// Keep this function self-contained so it can be injected without granting host access.
export function installCustomAppearance() {
  let layoutScale=1,layoutWidth=0,layoutHeight=0;
  function layout(){
    if(!document.body)return;
    // CSS zoom inside the iframe lays out text at the actual output resolution.
    // Keep the logical viewport unchanged for existing fixed-size widget packs.
    document.body.style.setProperty('zoom',String(layoutScale),'important');
    if(layoutWidth)document.body.style.setProperty('width',`${layoutWidth}px`,'important');
    if(layoutHeight)document.body.style.setProperty('height',`${layoutHeight}px`,'important');
  }
  window.addEventListener('message',event=>{
    if(event.source!==window.parent||event.data?.type!=='xiaoban-widget-layout')return;
    const scale=event.data.scale;
    if(typeof scale!=='number'||!Number.isFinite(scale)||scale<0.1||scale>10)return;
    const {width,height}=event.data;
    if(typeof width!=='number'||typeof height!=='number'||!Number.isFinite(width)||!Number.isFinite(height)||width<=0||height<=0||width>4096||height>4096)return;
    layoutScale=scale;layoutWidth=width;layoutHeight=height;layout();
  });
  function soften(style: CSSStyleDeclaration) {
    const shadow = style.getPropertyValue('box-shadow');
    if (shadow && !['none', 'inherit', 'initial', 'unset', 'revert'].includes(shadow.trim())) {
      // Split only between shadows, keeping commas inside rgba() and var() intact.
      const parts: string[] = [];
      let depth = 0, start = 0;
      for (let i = 0; i < shadow.length; i++) {
        if (shadow[i] === '(') depth++;
        else if (shadow[i] === ')') depth--;
        else if (shadow[i] === ',' && depth === 0) { parts.push(shadow.slice(start, i)); start = i + 1; }
      }
      parts.push(shadow.slice(start));
      const inset = parts.filter(part => /\binset\b/i.test(part));
      if (inset.length < parts.length) {
        style.setProperty('box-shadow', ['0 2px 5px -3px rgba(0,0,0,0.12)', ...inset].join(','), style.getPropertyPriority('box-shadow'));
      }
    }
    for (const property of ['filter', '-webkit-filter']) {
      let filter = style.getPropertyValue(property);
      // Replace each drop-shadow() as a whole, including nested color functions.
      const pattern = /drop-shadow\(/ig;
      let match: RegExpExecArray | null;
      while ((match = pattern.exec(filter))) {
        let depth = 1, end = pattern.lastIndex;
        while (end < filter.length && depth) {
          if (filter[end] === '(') depth++;
          else if (filter[end] === ')') depth--;
          end++;
        }
        if (depth) break;
        const replacement = 'drop-shadow(0 1px 2px rgba(0,0,0,0.08))';
        filter = filter.slice(0, match.index) + replacement + filter.slice(end);
        pattern.lastIndex = match.index + replacement.length;
      }
      if (filter !== style.getPropertyValue(property)) style.setProperty(property, filter, style.getPropertyPriority(property));
    }
  }
  function visit(rules: CSSRuleList) {
    for (const rule of Array.from(rules)) {
      if ('style' in rule) soften((rule as CSSStyleRule).style);
      if ('cssRules' in rule) visit((rule as CSSGroupingRule).cssRules);
    }
  }
  function apply() {
    layout();
    for (const sheet of Array.from(document.styleSheets)) {
      try { visit(sheet.cssRules); } catch { /* Cross-origin stylesheets remain isolated. */ }
    }
    for (const element of document.querySelectorAll<HTMLElement | SVGElement>('[style]')) soften(element.style);
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', apply, { once: true });
  else apply();
}
