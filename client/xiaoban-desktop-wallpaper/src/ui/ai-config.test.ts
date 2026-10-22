import { describe, expect, it } from 'vitest';
import { aiProtocols, endpointPreview, insecureRemote, normalizeBaseUrl, protocolOption, supportsReplyPreference, reasoningLevels, preferenceOptions, providerReasoningLevels, validateAiForm, validateBaseUrl, validateKey } from './ai-config.ts';

describe('model service form helpers', () => {
  it('uses model and protocol capabilities regardless of endpoint domain', () => {
    expect(supportsReplyPreference('responses', ' https://api.openai.com/v1/ ', 'gpt-6.1-sol')).toBe(true);
    expect(supportsReplyPreference('responses', 'https://api.deepseek.com', 'deepseek-flash')).toBe(true);
    expect(supportsReplyPreference('responses', 'https://gateway.example/v1', 'gpt-6.1-sol')).toBe(true);
    expect(supportsReplyPreference('chat', 'https://gateway.example/v1', 'glm-5.3')).toBe(true);
    for (const [protocol, url, model] of [
      ['responses', 'https://api.openai.com/v1', 'gpt-4o'],
      ['anthropic', 'https://api.deepseek.com/anthropic', 'deepseek-flash'],
    ] as const) expect(supportsReplyPreference(protocol, url, model)).toBe(false);
  });
  it('supports declared aliases, disabled capabilities and only valid grades', () => {
    expect(reasoningLevels('chat', 'private-alias', ['medium', 'high'])).toEqual(['medium', 'high']);
    expect(reasoningLevels('responses', 'gpt-6.1-sol', [])).toEqual([]);
    expect(reasoningLevels('responses', 'moonshot/kimi-k3')).toEqual(['low', 'high', 'max']);
    expect(reasoningLevels('anthropic', 'claude-opus-5-5')).toEqual(['low', 'medium', 'high', 'xhigh', 'max']);
    expect(reasoningLevels('anthropic', 'anthropic/claude-sonnet-4-6')).toEqual(['low', 'medium', 'high', 'max']);
    expect(reasoningLevels('anthropic', 'claude-haiku-4-5'), 'budget_tokens models are left alone').toEqual([]);
    expect(preferenceOptions(['high', 'low']).map(o => o.value)).toEqual(['default', 'faster', 'deeper']);
    const p = {id:'test', name:'t', isCustom:false, model:'alias', protocol:'responses' as const, baseUrl:'https://proxy.example', hasKey:true, keyHint:'', reasoningEfforts:['high'] as const};
    expect(providerReasoningLevels({...p,reasoningEfforts:['high']}, 'alias')).toEqual(['high']);
    expect(providerReasoningLevels({...p,reasoningEfforts:['high']}, 'other')).toEqual([]);
  });
  it('previews the address each protocol will call', () => {
    expect(endpointPreview('chat', 'https://api.openai.com/v1')).toBe('https://api.openai.com/v1/chat/completions');
    expect(endpointPreview('chat', '  https://api.openai.com/v1//  ')).toBe('https://api.openai.com/v1/chat/completions');
    expect(endpointPreview('responses', 'https://api.openai.com/v1')).toBe('https://api.openai.com/v1/responses');
    expect(endpointPreview('anthropic', 'https://api.anthropic.com')).toBe('https://api.anthropic.com/v1/messages');
    expect(endpointPreview('anthropic', 'https://gw.example.com/anthropic/v1/')).toBe('https://gw.example.com/anthropic/v1/messages');
    expect(endpointPreview('chat', '   ')).toBe('');
    expect(normalizeBaseUrl(' https://a.example.com/ ')).toBe('https://a.example.com');
  });
  it('accepts normal addresses and explains the common mistakes', () => {
    for (const good of ['https://api.openai.com/v1', 'http://localhost:11434/v1', ' https://gw.example.com/proxy/ ']) expect(validateBaseUrl(good)).toBe('');
    const cases: [string, string][] = [
      ['', '请填写'], ['   ', '请填写'], ['api.openai.com/v1', '格式不对'], ['not a url', '格式不对'],
      ['ftp://example.com', 'http://'], ['file:///etc/passwd', 'http://'], ['javascript:alert(1)', 'http://'],
      ['https://user:pw@example.com/v1', '用户名或密码'], ['https://example.com/v1?key=abc', '参数'], ['https://example.com/v1#x', '参数'],
    ];
    for (const [bad, fragment] of cases) expect(validateBaseUrl(bad), bad).toContain(fragment);
  });
  it('needs an address and a model, and checks only a typed key', () => {
    expect(validateAiForm({ baseUrl: 'https://a.example.com', model: 'm' })).toBe('');
    expect(validateAiForm({ baseUrl: '', model: 'm' })).toContain('API 地址');
    expect(validateAiForm({ baseUrl: 'https://a.example.com', model: '  ' })).toContain('模型名');
    expect(validateKey('')).toBe('');
    expect(validateKey('  sk-abc123\n')).toBe('');
    expect(validateKey('sk abc')).toContain('空格');
    expect(validateKey('sk-abc\nX-Evil: 1')).toContain('空格');
  });
  it('warns about plain http except to this computer', () => {
    for (const risky of ['http://192.168.1.5:8080/v1', 'http://api.example.com/v1', 'http://10.0.0.2', 'http://[2001:db8::1]/v1']) expect(insecureRemote(risky), risky).toBe(true);
    for (const fine of ['http://localhost:11434/v1', 'http://127.0.0.1:1234', 'http://[::1]:8080', 'https://api.example.com/v1', '', 'nonsense']) expect(insecureRemote(fine), fine).toBe(false);
  });
  it('offers the three protocols with the right starting addresses', () => {
    expect(aiProtocols.map(option => option.value)).toEqual(['responses', 'anthropic', 'chat']);
    expect(protocolOption('anthropic').placeholder).toBe('https://api.anthropic.com');
    expect(protocolOption('chat').placeholder).toContain('/v1');
    expect(protocolOption('chat').label).toContain('Chat Completions');
  });
});
