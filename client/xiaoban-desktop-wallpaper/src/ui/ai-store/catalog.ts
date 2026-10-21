export interface Skill { slug: string; name: string; description: string; version: string; namespace?: string; iconUrl?: string | null; source?: string; removable?: boolean }
export interface SkillPage { results: Skill[]; page: number; total: number | null; hasMore: boolean }
export interface SkillReview {
  slug: string; namespace?: string; sha256: string; content: string;
  manifest: { version: string; fileCount: number; totalBytes: number; files: { path: string; size: number; sha256: string; executableScript: boolean }[] };
  scripts: { path: string; sha256: string; preview: string | null; previewTruncated: boolean }[];
}
