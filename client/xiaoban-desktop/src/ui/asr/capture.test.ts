// @vitest-environment happy-dom
import { afterEach, expect, it, vi } from 'vitest';
import { captureMicrophone, supportsMicrophone } from './capture.ts';

function setup(supported?: MediaTrackSupportedConstraints) {
  const track = { stop: vi.fn(), onended: null };
  const stream = { getTracks: () => [track], getAudioTracks: () => [track] };
  const getUserMedia = vi.fn().mockResolvedValue(stream);
  const mediaDevices = { getUserMedia, ...(supported ? { getSupportedConstraints: () => supported } : {}) };
  vi.stubGlobal('navigator', { mediaDevices });
  const source = { connect: vi.fn(), disconnect: vi.fn() };
  const context = { sampleRate: 48000, state: 'running', audioWorklet: { addModule: vi.fn().mockResolvedValue(undefined) },
    createMediaStreamSource: vi.fn(() => source), destination: {}, resume: vi.fn().mockResolvedValue(undefined), close: vi.fn().mockResolvedValue(undefined) };
  const node = { port: { onmessage: null }, connect: vi.fn(), disconnect: vi.fn(), onprocessorerror: null };
  vi.stubGlobal('AudioContext', class { constructor() { return context; } });
  vi.stubGlobal('AudioWorkletNode', class { constructor() { return node; } });
  return { getUserMedia, stream, track, source, context, node };
}
afterEach(() => vi.unstubAllGlobals());

it('detects an unsupported microphone environment without requesting permission', async () => {
  vi.stubGlobal('navigator', {});
  expect(supportsMicrophone()).toBe(false);
  await expect(captureMicrophone(vi.fn(), vi.fn())).rejects.toThrow('不支持麦克风');
});

it('requests only supported processing options and lets the device choose its channel count', async () => {
  const h = setup({ echoCancellation: true, noiseSuppression: false });
  const capture = await captureMicrophone(vi.fn(), vi.fn());
  expect(h.getUserMedia).toHaveBeenCalledExactlyOnceWith({ audio: { echoCancellation: true }, video: false });
  capture.stop(); capture.stop();
  expect(h.track.stop).toHaveBeenCalledOnce(); expect(h.context.close).toHaveBeenCalledOnce();
});

it.each([undefined, {}])('uses default audio when capability detection is unavailable or empty', async supported => {
  const h = setup(supported); const capture = await captureMicrophone(vi.fn(), vi.fn());
  expect(h.getUserMedia).toHaveBeenCalledExactlyOnceWith({ audio: true, video: false }); capture.stop();
});

it.each([
  { name: 'OverconstrainedError', message: 'Invalid constraint', constraint: 'channelCount' },
  { name: 'ConstraintNotSatisfiedError', message: 'unsupported constraint' },
  new TypeError('Invalid constraint'),
  'Invalid constraint',
])('retries a rejected constraint with default audio, including WebView error objects', async error => {
  const h = setup({ echoCancellation: true, noiseSuppression: true, autoGainControl: true });
  h.getUserMedia.mockRejectedValueOnce(error);
  const capture = await captureMicrophone(vi.fn(), vi.fn());
  expect(h.getUserMedia).toHaveBeenCalledTimes(2);
  expect(h.getUserMedia).toHaveBeenNthCalledWith(2, { audio: true, video: false });
  expect(h.context.createMediaStreamSource).toHaveBeenCalledWith(h.stream); capture.stop();
});

it('keeps a permission denial distinct and does not make a second request', async () => {
  const h = setup({ echoCancellation: true });
  h.getUserMedia.mockRejectedValue({ name: 'NotAllowedError', message: 'Denied' });
  await expect(captureMicrophone(vi.fn(), vi.fn())).rejects.toThrow('允许小伴访问麦克风');
  expect(h.getUserMedia).toHaveBeenCalledOnce(); expect(h.context.createMediaStreamSource).not.toHaveBeenCalled();
});

it('does not loop when default audio is also unavailable and reports an actionable device error', async () => {
  const h = setup({ echoCancellation: true });
  h.getUserMedia.mockRejectedValue({ name: 'OverconstrainedError', message: 'Invalid constraint' });
  await expect(captureMicrophone(vi.fn(), vi.fn())).rejects.toThrow('系统声音设置');
  expect(h.getUserMedia).toHaveBeenCalledTimes(2);
});

it('releases the microphone if audio processing fails after the fallback succeeds', async () => {
  const h = setup({ echoCancellation: true });
  h.getUserMedia.mockRejectedValueOnce(new TypeError('Invalid constraint'));
  h.context.audioWorklet.addModule.mockRejectedValue(new Error('Worklet unavailable'));
  await expect(captureMicrophone(vi.fn(), vi.fn())).rejects.toThrow('Worklet unavailable');
  expect(h.track.stop).toHaveBeenCalledOnce(); expect(h.context.close).toHaveBeenCalledOnce();
});

it('asks for the system voice isolation when it is offered, and falls back to ordinary processing without it', async () => {
  const h = setup({ echoCancellation: true, noiseSuppression: true, voiceIsolation: true } as MediaTrackSupportedConstraints);
  const capture = await captureMicrophone(vi.fn(), vi.fn());
  expect(h.getUserMedia).toHaveBeenCalledExactlyOnceWith({ audio: { echoCancellation: true, noiseSuppression: true, voiceIsolation: true }, video: false });
  capture.stop();
  const refused = setup({ echoCancellation: true, noiseSuppression: true, voiceIsolation: true } as MediaTrackSupportedConstraints);
  refused.getUserMedia.mockRejectedValueOnce(new TypeError('Invalid constraint'));
  const second = await captureMicrophone(vi.fn(), vi.fn());
  expect(refused.getUserMedia).toHaveBeenCalledTimes(2);
  expect(refused.getUserMedia).toHaveBeenNthCalledWith(2, { audio: { echoCancellation: true, noiseSuppression: true }, video: false });
  second.stop();
});
