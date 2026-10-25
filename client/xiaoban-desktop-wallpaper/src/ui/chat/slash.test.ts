import { describe, expect, it } from 'vitest';
import { applySlash, filterSkills, slashToken, withLeadingSlash, type SlashSkill } from './slash.ts';

const skill = (name: string, displayName = name, description = ''): SlashSkill => ({ name, displayName, description, source: 'installed' });

describe('slashToken', () => {
  it('finds the /name being typed at the end, after the start or whitespace only', () => {
    expect(slashToken('/')).toEqual({ start: 0, query: '' });
    expect(slashToken('/ppt')).toEqual({ start: 0, query: 'ppt' });
    expect(slashToken('先做表格 /xl')).toEqual({ start: 5, query: 'xl' });
    expect(slashToken('换行后\n/a-b1')).toEqual({ start: 4, query: 'a-b1' });
  });
  it('ignores paths, finished names and anything else', () => {
    for (const text of ['', '你好', 'a/b', 'https://x.com/p', '/pptx 做汇报', '/Upper', '/中文', '看 /etc/hosts']) expect(slashToken(text), text).toBeNull();
  });
});

describe('filterSkills', () => {
  const items = [skill('pptx', 'PPT 演示文稿', '做幻灯片'), skill('xlsx', 'Excel 表格', '处理表格和 ppt 图表'), skill('docx', 'Word', '文档'), skill('pdf-tools')];
  it('ranks name prefixes first, then display names, then other matches, descriptions last', () => {
    expect(filterSkills(items, 'p').map((s) => s.name)).toEqual(['pdf-tools', 'pptx', 'xlsx']);
    expect(filterSkills(items, 'ppt').map((s) => s.name)).toEqual(['pptx', 'xlsx']);
    expect(filterSkills(items, 'excel').map((s) => s.name)).toEqual(['xlsx']);
    expect(filterSkills(items, 'zzz')).toEqual([]);
  });
  it('lists everything by name for an empty query, capped', () => {
    expect(filterSkills(items, '').map((s) => s.name)).toEqual(['docx', 'pdf-tools', 'pptx', 'xlsx']);
    expect(filterSkills(items, '', 2)).toHaveLength(2);
  });
});

describe('inserting a skill', () => {
  it('replaces the typed token with the chosen name and a space', () => {
    expect(applySlash('先做表格 /xl', { start: 5, query: 'xl' }, 'xlsx')).toBe('先做表格 /xlsx ');
    expect(applySlash('/', { start: 0, query: '' }, 'pptx')).toBe('/pptx ');
  });
  it('puts a skill from the plus menu in front of the draft, once', () => {
    expect(withLeadingSlash('', 'pptx')).toBe('/pptx ');
    expect(withLeadingSlash('做个汇报', 'pptx')).toBe('/pptx 做个汇报');
    expect(withLeadingSlash('/pptx 做个汇报', 'pptx')).toBe('/pptx 做个汇报');
    expect(withLeadingSlash('/pptxx 做', 'pptx')).toBe('/pptx /pptxx 做');
  });
});
