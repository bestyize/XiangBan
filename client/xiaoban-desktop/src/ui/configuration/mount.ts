import { createApp } from 'vue';
import '../control.css';
import ConfigurationWindow from './ConfigurationWindow.vue';
import CommunitySchemesWindow from '../community/CommunitySchemesWindow.vue';
export function mountConfiguration() {
  document.body.className = 'desktop-control';
  const app = createApp(new URLSearchParams(location.search).get("kind") === "community" ? CommunitySchemesWindow : ConfigurationWindow);
  app.mount(document.querySelector('#app')!);
  window.addEventListener('beforeunload', () => app.unmount(), { once: true });
}
