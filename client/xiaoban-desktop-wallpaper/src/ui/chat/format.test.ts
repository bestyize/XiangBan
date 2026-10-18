import desktopToolSource from '../../../src-tauri/src/chat/desktop_tools.rs?raw';
import { describe, expect, it } from 'vitest';
import { approvalPreview, clip, contextLevel, contextPercent, formatAgo, formatTokens, isImeEnter, isNearBottom, prettyArguments, riskVerb, toolLabel } from './format.ts';

describe('what the cards say', () => {
  it('names tools in plain words and falls back to the tool name', () => {
    expect(toolLabel('shell')).toBe('运行命令');
    expect(toolLabel('wallpaper_apply')).toBe('更换壁纸');
    expect(toolLabel('widget_add')).toBe('添加桌面组件');
    expect(toolLabel('widget_list')).toBe('查看桌面组件');
    expect(toolLabel('game_open')).toBe('打开游戏');
    expect(toolLabel('click')).toBe('模拟点击');
    expect(toolLabel('type_text')).toBe('模拟键盘打字');
    expect(toolLabel('some_new_tool')).toBe('some_new_tool');
    expect(toolLabel('')).toBe('工具');
    expect(['read', 'write', 'network', 'exec'].map(r => riskVerb(r as never))).toEqual(['读取', '修改', '联网', '运行命令']);
  });

  it('shows the command itself, the file and its new text, or the arguments', () => {
    expect(approvalPreview({ tool: 'shell', arguments: { command: 'rm -rf build' } })).toBe('rm -rf build');
    expect(approvalPreview({ tool: 'shell', arguments: {} })).toBe('（没有命令内容）');
    expect(approvalPreview({ tool: 'write_file', arguments: { path: 'a.md', content: '# 标题' } })).toBe('a.md\n\n# 标题');
    expect(approvalPreview({ tool: 'write_file', arguments: { path: 'a.md', content: 'x'.repeat(3000) } })).toHaveLength('a.md\n\n'.length + 1501);
    expect(approvalPreview({ tool: 'edit_file', arguments: { path: 'a.md', old_string: '旧\n行', new_string: '新' } })).toBe('a.md\n\n- 旧\n  行\n+ 新');
    expect(approvalPreview({ tool: 'wallpaper_apply', arguments: { id: 'w2' } })).toBe('{\n  "id": "w2"\n}');
    expect(approvalPreview({ tool: 'widget_add', arguments: { kind: 'fortune', theme: 'paper' } })).toBe('在桌面添加组件：fortune（paper风格）');
    expect(approvalPreview({ tool: 'widget_remove', arguments: { id: 'wgt-1' } })).toBe('从桌面移除组件：wgt-1');
    expect(approvalPreview({ tool: 'game_open', arguments: { id: 'snake-arena' } })).toBe('进入游戏空间并启动游戏：snake-arena');
    expect(approvalPreview({ tool: 'game_close', arguments: {} })).toBe('退出游戏空间，返回桌面');
    expect(approvalPreview({ tool: 'click', arguments: { app: 'TextEdit', target: '保存按钮' } })).toBe('点击：TextEdit 保存按钮');
    expect(approvalPreview({ tool: 'type_text', arguments: { app: 'TextEdit', text: 'Hello' } })).toBe('在 TextEdit 中输入：Hello');
    expect(approvalPreview({ tool: 'press_key', arguments: { app: 'TextEdit', key: 'Return' } })).toBe('按键：TextEdit Return');
    expect(approvalPreview({ tool: 'widget_update', arguments: { id: 'wgt-1', scale: 1.2 } })).toBe('调整桌面组件 wgt-1');
    expect(approvalPreview({ tool: 'x', arguments: null })).toBe('{}');
    expect(approvalPreview({ tool: 'shell', arguments: 'oops' })).toBe('（没有命令内容）');
  });

  it('cuts long text with a note about what is missing', () => {
    expect(clip('abc', 10)).toBe('abc');
    expect(clip('a'.repeat(10), 4)).toBe('aaaa\n… 还有 6 个字符没有显示');
    expect(prettyArguments('{"a":1}')).toBe('{\n  "a": 1\n}');
    expect(prettyArguments('{"half')).toBe('{"half');
  });
});

describe('typing and scrolling', () => {
  it('does not take the Enter that picks an input-method candidate for "send"', () => {
    expect(isImeEnter({ isComposing: false, keyCode: 13 }, false)).toBe(false);
    expect(isImeEnter({ isComposing: true, keyCode: 13 }, false)).toBe(true);
    expect(isImeEnter({ isComposing: false, keyCode: 229 }, false)).toBe(true);
    expect(isImeEnter({ isComposing: false, keyCode: 13 }, true)).toBe(true);
  });

  it('follows new text only while the reader is at the bottom', () => {
    expect(isNearBottom({ scrollTop: 920, clientHeight: 100, scrollHeight: 1100 })).toBe(true);
    expect(isNearBottom({ scrollTop: 1000, clientHeight: 100, scrollHeight: 1100 })).toBe(true);
    expect(isNearBottom({ scrollTop: 500, clientHeight: 100, scrollHeight: 1100 })).toBe(false);
    expect(isNearBottom({ scrollTop: 0, clientHeight: 100, scrollHeight: 100 })).toBe(true);
  });
});

describe('times', () => {
  it('says how long ago in the way people do', () => {
    const now = 1_700_000_000;
    expect(formatAgo(now - 5, now)).toBe('刚刚');
    expect(formatAgo(now + 100, now)).toBe('刚刚');
    expect(formatAgo(now - 120, now)).toBe('2 分钟前');
    expect(formatAgo(now - 7200, now)).toBe('2 小时前');
    expect(formatAgo(now - 100_000, now)).toBe('昨天');
    expect(formatAgo(now - 3 * 86400, now)).toBe('3 天前');
    expect(formatAgo(now - 30 * 86400, now)).toMatch(/^\d{1,2} 月 \d{1,2} 日$/);
  });
});



it('gives every registered desktop tool a readable label', () => {
  const source = desktopToolSource;
  const names = [...source.matchAll(/ToolSpec::new\(\s*"([^"]+)"/g)].map(match => match[1]!);
  expect(names.length).toBeGreaterThan(20);
  for (const name of names) expect(toolLabel(name), name).not.toBe(name);
});

describe('the numbers of the context ring', () => {
  it('writes token counts short', () => {
    expect(['0', 0, -5, NaN, 850, 1000, 1234, 12345, 48000, 99999, 128000, 999999, 1_000_000, 1_500_000].map(n => formatTokens(Number(n))))
      .toEqual(['0', '0', '0', '0', '850', '1K', '1.2K', '12.3K', '48K', '100K', '128K', '1M', '1M', '1.5M']);
  });

  it('turns used and limit into a percent that stays inside the ring', () => {
    expect(contextPercent(12_000, 48_000)).toBe(25);
    expect(contextPercent(60_000, 48_000)).toBe(100);
    expect(contextPercent(0, 48_000)).toBe(0);
    expect(contextPercent(5, 0)).toBe(0);
    expect(contextPercent(NaN, 48_000)).toBe(0);
  });

  it('warns from 70% and calls it full from 90%', () => {
    expect([0, 0.69, 0.7, 0.89, 0.9, 1.4].map(r => contextLevel(r * 1000, 1000))).toEqual(['ok', 'ok', 'warn', 'warn', 'full', 'full']);
    expect(contextLevel(10, 0)).toBe('ok');
  });
});
