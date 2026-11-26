import { applyUiTokens } from './ui/design-tokens';
import { invoke, listen } from './bridge.ts';
import { type CareDelivery, type CareKind } from './care-model.ts';
import './care.css';

const KIND_LABELS: Record<CareKind, string> = {
  custom: '关怀提醒',
  rest: '久坐休息',
  'off-work': '下班提醒',
  weather: '今日天气',
  agent: '定时任务完成',
};

const KIND_SVGS: Record<CareKind, string> = {
  custom: '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linejoin="round"><path d="M12 3s6 6.5 6 11a6 6 0 0 1-12 0c0-4.5 6-11 6-11z"/></svg>',
  rest: '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M17 8h1a4 4 0 1 1 0 8h-1"/><path d="M3 8h14v9a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4Z"/><line x1="6" y1="2" x2="6" y2="4"/><line x1="10" y1="2" x2="10" y2="4"/></svg>',
  'off-work': '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>',
  weather: '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>',
  agent: '<svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="10" rx="2"/><circle cx="12" cy="5" r="2"/><path d="M12 7v4"/><line x1="8" y1="16" x2="8" y2="16.01"/><line x1="16" y1="16" x2="16" y2="16.01"/></svg>',
};

function formatClock(unixSeconds?: number): string {
  const date = unixSeconds ? new Date(unixSeconds * 1000) : new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export async function mountCareBubble(): Promise<void> {
  applyUiTokens();
  document.body.className = 'care-bubble-scene';
  const host = document.querySelector<HTMLElement>('#app')!;
  host.innerHTML = [
    '<aside class="care-bubble" role="status" aria-live="polite" hidden>',
    '  <div class="care-bubble-inner">',
    '    <div class="care-bubble-main">',
    '      <span class="care-bubble-ico" aria-hidden="true"></span>',
    '      <div class="care-bubble-body">',
    '        <div class="care-bubble-top">',
    '          <span class="care-bubble-kind"></span>',
    '          <span class="care-bubble-dot"></span>',
    '          <span class="care-bubble-time"></span>',
    '          <span class="care-bubble-sticky-dot" hidden title="常驻强提醒"></span>',
    '        </div>',
    '        <p class="care-bubble-text"></p>',
    '        <div class="care-bubble-extra" hidden></div>',
    '      </div>',
    '      <button type="button" class="care-close" aria-label="关闭提醒">',
    '        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12"/></svg>',
    '      </button>',
    '    </div>',
    '    <div class="care-bubble-bottom">',
    '      <div class="care-bubble-status">',
    '        <button type="button" class="care-voice-btn" hidden title="点击停止朗读">',
    '          <span class="care-wave"><i></i><i></i><i></i></span>',
    '          <span class="care-voice-label">朗读中</span>',
    '        </button>',
    '        <small class="care-bubble-count"></small>',
    '      </div>',
    '      <div class="care-bubble-actions">',
    '        <button type="button" class="care-snooze">10 分钟后</button>',
    '        <button type="button" class="care-ack" hidden>知道了</button>',
    '        <button type="button" class="care-open-chat" hidden>查看完整结果 ↗</button>',
    '      </div>',
    '    </div>',
    '    <p class="care-bubble-error" role="alert" hidden></p>',
    '  </div>',
    '  <div class="care-bubble-progress" hidden><div class="care-bubble-bar"></div></div>',
    '</aside>',
  ].join('');

  const bubble = host.querySelector<HTMLElement>('.care-bubble')!;
  const ico = host.querySelector<HTMLElement>('.care-bubble-ico')!;
  const kindEl = host.querySelector<HTMLElement>('.care-bubble-kind')!;
  const timeEl = host.querySelector<HTMLElement>('.care-bubble-time')!;
  const stickyDot = host.querySelector<HTMLElement>('.care-bubble-sticky-dot')!;
  const textEl = host.querySelector<HTMLElement>('.care-bubble-text')!;
  const extraEl = host.querySelector<HTMLElement>('.care-bubble-extra')!;
  const voiceBtn = host.querySelector<HTMLButtonElement>('.care-voice-btn')!;
  const voiceLabel = host.querySelector<HTMLElement>('.care-voice-label')!;
  const countEl = host.querySelector<HTMLElement>('.care-bubble-count')!;
  const snoozeBtn = host.querySelector<HTMLButtonElement>('.care-snooze')!;
  const ackBtn = host.querySelector<HTMLButtonElement>('.care-ack')!;
  const openChatBtn = host.querySelector<HTMLButtonElement>('.care-open-chat')!;
  const errorEl = host.querySelector<HTMLElement>('.care-bubble-error')!;
  const progressWrap = host.querySelector<HTMLElement>('.care-bubble-progress')!;
  const progressBar = host.querySelector<HTMLElement>('.care-bubble-bar')!;

  let current: CareDelivery | null = null;
  let audio: HTMLAudioElement | null = null;
  let audioUrl = '';

  function setVoiceStopped(stopped: boolean) {
    voiceBtn.classList.toggle('is-stopped', stopped);
    voiceLabel.textContent = stopped ? '已静音' : '朗读中';
  }

  function stopVoice() {
    if (audio) {
      audio.pause();
      audio.onended = null;
      audio.onerror = null;
      audio.src = '';
      audio = null;
    }
    if (audioUrl) {
      URL.revokeObjectURL(audioUrl);
      audioUrl = '';
    }
    setVoiceStopped(true);
  }

  async function speak(delivery: CareDelivery) {
    setVoiceStopped(false);
    try {
      const bytes = await invoke<ArrayBuffer | number[]>('care_synthesize', { id: delivery.id });
      if (current?.id !== delivery.id) return;
      const data = bytes instanceof ArrayBuffer ? bytes : new Uint8Array(bytes);
      if (!data.byteLength) throw new Error('语音服务没有返回音频');
      audioUrl = URL.createObjectURL(new Blob([data], { type: 'audio/mpeg' }));
      audio = new Audio(audioUrl);
      audio.onended = stopVoice;
      audio.onerror = () => {
        if (current?.id === delivery.id) {
          errorEl.textContent = '语音播放失败，文字提醒仍可查看。';
          errorEl.hidden = false;
        }
        stopVoice();
      };
      await audio.play();
    } catch {
      if (current?.id === delivery.id) {
        errorEl.textContent = '语音播报暂不可用，文字提醒仍可查看。';
        errorEl.hidden = false;
        stopVoice();
      }
    }
  }

  const apply = (delivery: CareDelivery | null) => {
    const changed = current?.id !== delivery?.id;
    if (changed) stopVoice();
    current = delivery;
    bubble.hidden = !delivery;
    errorEl.textContent = '';
    errorEl.hidden = true;
    if (!delivery) return;

    const primary = delivery.items[0];
    const kind: CareKind = primary?.kind ?? 'custom';
    const isPreview = delivery.items.every(item => item.preview);
    const isSticky = delivery.items.some(item => item.sticky) || delivery.displaySeconds >= 3600;
    const session = delivery.items.find(item => item.session)?.session ?? null;

    bubble.classList.toggle('is-sticky', isSticky);
    ico.className = `care-bubble-ico is-${kind}`;
    ico.innerHTML = KIND_SVGS[kind] ?? KIND_SVGS.custom;
    kindEl.textContent = KIND_LABELS[kind] ?? '关怀提醒';
    timeEl.textContent = formatClock(primary?.dueAt);
    stickyDot.hidden = !isSticky;

    textEl.textContent = primary?.text?.trim() || '记得休息一下～';
    const extraCount = Math.max(0, delivery.items.length - 1);
    extraEl.hidden = extraCount === 0;
    if (extraCount > 0) extraEl.textContent = `另有 ${extraCount} 条同时到期的提醒`;

    voiceBtn.hidden = !delivery.voice;
    countEl.hidden = delivery.voice;
    if (isPreview) {
      countEl.textContent = '预览样式，不创建任务';
    } else if (isSticky) {
      countEl.textContent = delivery.sound ? '常驻强提醒 · 已响铃' : '常驻强提醒';
    } else {
      countEl.textContent = delivery.sound ? '提示音 · 悬停暂停收起' : '悬停暂停收起';
    }

    const isAgentWithSession = kind === 'agent' && Boolean(session);
    openChatBtn.hidden = !isAgentWithSession;
    snoozeBtn.hidden = isPreview || isAgentWithSession;
    ackBtn.hidden = !isSticky || isAgentWithSession;

    progressWrap.hidden = isSticky;
    if (!isSticky) {
      const secs = Math.max(5, Math.min(60, delivery.displaySeconds || 10));
      progressBar.style.animation = 'none';
      void progressBar.offsetWidth;
      progressBar.style.setProperty('--bub-dur', `${secs}s`);
      progressBar.style.animation = '';
    }

    if (changed && delivery.voice) void speak(delivery);
  };

  const act = async (command: string, extraPayload: Record<string, unknown> = {}) => {
    if (!current) return;
    const id = current.id;
    try {
      await invoke(command, { id, ...extraPayload });
      if (current?.id === id) apply(null);
    } catch (problem) {
      errorEl.textContent = String(problem);
      errorEl.hidden = false;
    }
  };

  host.querySelector<HTMLButtonElement>('.care-close')!.addEventListener('click', () => void act('dismiss_care'));
  ackBtn.addEventListener('click', () => void act('dismiss_care'));
  snoozeBtn.addEventListener('click', () => void act('snooze_care'));
  openChatBtn.addEventListener('click', () => {
    const session = current?.items.find(item => item.session)?.session;
    if (session) void act('care_open_session', { session });
  });
  voiceBtn.addEventListener('click', () => {
    if (audio) stopVoice();
    else if (current?.voice) void speak(current);
  });

  bubble.addEventListener('mouseenter', () => { void invoke('care_hover', { hovering: true }).catch(() => {}); });
  bubble.addEventListener('mouseleave', () => { void invoke('care_hover', { hovering: false }).catch(() => {}); });

  const unlisten = await listen<CareDelivery | null>('care-delivery', apply);
  apply(await invoke<CareDelivery | null>('care_ready'));
  window.addEventListener('beforeunload', () => { stopVoice(); unlisten(); }, { once: true });
}
