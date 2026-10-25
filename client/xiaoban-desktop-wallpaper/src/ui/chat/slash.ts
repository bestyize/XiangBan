/**
 * `/` 菜单：输入框里输入 `/` 加技能名，从用户可以启动的技能里选。选中后文字里留下 `/名称 `，
 * 发送时由后端（skill_invocation）把这个技能的完整说明放到这条请求后面。
 */

export interface SlashSkill {
  name: string;
  displayName: string;
  description: string;
  descriptionEn?: string | null;
  /** plugin | installed | assistant | builtin */
  source: string;
  plugin?: string | null;
  version?: string | null;
  modelInvocable?: boolean;
}

export interface SlashToken { start: number; query: string }

/** 末尾正在输入的 `/xxx`：必须在开头或空白后面，名称只含小写字母、数字和连字符。 */
export function slashToken(draft: string): SlashToken | null {
  const match = /(^|\s)\/([a-z0-9-]*)$/.exec(draft);
  return match ? { start: match.index + match[1].length, query: match[2] } : null;
}

/** 名称开头匹配的排前面，其次名称、显示名、说明里包含的；空查询按名称排序。 */
export function filterSkills(items: SlashSkill[], query: string, limit = 8): SlashSkill[] {
  const q = query.trim().toLowerCase();
  const scored = items.flatMap((item) => {
    if (!q) return [{ item, score: 3 }];
    const name = item.name.toLowerCase();
    const display = item.displayName.toLowerCase();
    if (name.startsWith(q)) return [{ item, score: 0 }];
    if (display.startsWith(q)) return [{ item, score: 1 }];
    if (name.includes(q) || display.includes(q)) return [{ item, score: 2 }];
    if (item.description.toLowerCase().includes(q)) return [{ item, score: 4 }];
    return [];
  });
  scored.sort((a, b) => a.score - b.score || a.item.name.localeCompare(b.item.name));
  return scored.slice(0, limit).map((s) => s.item);
}

/** 把末尾的 `/xxx` 换成选中的 `/名称 `。 */
export function applySlash(draft: string, token: SlashToken, name: string): string {
  return `${draft.slice(0, token.start)}/${name} `;
}

/** 从“＋”菜单选了技能：把 `/名称 ` 放在草稿最前面，已经有就不重复。 */
export function withLeadingSlash(draft: string, name: string): string {
  const lead = `/${name}`;
  if (draft === lead || draft.startsWith(`${lead} `) || draft.startsWith(`${lead}\n`)) return draft;
  return draft.trim() ? `${lead} ${draft}` : `${lead} `;
}

export const SOURCE_LABEL: Record<string, string> = { plugin: '插件', installed: '已安装', assistant: '小伴创建', builtin: '内置' };
