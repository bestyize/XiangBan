import { applyUiTokens } from './ui/design-tokens';
import { invoke, listen } from "./bridge.ts";
import { deliverySummary, type CareDelivery } from "./care-model.ts";
import "./care.css";

export async function mountCareBubble(): Promise<void> {
  applyUiTokens();
  document.body.className = "care-bubble-scene";
  const host = document.querySelector<HTMLElement>("#app")!;
  host.innerHTML = '<aside class="care-bubble" role="status" aria-live="polite" hidden><div class="care-bubble-top"><span>小伴的轻声提醒</span><button class="care-close" aria-label="关闭提醒">×</button></div><p class="care-bubble-text"></p><div class="care-bubble-bottom"><small class="care-bubble-count"></small><button class="care-snooze">10 分钟后提醒</button></div><p class="care-bubble-error" role="alert"></p></aside>';
  const bubble = host.querySelector<HTMLElement>(".care-bubble")!;
  const text = host.querySelector<HTMLElement>(".care-bubble-text")!;
  const error = host.querySelector<HTMLElement>(".care-bubble-error")!;
  const snooze = host.querySelector<HTMLButtonElement>(".care-snooze")!;
  let current: CareDelivery | null = null;
  let audio: HTMLAudioElement | null = null;
  let audioUrl = "";
  function stopVoice() {
    if (audio) { audio.pause(); audio.onended = null; audio.onerror = null; audio.src = ""; audio = null; }
    if (audioUrl) { URL.revokeObjectURL(audioUrl); audioUrl = ""; }
  }
  async function speak(delivery: CareDelivery) {
    try {
      const bytes = await invoke<ArrayBuffer | number[]>("care_synthesize", { id: delivery.id });
      if (current?.id !== delivery.id) return;
      const data = bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes);
      if (!data.byteLength) throw new Error("语音服务没有返回音频");
      audioUrl = URL.createObjectURL(new Blob([data], { type: "audio/mpeg" }));
      audio = new Audio(audioUrl);
      audio.onended = stopVoice;
      audio.onerror = () => { if (current?.id === delivery.id) error.textContent = "语音播放失败，文字提醒仍可查看。"; stopVoice(); };
      await audio.play();
    } catch {
      if (current?.id === delivery.id) { error.textContent = "语音播报暂不可用，文字提醒仍可查看。"; stopVoice(); }
    }
  }
  const apply = (delivery: CareDelivery | null) => {
    const changed = current?.id !== delivery?.id;
    if (changed) stopVoice();
    current = delivery;
    bubble.hidden = !delivery;
    error.textContent = "";
    if (!delivery) return;
    text.textContent = deliverySummary(delivery);
    snooze.hidden = delivery.items.every(item => item.preview);
    host.querySelector<HTMLElement>(".care-bubble-count")!.textContent = delivery.items.some(item => item.preview) ? "仅预览，不创建任务" : `${delivery.items.length} 条到期提醒 · ${delivery.displaySeconds} 秒后收起`;
    if (changed && delivery.voice) void speak(delivery);
  };
  const act = async (command: string) => {
    if (!current) return;
    const id = current.id;
    try { await invoke(command, { id }); if (current?.id === id) apply(null); }
    catch (problem) { error.textContent = String(problem); }
  };
  host.querySelector<HTMLButtonElement>(".care-close")!.addEventListener("click", () => void act("dismiss_care"));
  snooze.addEventListener("click", () => void act("snooze_care"));
  const unlisten = await listen<CareDelivery | null>("care-delivery", apply);
  apply(await invoke<CareDelivery | null>("care_ready"));
  window.addEventListener("beforeunload", () => { stopVoice(); unlisten(); }, { once: true });
}
