export type WidgetCheck =
  | { action: 'fill'; selector: string; value: string }
  | { action: 'click' | 'expect_visible'; selector: string };

// Serialized into the same opaque-origin iframe used by the desktop widget.
export function installWidgetProbe(checks: WidgetCheck[]) {
  const errors: string[] = [];
  const record = (message: unknown) => {
    const text = String(message || '脚本运行出错').slice(0, 300);
    if (errors.length < 6 && !errors.includes(text)) errors.push(text);
  };
  addEventListener('error', event => {
    if (event instanceof ErrorEvent) record(event.message);
  }, true);
  addEventListener('unhandledrejection', event => record(event.reason));

  function visible(element: Element) {
    const box = element.getBoundingClientRect();
    if (!box.width || !box.height || box.right <= 0 || box.bottom <= 0
      || box.left >= innerWidth || box.top >= innerHeight) return false;
    for (let parent: Element | null = element; parent; parent = parent.parentElement) {
      const style = getComputedStyle(parent);
      if (style.display === 'none' || style.visibility !== 'visible' || Number(style.opacity) === 0) return false;
    }
    return true;
  }

  function visibleContent() {
    if (!document.body) return false;
    const elements = document.body.getElementsByTagName('*');
    for (let index = 0; index < Math.min(elements.length, 2000); index++) {
      const element = elements[index];
      if (element.matches('script, style, template') || !visible(element)) continue;
      if (element.matches('canvas, svg, img, video, input, button, select, textarea')
        || Array.from(element.childNodes).some(node => node.nodeType === Node.TEXT_NODE && node.textContent?.trim())) return true;
      const style = getComputedStyle(element);
      if (style.backgroundImage !== 'none'
        || !['transparent', 'rgba(0, 0, 0, 0)'].includes(style.backgroundColor)) return true;
    }
    return Array.from(document.body.childNodes).some(node =>
      node.nodeType === Node.TEXT_NODE && !!node.textContent?.trim()) && visible(document.body);
  }

  const wait = (ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms));
  addEventListener('DOMContentLoaded', async () => {
    await wait(200);
    const initialVisible = visibleContent();
    const steps: Array<{ ok: boolean; error?: string }> = [];
    for (const check of checks) {
      try {
        const element = document.querySelector(check.selector);
        if (!element || !visible(element)) throw new Error('找不到可见目标');
        if (check.action === 'fill') {
          if (!(element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement)
            || element.matches(':disabled, [readonly], input[type=file]')) throw new Error('目标不是可填写的控件');
          element.value = check.value;
          if (element.value !== check.value) throw new Error('控件拒绝了测试值');
          element.dispatchEvent(new Event('input', { bubbles: true }));
          element.dispatchEvent(new Event('change', { bubbles: true }));
        } else if (check.action === 'click') {
          if (!(element instanceof HTMLElement) || element.matches(':disabled, [aria-disabled=true]')) throw new Error('目标不可点击');
          element.click();
        }
        await wait(100);
        steps.push({ ok: true });
      } catch (error) {
        steps.push({ ok: false, error: String(error).slice(0, 180) });
        break;
      }
    }
    parent.postMessage({
      type: 'xiaoban-widget-preview-report',
      report: { initialVisible, finalVisible: visibleContent(), errors, steps },
    }, '*');
  }, { once: true });
}
