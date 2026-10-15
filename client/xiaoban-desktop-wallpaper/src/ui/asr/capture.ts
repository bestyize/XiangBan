import { PcmResampler } from './audio.ts';
import pcmWorkletUrl from './pcm-worklet.js?url&no-inline';
export interface Capture { stop(): void }
function mediaError(error: unknown): { name: string; message: string } {
  if (error && typeof error === 'object') {
    const value = error as { name?: unknown; message?: unknown };
    return { name: typeof value.name === 'string' ? value.name : '', message: typeof value.message === 'string' ? value.message : '' };
  }
  return { name: '', message: String(error) };
}
function constraintError(error: unknown): boolean {
  const { name, message } = mediaError(error);
  return ['OverconstrainedError', 'ConstraintNotSatisfiedError', 'TypeError'].includes(name) || /invalid\s+constraint/i.test(message);
}
async function openMicrophone(): Promise<MediaStream> {
  const media = navigator.mediaDevices;
  const supported = media.getSupportedConstraints?.() ?? {};
  const audio: MediaTrackConstraints = {};
  // Let the device choose its channel count; the worklet already downmixes to mono.
  for (const key of ['echoCancellation', 'noiseSuppression', 'autoGainControl'] as const) {
    if (supported[key]) audio[key] = true;
  }
  const options: MediaStreamConstraints = { audio: Object.keys(audio).length ? audio : true, video: false };
  try { return await media.getUserMedia(options); }
  catch (error) {
    // Some desktop WebViews reject constraints they report as supported. Retry once with defaults.
    // Permission and device-access errors must retain their original meaning and never trigger a retry.
    if (options.audio === true || !constraintError(error)) throw error;
    return media.getUserMedia({ audio: true, video: false });
  }
}
export async function captureMicrophone(onFrame: (frame: Float32Array) => void, onError: (error: string) => void): Promise<Capture> {
  if (!navigator.mediaDevices?.getUserMedia) throw new Error('当前运行环境无法使用麦克风，请在桌面应用中开启语音');
  let stream: MediaStream | undefined;
  let context: AudioContext | undefined;
  let source: MediaStreamAudioSourceNode | undefined;
  let node: AudioWorkletNode | undefined;
  let stopped = false;
  const stop = () => {
    if (stopped) return; stopped = true;
    if (node) { node.port.onmessage = null; node.disconnect(); }
    source?.disconnect();
    stream?.getTracks().forEach(track => { track.onended = null; track.stop(); });
    if (context && context.state !== 'closed') void context.close().catch(() => {});
  };
  try {
    stream = await openMicrophone();
    context = new AudioContext();
    const resampler = new PcmResampler(context.sampleRate);
    await context.audioWorklet.addModule(pcmWorkletUrl);
    source = context.createMediaStreamSource(stream);
    node = new AudioWorkletNode(context, 'xiaoban-pcm', { numberOfInputs: 1, numberOfOutputs: 1, outputChannelCount: [1] });
    node.port.onmessage = event => { if (!stopped) onFrame(resampler.push(event.data as Float32Array)); };
    node.onprocessorerror = () => { stop(); onError('麦克风采集失败，请重新开启语音'); };
    stream.getAudioTracks().forEach(track => { track.onended = () => { stop(); onError('麦克风已断开，请重新开启语音'); }; });
    // The worklet emits silent output so microphone audio is never played back.
    source.connect(node); node.connect(context.destination);
    await context.resume();
    return { stop };
  } catch (error) {
    stop();
    const { name } = mediaError(error);
    if (name === 'NotAllowedError' || name === 'SecurityError') throw new Error('无法使用麦克风，请在系统隐私设置中允许小伴访问麦克风后重试');
    if (name === 'NotFoundError' || name === 'NotReadableError') throw new Error('找不到可用麦克风，请检查设备连接及是否被其他应用占用');
    if (constraintError(error)) throw new Error('当前麦克风无法开始录音，请在系统声音设置中选择可用的输入设备，或切换麦克风后重试');
    throw error;
  }
}
