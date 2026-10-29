import { useDialog } from 'naive-ui';

export interface ConfirmOptions { title: string; content: string; positiveText: string; negativeText?: string; danger?: boolean; onPositiveClick: () => unknown }

/** Asks before something is deleted. Uses the app's dialog, or the browser's when no dialog provider is mounted. */
export function useConfirm() {
  let dialog: ReturnType<typeof useDialog> | null = null;
  try { dialog = useDialog(); } catch { /* mounted outside the app shell */ }
  return (options: ConfirmOptions) => {
    const { danger, ...rest } = options;
    if (dialog) { (danger ? dialog.error : dialog.warning)({ negativeText: '取消', ...rest }); return; }
    if (window.confirm(`${options.title}\n${options.content}`)) void options.onPositiveClick();
  };
}
