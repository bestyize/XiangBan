import type { WidgetTheme } from './model.ts';

/** Apply transparent root surfaces at render time, without rewriting the author's source. */
export function customWidgetDocument(html: string, theme: WidgetTheme): string {
  if (theme !== 'transparent') return html;
  return `${html}\n<style data-xiaoban-transparent>html:root,html:root body{background:transparent!important;background-image:none!important;margin:0!important}</style>`;
}
