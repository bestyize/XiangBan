export function isWeChat(): boolean {
  return /MicroMessenger/i.test(navigator.userAgent)
}

/** Share-card attribution from the QR link (?from=card&inv=…), kept until the session has been created. */
const REF_KEY = 'xb_ref'

export function captureReferral(): { ref?: string; from?: string } {
  try {
    const params = new URLSearchParams(location.search)
    const inv = params.get('inv')
    const from = params.get('from')
    if (inv || from) sessionStorage.setItem(REF_KEY, JSON.stringify({ ref: inv ?? undefined, from: from ?? undefined }))
    return JSON.parse(sessionStorage.getItem(REF_KEY) ?? '{}')
  } catch {
    return {}
  }
}

export function publicUrl(): string {
  return (import.meta.env.VITE_PUBLIC_URL as string | undefined) ?? location.origin
}
