import { nextTick, onBeforeUnmount, onMounted, type Ref } from 'vue';

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

/** 叠在一起的弹窗（抽屉上再开一个确认框）：只有最上面的那个响应键盘。 */
const stack: symbol[] = [];

/**
 * 弹窗和抽屉共用的键盘与焦点管理。
 * 打开时把焦点移进去；Tab 只在里面循环；Esc 只关最上面一层（不传 close 则 Esc 不关，用于必须点确认的窗口）；
 * 关闭后把焦点还给打开它的那个按钮。
 */
export function useModal(root: Ref<HTMLElement | null>, close?: () => void) {
  const me = Symbol('modal');
  let opener: HTMLElement | null = null;

  const focusables = () =>
    [...(root.value?.querySelectorAll<HTMLElement>(FOCUSABLE) ?? [])].filter((el) => el.offsetParent !== null || el === document.activeElement);

  function onKey(event: KeyboardEvent) {
    if (stack[stack.length - 1] !== me) return;
    if (event.key === 'Escape' && close) {
      event.preventDefault();
      event.stopPropagation();
      close();
      return;
    }
    if (event.key !== 'Tab') return;
    const items = focusables();
    if (items.length === 0) {
      event.preventDefault();
      root.value?.focus();
      return;
    }
    const active = document.activeElement;
    const outside = !root.value?.contains(active);
    if (event.shiftKey && (active === items[0] || outside)) {
      event.preventDefault();
      items[items.length - 1].focus();
    } else if (!event.shiftKey && (active === items[items.length - 1] || outside)) {
      event.preventDefault();
      items[0].focus();
    }
  }

  onMounted(async () => {
    opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    stack.push(me);
    document.addEventListener('keydown', onKey, true);
    await nextTick();
    const el = root.value;
    if (!el) return;
    el.tabIndex = -1;
    (el.querySelector<HTMLElement>('[data-autofocus], [autofocus]') ?? focusables()[0] ?? el).focus({ preventScroll: true });
  });

  onBeforeUnmount(() => {
    document.removeEventListener('keydown', onKey, true);
    const index = stack.indexOf(me);
    if (index >= 0) stack.splice(index, 1);
    if (opener && document.contains(opener)) opener.focus({ preventScroll: true });
  });
}
