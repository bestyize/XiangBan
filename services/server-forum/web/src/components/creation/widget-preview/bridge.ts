// The copied renderer is always in preview mode; there is no desktop bridge in the web app.
export const desktop = false
export function invoke<T = unknown>(_command: string, _args?: unknown): Promise<T> { return Promise.reject(new Error('网页预览不可使用桌面功能')) }
