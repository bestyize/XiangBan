export interface Settings {
  mode: 'floating' | 'wallpaper'
  expression: string
  size: number
  theme: 'night' | 'dawn'
  paused: boolean
  fps: 30 | 60
}

export interface CursorSample {
  x: number; y: number; moved: boolean; pressed: boolean; released: boolean; desktop: boolean
}

declare global {
  interface Window {
    __TAURI__?: {
      core: { invoke: <T>(command: string, args?: Record<string, unknown>) => Promise<T> }
      event: { listen: <T>(event: string, callback: (event: { payload: T }) => void) => Promise<() => void> }
    }
  }
}

export const desktop = Boolean(window.__TAURI__)
export const defaults: Settings = { mode: 'floating', expression: 'idle', size: 260, theme: 'night', paused: false, fps: 60 }
export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!window.__TAURI__) return Promise.reject(new Error('请通过桌面程序运行这个功能'))
  return window.__TAURI__.core.invoke<T>(command, args)
}
export async function listen<T>(event: string, callback: (payload: T) => void): Promise<() => void> {
  return window.__TAURI__?.event.listen<T>(event, ({ payload }) => callback(payload)) ?? (() => {})
}
