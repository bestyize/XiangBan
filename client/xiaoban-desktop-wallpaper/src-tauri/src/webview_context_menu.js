// Installed at document start in every WebView and child frame.
// Cancel the browser menu without stopping app-defined contextmenu handlers.
window.addEventListener('contextmenu', function (event) {
  event.preventDefault();
}, { capture: true });
