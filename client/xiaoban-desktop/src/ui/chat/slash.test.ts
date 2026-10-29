import { describe, expect, it } from 'vitest';
import { MAX_SKILLS, dropSlash, filterSkills, slashToken, withSkills, type SlashSkill } from './slash.ts';

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

describe('choosing a skill', () => {
  it('drops the typed /token, because the skill becomes a chip above the box', () => {
    expect(dropSlash('先做表格 /xl', { start: 5, query: 'xl' })).toBe('先做表格 ');
    expect(dropSlash('/', { start: 0, query: '' })).toBe('');
  });
  it('writes the chosen skills in front of the message as /names, in order', () => {
    expect(withSkills('做个汇报', ['pptx'])).toBe('/pptx 做个汇报');
    expect(withSkills('做个汇报', ['research', 'pptx'])).toBe('/research /pptx 做个汇报');
    expect(withSkills('', ['pptx'])).toBe('/pptx');
    expect(withSkills('你好', [])).toBe('你好');
  });
  it('allows as many skills as the host expands for one message', () => { expect(MAX_SKILLS).toBe(3); });
});
