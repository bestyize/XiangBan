// Installed at document start in every WebView and child frame.
// Keep native editing actions in text fields. Cancel the browser menu elsewhere
// without stopping app-defined contextmenu handlers.
window.addEventListener('contextmenu', function (event) {
  const target = event.composedPath()[0];
  if (target instanceof window.Element) {
    const editor = target.closest('input, textarea, [contenteditable]');
    if (editor) {
      const textControl = !editor.disabled && (editor.tagName === 'TEXTAREA'
        || (editor.tagName === 'INPUT' && ['text', 'search', 'email', 'password', 'url', 'tel', 'number'].includes(editor.type)));
      const editable = editor.getAttribute('contenteditable');
      if (textControl || target.isContentEditable || editable === '' || editable === 'true' || editable === 'plaintext-only') return;
    }
  }
  event.preventDefault();
}, { capture: true });
