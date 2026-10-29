import {installCustomAppearance} from './custom-appearance.ts';

// This function runs inside the opaque-origin iframe; keep it self-contained.
export function installCustomDrag() {
  let pointer: { id: number; x: number; y: number; target: Element; moved: boolean } | null = null;
  let suppressClick = false;
  const send = (phase: string, event: PointerEvent) => window.parent.postMessage({
    type: 'xiaoban-widget-drag', phase, screenX: event.screenX, screenY: event.screenY,
    x: event.clientX / Math.max(1, window.innerWidth), y: event.clientY / Math.max(1, window.innerHeight),
  }, '*');
  document.addEventListener('pointerdown', event => {
    if (event.button !== 0 || pointer || !(event.target instanceof Element)
      || event.target.closest('button,input,textarea,select,option,a,label,form,[contenteditable]:not([contenteditable="false"]),[role="button"],[data-no-drag]')) return;
    suppressClick = false;
    pointer = { id: event.pointerId, x: event.screenX, y: event.screenY, target: event.target, moved: false };
    send('start', event);
  }, true);
  document.addEventListener('pointermove', event => {
    if (!pointer || event.pointerId !== pointer.id) return;
    if (!pointer.moved && Math.hypot(event.screenX - pointer.x, event.screenY - pointer.y) <= 5) return;
    if (!pointer.moved) pointer.target.setPointerCapture?.(event.pointerId);
    pointer.moved = true;
    suppressClick = true;
    event.preventDefault();
    send('move', event);
  }, true);
  const finish = (event: PointerEvent) => {
    if (!pointer || event.pointerId !== pointer.id) return;
    send('end', event);
    pointer = null;
    setTimeout(() => { suppressClick = false; }, 0);
  };
  document.addEventListener('pointerup', finish, true);
  document.addEventListener('pointercancel', finish, true);
  document.addEventListener('lostpointercapture', finish, true);
  document.addEventListener('click', event => {
    if (suppressClick) { event.preventDefault(); event.stopImmediatePropagation(); }
  }, true);
  document.addEventListener('contextmenu', event => {
    event.preventDefault();
    event.stopImmediatePropagation();
    window.parent.postMessage({ type: 'xiaoban-widget-settings' }, '*');
  }, true);
  document.addEventListener('dragstart', event => { if (pointer) event.preventDefault(); }, true);
}

export function customWidgetDocument(html: string, draggable: boolean): string {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  const style = doc.createElement('style');
  style.textContent = 'html,body{margin:0!important;padding:0!important;overflow:hidden!important;width:100%!important;height:100%!important;background:transparent!important}';
  doc.head.prepend(style);
  const appearance = doc.createElement('script');
  appearance.textContent = `(${installCustomAppearance.toString()})();`;
  doc.head.prepend(appearance);
  if (draggable) {
    const script = doc.createElement('script');
    script.textContent = `(${installCustomDrag.toString()})();`;
    doc.head.prepend(script);
  }
  return '<!DOCTYPE html>' + doc.documentElement.outerHTML;
}

export interface CustomDragMessage {
  type: 'xiaoban-widget-drag';
  phase: 'start' | 'move' | 'end';
  screenX: number;
  screenY: number;
  x: number;
  y: number;
}

export function isCustomDragMessage(value: unknown): value is CustomDragMessage {
  if (!value || typeof value !== 'object') return false;
  const data = value as CustomDragMessage;
  return data.type === 'xiaoban-widget-drag' && ['start', 'move', 'end'].includes(data.phase)
    && [data.screenX, data.screenY, data.x, data.y].every(v => typeof v === 'number' && Number.isFinite(v))
    && (data.phase !== 'start' || data.x >= 0 && data.x <= 1 && data.y >= 0 && data.y <= 1);
}

