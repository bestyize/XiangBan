import { createApp } from 'vue';
import '../control.css';
import ConfigurationWindow from './ConfigurationWindow.vue';
export function mountConfiguration() {
  document.body.className = 'desktop-control';
  const app = createApp(ConfigurationWindow);
  app.mount(document.querySelector('#app')!);
  window.addEventListener('beforeunload', () => app.unmount(), { once: true });
}
