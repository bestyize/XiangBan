import { createApp } from 'vue';
import PetWindow from './PetWindow.vue';

export function mountPet() {
  const app = createApp(PetWindow);
  app.mount('#app');
  window.addEventListener('pagehide', () => app.unmount(), { once: true });
}
