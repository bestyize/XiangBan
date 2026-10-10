// Apply the saved appearance before the app and its styles load to avoid a theme flash.
(() => {
  let preference = 'system'
  try { preference = localStorage.getItem('xb_theme') || 'system' } catch { /* Private browser mode. */ }
  const dark = preference === 'dark' || (preference !== 'light' && matchMedia('(prefers-color-scheme: dark)').matches)
  document.documentElement.dataset.theme = dark ? 'dark' : 'light'
  document.querySelector('meta[name="theme-color"]')?.setAttribute('content', dark ? '#141414' : '#f7f7f7')
})()
