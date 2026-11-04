import { desktop, invoke } from '../bridge.ts';
import { customWidgetDocument, customWidgetShell } from './custom-drag.ts';
import type { WidgetCheck } from './preview-probe.ts';

declare global {
  interface Window {
    __XIAOBAN_WIDGET_PREVIEW__?: { html: string; checks: WidgetCheck[] };
  }
}

export function mountWidgetPreview() {
  const draft = window.__XIAOBAN_WIDGET_PREVIEW__;
  delete window.__XIAOBAN_WIDGET_PREVIEW__;
  if (!desktop || !draft) return;

  const html = customWidgetDocument(draft.html, true, draft.checks);
  const frame = document.createElement('iframe');
  frame.setAttribute('sandbox', 'allow-scripts');
  frame.title = '组件运行预览';
  frame.scrolling = 'no';
  frame.src = customWidgetShell(desktop);
  frame.style.cssText = 'display:block;border:0;width:100vw;height:100vh';
  document.body.style.cssText = 'margin:0;overflow:hidden;background:#727680';
  let initialized = false;
  let reported = false;
  window.addEventListener('message', event => {
    if (event.source !== frame.contentWindow || event.origin !== 'null') return;
    if (!initialized && event.data?.type === 'xiaoban-widget-shell-ready') {
      initialized = true;
      frame.contentWindow?.postMessage({ type: 'xiaoban-widget-init', html }, '*');
    } else if (initialized && !reported && event.data?.type === 'xiaoban-widget-preview-report') {
      reported = true;
      const { initialVisible, finalVisible, errors, steps } = event.data.report ?? {};
      void invoke('widget_preview_report', {
        report: {
          initialVisible: initialVisible === true, finalVisible: finalVisible === true,
          errors: Array.isArray(errors) ? errors.slice(0, 6).map(error => String(error).slice(0, 300)) : [],
          steps: Array.isArray(steps) ? steps.slice(0, 8).map(step => ({
            ok: step?.ok === true,
            error: typeof step?.error === 'string' ? step.error.slice(0, 180) : undefined,
          })) : [],
        },
      }).catch(console.error);
    }
  });
  document.body.append(frame);
}
