import { describe, expect, it } from 'vitest';
import { memberText, SUBTABS, subTabFor, VARIANT_LABEL } from '../src/lib/tabs';

describe('sub tabs', () => {
  it('matches the design: experts/teams, recommended/SkillHub/bundles, no tabs for connectors', () => {
    expect(SUBTABS.experts.map((t) => t.label)).toEqual(['专家', '专家团']);
    expect(SUBTABS.skills.map((t) => t.label)).toEqual(['推荐', 'SkillHub', '套件']);
    expect(SUBTABS.connectors).toEqual([]);
  });

  it('maps a tab key to the filters sent to the API and falls back to the first tab', () => {
    expect(subTabFor('experts', 'team')).toMatchObject({ variant: 'team' });
    expect(subTabFor('skills', 'skillhub')).toMatchObject({ variant: 'single', origin: 'skillhub' });
    expect(subTabFor('skills', 'bundle')).toMatchObject({ variant: 'bundle' });
    expect(subTabFor('skills', '')).toMatchObject({ key: '', variant: 'single' });
    expect(subTabFor('skills', 'nonsense')).toMatchObject({ key: '' });
    expect(subTabFor('connectors', 'team')).toBeUndefined();
  });

  it('describes members per variant', () => {
    expect(memberText('team', 5)).toBe('5 位成员');
    expect(memberText('bundle', 3)).toBe('3 个技能');
    expect(memberText('single', 3)).toBe('');
    expect(memberText('team', null)).toBe('');
    expect(VARIANT_LABEL.team).toBe('专家团');
  });
});
