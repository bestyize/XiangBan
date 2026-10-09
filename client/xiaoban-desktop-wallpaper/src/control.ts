import { createApp } from "vue";
import App from "./ui/App.vue";

export async function mountControl() {
  document.body.className = "desktop-control";
  const host = document.querySelector<HTMLElement>("#app");
  if (!host) throw new Error("设置面板的挂载节点不存在");
  const app = createApp(App);
  app.mount(host);
  window.addEventListener("beforeunload", () => app.unmount(), { once: true });
}
