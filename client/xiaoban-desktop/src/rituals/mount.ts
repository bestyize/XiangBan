import { createApp } from 'vue';
import RitualSpace from './RitualSpace.vue';
export function mountRitual() {
  document.body.className = 'ritual-scene';
  const app = createApp(RitualSpace);
  app.mount('#app');
  window.addEventListener('pagehide', () => app.unmount(), { once: true });
}
