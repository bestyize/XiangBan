import type { Plural } from './types';

export type Variant = 'single' | 'team' | 'bundle';

export interface SubTab {
  /** 写进地址栏 `?tab=`；第一个子标签用空串，地址更干净。 */
  key: string;
  label: string;
  variant: Variant;
  origin?: string;
}

/** 与设计稿一致：专家 / 专家团；推荐 / SkillHub / 套件；连接器没有子标签。 */
export const SUBTABS: Record<Plural, SubTab[]> = {
  experts: [
    { key: '', label: '专家', variant: 'single' },
    { key: 'team', label: '专家团', variant: 'team' },
  ],
  skills: [
    { key: '', label: '推荐', variant: 'single' },
    { key: 'skillhub', label: 'SkillHub', variant: 'single', origin: 'skillhub' },
    { key: 'bundle', label: '套件', variant: 'bundle' },
  ],
  connectors: [],
};

/** 未知的 `tab` 回到第一个子标签。 */
export function subTabFor(kind: Plural, key: string | undefined): SubTab | undefined {
  const tabs = SUBTABS[kind];
  return tabs.find((t) => t.key === (key ?? '')) ?? tabs[0];
}

export const VARIANT_LABEL: Record<Variant, string> = { single: '', team: '专家团', bundle: '套件' };

export function memberText(variant: Variant, n: number | null | undefined): string {
  if (!n) return '';
  return variant === 'team' ? `${n} 位成员` : variant === 'bundle' ? `${n} 个技能` : '';
}
