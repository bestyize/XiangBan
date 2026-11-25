function syncStudyCapsule() {
  document.documentElement.toggleAttribute('data-host-capsule', Boolean(window.xb.info?.capsule?.width));
}

syncStudyCapsule();
window.xb.on?.('safearea', syncStudyCapsule);
document.addEventListener('DOMContentLoaded', () => {
  let activeDialog;
  const observer = new MutationObserver(records => {
    for (const { target } of records) {
      if (!target.open || target === activeDialog) continue;
      activeDialog = target;
      history.pushState(null, '');
    }
  });
  for (const dialog of document.querySelectorAll('dialog')) {
    const button = dialog.querySelector('form[method="dialog"] button');
    if (button) {
      button.type = 'button';
      button.addEventListener('click', () => dialog.close());
    }
    dialog.addEventListener('close', () => {
      if (dialog !== activeDialog) return;
      activeDialog = null;
      history.back();
    });
    observer.observe(dialog, { attributeFilter: ['open'] });
  }
  window.addEventListener('popstate', () => {
    const dialog = activeDialog;
    activeDialog = null;
    dialog?.close();
  });
  requestAnimationFrame(() => window.xb.ready());
}, { once: true });
