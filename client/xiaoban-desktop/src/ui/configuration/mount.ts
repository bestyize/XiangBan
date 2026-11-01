import { createApp } from 'vue';
import '../control.css';
import ConfigurationWindow from './ConfigurationWindow.vue';
import CommunitySchemesWindow from '../community/CommunitySchemesWindow.vue';
import StoreWindow from '../ai-store/StoreWindow.vue';
export function mountConfiguration() {
  document.body.className = 'desktop-control';
  const kind = new URLSearchParams(location.search).get('kind');
  const app = createApp(kind === 'community' ? CommunitySchemesWindow : kind === 'store' ? StoreWindow : ConfigurationWindow);
  app.mount(document.querySelector('#app')!);
  window.addEventListener('beforeunload', () => app.unmount(), { once: true });
}
