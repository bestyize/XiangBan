import { createApp } from 'vue';
import GreetingBubble from './GreetingBubble.vue';

export function mountGreetingBubble() {
 const app = createApp(GreetingBubble);
 app.mount('#app');
 window.addEventListener('pagehide', () => app.unmount(), { once: true });
}
