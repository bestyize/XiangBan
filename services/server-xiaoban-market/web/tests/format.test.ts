import { describe, expect, it } from 'vitest';
import { deepLink, formatCount, formatDuration, formatSize, formatTime, highlight, initialOf, shortHash, tintFor } from '../src/lib/format';
import { toQuery } from '../src/lib/api';

describe('formatCount', () => {
  it('keeps small numbers and abbreviates 10k+ with w', () => {
    expect(formatCount(0)).toBe('0');
    expect(formatCount(9999)).toBe('9999');
    expect(formatCount(10000)).toBe('1w');
    expect(formatCount(1094300)).toBe('109w');
    expect(formatCount(24500)).toBe('2.5w');
  });
});

describe('formatSize', () => {
  it('uses B / KB / MB', () => {
    expect(formatSize(506)).toBe('506 B');
    expect(formatSize(2150)).toBe('2.1 KB');
    expect(formatSize(38 * 1024)).toBe('38 KB');
    expect(formatSize(3 * 1024 * 1024)).toBe('3.0 MB');
  });
});

describe('formatTime', () => {
  const now = new Date(2026, 9, 24, 18, 0, 0);
  it('describes recent times relatively and old ones by date', () => {
    expect(formatTime(new Date(2026, 9, 24, 9, 5).toISOString(), now)).toBe('今天 09:05');
    expect(formatTime(new Date(2026, 9, 23, 23, 59).toISOString(), now)).toBe('昨天');
    expect(formatTime(new Date(2026, 9, 21, 8, 0).toISOString(), now)).toBe('3 天前');
    expect(formatTime(new Date(2026, 9, 3, 8, 0).toISOString(), now)).toBe('3 周前');
    expect(formatTime(new Date(2026, 5, 1, 8, 0).toISOString(), now)).toBe('2026-06-01');
    expect(formatTime(null)).toBe('—');
    expect(formatTime('not a date')).toBe('—');
  });
});

describe('formatDuration', () => {
  it('formats seconds and minutes', () => {
    expect(formatDuration('2026-10-24T10:00:00Z', '2026-10-24T10:00:38Z')).toBe('38 秒');
    expect(formatDuration('2026-10-24T10:00:00Z', '2026-10-24T10:04:12Z')).toBe('4 分 12 秒');
    expect(formatDuration(null, null)).toBe('—');
  });
});

describe('highlight', () => {
  it('splits matches case-insensitively without html', () => {
    expect(highlight('PDF 转 Word', 'pdf')).toEqual([{ text: 'PDF', hit: true }, { text: ' 转 Word', hit: false }]);
    expect(highlight('合并拆分 PDF 工具', 'pdf 工具')).toEqual([
      { text: '合并拆分 ', hit: false }, { text: 'PDF', hit: true }, { text: ' ', hit: false }, { text: '工具', hit: true },
    ]);
  });
  it('returns the text untouched when there is nothing to match', () => {
    expect(highlight('abc', '')).toEqual([{ text: 'abc', hit: false }]);
    expect(highlight('abc', 'zzz')).toEqual([{ text: 'abc', hit: false }]);
    expect(highlight('<b>x</b>', 'b')).toEqual([
      { text: '<', hit: false }, { text: 'b', hit: true }, { text: '>x</', hit: false }, { text: 'b', hit: true }, { text: '>', hit: false },
    ]);
  });
});

describe('avatars', () => {
  it('picks a stable tint per seed and a visible initial', () => {
    expect(tintFor('pdf-toolkit')).toEqual(tintFor('pdf-toolkit'));
    expect(initialOf('  pdf 文档')).toBe('P');
    expect(initialOf('文档')).toBe('文');
    expect(initialOf('')).toBe('?');
  });
});

describe('links and queries', () => {
  it('builds desktop deep links with encoded segments', () => {
    expect(deepLink('install', 'skill', 'user_1', 'pdf-toolkit', '1.2.0', 'abc')).toBe('xiaoban://market/install/skills/user_1/pdf-toolkit?version=1.2.0&sha256=abc');
    expect(deepLink('open', 'expert', 'a b', 'x')).toBe('xiaoban://market/open/experts/a%20b/x');
  });
  it('drops empty query values', () => {
    expect(toQuery({ keyword: 'pdf', category: '', page: 2, none: undefined })).toBe('?keyword=pdf&page=2');
    expect(toQuery({})).toBe('');
  });
  it('shortens long hashes only', () => {
    expect(shortHash('a'.repeat(64))).toBe(`${'a'.repeat(8)}…${'a'.repeat(4)}`);
    expect(shortHash('abc')).toBe('abc');
  });
});
