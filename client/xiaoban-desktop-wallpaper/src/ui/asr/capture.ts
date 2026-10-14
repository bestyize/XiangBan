import { PcmResampler } from './audio.ts';
import pcmWorkletUrl from './pcm-worklet.js?url&no-inline';
export interface Capture { stop(): void }
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
    stream = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true }, video: false });
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
    const name = error instanceof Error ? error.name : '';
    if (name === 'NotAllowedError' || name === 'SecurityError') throw new Error('无法使用麦克风，请在系统隐私设置中允许小伴访问麦克风后重试');
    if (name === 'NotFoundError' || name === 'NotReadableError') throw new Error('找不到可用麦克风，请检查设备连接及是否被其他应用占用');
    throw error;
  }
}
