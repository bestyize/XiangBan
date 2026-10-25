export interface Skill {
  slug: string; name: string; description: string; version: string; namespace?: string; iconUrl?: string | null; source?: string; downloads?: number; stars?: number; removable?: boolean;
  /** Only from the market: a team or bundle installs several packages at once. */
  variant?: 'single' | 'team' | 'bundle'; memberCount?: number | null; category?: string | null;
  /** Only on installed entries that came from the market. */
  market?: boolean; marketKind?: 'skill' | 'expert' | null; manifestSha256?: string | null; files?: { path: string; sha256: string }[] | null;
  /** Installed WorkBuddy-style expert (an agent persona): using it summons it for a new conversation. */
  plugin?: boolean; profession?: string; agent?: string | null; quickPrompts?: string[]; tags?: string[]; hasAvatar?: boolean; pluginKind?: 'agent' | 'team';
}
export interface SkillPage { results: Skill[]; page: number; total: number | null; hasMore: boolean }
export interface SkillReview {
  slug: string; namespace?: string; sha256: string; content: string;
  manifest: { version: string; fileCount: number; totalBytes: number; files: { path: string; size: number; sha256: string; executableScript: boolean }[] };
  scripts: { path: string; sha256: string; preview: string | null; previewTruncated: boolean }[];
}
