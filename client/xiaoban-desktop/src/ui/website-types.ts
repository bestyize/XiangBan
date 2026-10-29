export interface Notice { id: string; title: string; body: string; revision: string }
export interface Update { available: boolean; version: string; notes: string; publishedAt?: string; downloadUrl?: string }
export interface WebsiteStatus { announcements: Notice[]; update: Update | null; dismissed: Record<string, string>; errors: string[] }
