import { describe, expect, it } from 'vitest';
import { modelFailure, retryNotice, retryReason, silenceNotice, waitedFor } from './model-errors.ts';
import { applyEvent, newView } from './view.ts';
describe('model failure recovery', () => {
  it('does not treat a missing route or temporary limits as a retired model', () => {
    expect(modelFailure('HTTP 404: route not found')).toMatchObject({ refresh: false });
    expect(modelFailure('model_not_found')).toMatchObject({ refresh: true });
    expect(modelFailure('HTTP 429 rate_limit')).toMatchObject({ refresh: false });
    expect(modelFailure('insufficient_quota').message).toContain('额度');
    expect(modelFailure('invalid_api_key').message).toContain('认证');
  });
  it('offers original-message retry but stops offering it after output or a tool action', () => {
    for (const partial of [false, true]) {
      const view = newView('s');
      view.items.push({ kind: 'user', id: 'u', text: 'hello', images: ['data:image/png;base64,x'] });
      if (partial) applyEvent(view, { type: 'tool_start', id: 't', name: 'shell', arguments: '{}' });
      applyEvent(view, { type: 'error', message: 'model_not_found' });
      expect(view.items.at(-1)).toMatchObject({ canSwitchModel: true, canRetry: !partial });
      expect(view.items.filter(i => i.kind === 'user')).toHaveLength(1);
    }
  });
  it('explains an oversized request without offering an identical retry or continuation', () => {
    for (const message of ['request_too_large: too large', 'HTTP 413: payload too large']) {
      for (const partial of [false, true]) {
        const view = newView('s');
        view.items.push({ kind: 'user', id: 'u', text: 'hello', images: [] });
        if (partial) applyEvent(view, { type: 'tool_start', id: 't', name: 'read_file', arguments: '{}' });
        applyEvent(view, { type: 'error', message });
        expect(view.items.at(-1)).toMatchObject({ text: expect.stringContaining('新开会话') });
        expect(view.items.at(-1)).not.toHaveProperty('canRetry');
        expect(view.items.at(-1)).not.toHaveProperty('canContinue');
      }
    }
  });
});

describe('what the page says while the host retries or waits', () => {
  it('names the reason of an automatic retry in plain words', () => {
    expect(retryReason('model HTTP timeout waiting for response headers: no data for 45s')).toBe('模型服务长时间没有响应');
    expect(retryReason('model HTTP stream read timed out: error decoding response body: operation timed out')).toBe('模型服务长时间没有响应');
    expect(retryReason('HTTP 429: rate_limit_error')).toBe('模型服务繁忙，请求被限流');
    expect(retryReason('HTTP 529: overloaded')).toBe('模型服务暂时出错');
    expect(retryReason('stream ended before terminal event')).toBe('与模型的连接中断了');
    expect(retryReason('something odd')).toBe('与模型的连接不稳定');
    expect(retryReason('模型上下文超限，已缩减旧工具结果和历史，正在重试；已执行的操作不会重放。')).toBe('模型上下文超限，已缩减旧工具结果和历史');
    expect(retryNotice(2, 'HTTP 503')).toBe('模型服务暂时出错，正在进行第 2 次重试…');
  });
  it('counts a silent model in seconds, and keeps the retry line when one is showing', () => {
    expect(waitedFor(7)).toBe('7 秒');
    expect(waitedFor(65)).toBe('1 分 05 秒');
    expect(silenceNotice(9)).toBe('');
    expect(silenceNotice(9, '重试中')).toBe('重试中');
    expect(silenceNotice(30)).toBe('模型暂时没有新输出，已等待 30 秒；长时间没有响应会自动重试。');
    expect(silenceNotice(30, '模型服务长时间没有响应，正在进行第 1 次重试…')).toBe('模型服务长时间没有响应，正在进行第 1 次重试…（已等待 30 秒）');
  });
});
