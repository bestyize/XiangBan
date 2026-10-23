import { onBeforeUnmount, ref, type Ref } from 'vue';

/** Pointer events also work in Tauri windows whose native file drop handler owns HTML drag events. */
export function useWorkspaceSort(root: Ref<HTMLElement | null>, paths: () => string[], save: (paths: string[]) => void) {
  const dragging = ref('');
  const target = ref<{ path: string; after: boolean } | null>(null);
  let gesture: { path: string; id: number; x: number; y: number; clientX: number; clientY: number } | null = null;
  let frame = 0;
  let ignoreClickUntil = 0;

  function locate() {
    const el = root.value;
    if (!el || !gesture || !dragging.value) return;
    const viewport = el.closest('.sidebar-scroll')?.getBoundingClientRect() ?? el.getBoundingClientRect();
    const { clientX: x, clientY: y } = gesture;
    target.value = null;
    if (x < viewport.left || x > viewport.right || y < viewport.top || y > viewport.bottom) return;
    // Drop before/after a whole project, including its expanded conversations.
    for (const row of el.querySelectorAll<HTMLElement>('[data-workspace-path]')) {
      const box = row.getBoundingClientRect();
      if (y >= box.top && y <= box.bottom) {
        const path = row.dataset.workspacePath!;
        if (path !== gesture.path) target.value = { path, after: y > box.top + box.height / 2 };
        break;
      }
    }
  }
  function scroll() {
    if (!gesture || !dragging.value) return;
    const el = root.value?.closest<HTMLElement>('.sidebar-scroll');
    if (el) {
      const box = el.getBoundingClientRect(), { clientX: x, clientY: y } = gesture;
      if (x >= box.left && x <= box.right && y >= box.top && y <= box.bottom) {
        const delta = y < box.top + 28 ? -8 : y > box.bottom - 28 ? 8 : 0;
        if (delta) { el.scrollTop += delta; locate(); }
      }
    }
    frame = requestAnimationFrame(scroll);
  }
  function move(event: PointerEvent) {
    if (!gesture || event.pointerId !== gesture.id) return;
    gesture.clientX = event.clientX;
    gesture.clientY = event.clientY;
    if (!dragging.value && Math.hypot(event.clientX - gesture.x, event.clientY - gesture.y) >= 6) {
      dragging.value = gesture.path;
      frame = requestAnimationFrame(scroll);
    }
    if (dragging.value) { event.preventDefault(); locate(); }
  }
  function cleanup() {
    cancelAnimationFrame(frame);
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', drop);
    window.removeEventListener('pointercancel', cancel);
    window.removeEventListener('blur', cancel);
    window.removeEventListener('keydown', keydown);
    if (dragging.value) ignoreClickUntil = Date.now() + 400;
    gesture = null;
    dragging.value = '';
    target.value = null;
  }
  function drop(event: PointerEvent) {
    if (!gesture || event.pointerId !== gesture.id) return;
    gesture.clientX = event.clientX;
    gesture.clientY = event.clientY;
    locate();
    const source = dragging.value, destination = target.value;
    cleanup();
    if (!source || !destination) return;
    const current = paths();
    if (!current.includes(source) || !current.includes(destination.path)) return;
    const next = current.filter(path => path !== source);
    next.splice(next.indexOf(destination.path) + Number(destination.after), 0, source);
    if (next.some((path, index) => path !== current[index])) save(next);
  }
  function cancel() { cleanup(); }
  function keydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && gesture) { event.preventDefault(); cleanup(); }
  }
  function start(path: string, event: PointerEvent) {
    if (!path || event.button !== 0 || event.isPrimary === false) return;
    cleanup();
    ignoreClickUntil = 0;
    gesture = { path, id: event.pointerId, x: event.clientX, y: event.clientY, clientX: event.clientX, clientY: event.clientY };
    window.addEventListener('pointermove', move, { passive: false });
    window.addEventListener('pointerup', drop);
    window.addEventListener('pointercancel', cancel);
    window.addEventListener('blur', cancel);
    window.addEventListener('keydown', keydown);
  }
  function click(event: MouseEvent) {
    if (Date.now() < ignoreClickUntil) { event.preventDefault(); event.stopImmediatePropagation(); }
  }
  onBeforeUnmount(cleanup);
  return { dragging, target, start, click, cancel };
}
