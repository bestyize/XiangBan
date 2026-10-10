export interface ContextAction { key: string; label: string; disabled?: boolean }
interface NativeMenu { popup(): Promise<void>; close(): Promise<void> }
interface NativeMenuApi { Menu: { new: (options: { items: { id: string; text: string; enabled: boolean; action(): void }[] }) => Promise<NativeMenu> } }
function nativeMenuApi() { return (window.__TAURI__ as (typeof window.__TAURI__ & { menu?: NativeMenuApi }))?.menu; }

/** Isolated read-only smoke: validate the actual native API and menu capabilities without a popup. */
export async function probeWallpaperNativeMenu(): Promise<boolean> {
  const api = nativeMenuApi();
  if (!api?.Menu) return false;
  const menu = await api.Menu.new({ items: [{ id: 'wallpaper-probe', text: '预览与设置', enabled: false, action() {} }] });
  await menu.close();
  return true;
}

/** Keep the native menu alive until the next popup or page disposal, including on macOS. */
export function createWallpaperContextMenu() {
  let menu: NativeMenu | null = null;
  let disposed = false;
  return {
    async show(items: ContextAction[], action: (key: string) => void): Promise<boolean> {
      const api = nativeMenuApi();
      if (!api?.Menu || disposed) return false;
      if (menu) await menu.close();
      menu = null;
      const created = await api.Menu.new({ items: items.map(item => ({
        id: `wallpaper-${item.key}`, text: item.label, enabled: !item.disabled,
        action: () => { if (!disposed && !item.disabled) action(item.key); },
      })) });
      if (disposed) { await created.close(); return true; }
      menu = created;
      await created.popup();
      return true;
    },
    dispose() { disposed = true; if (menu) void menu.close().catch(() => {}); menu = null; },
  };
}
