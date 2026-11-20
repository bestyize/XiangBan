import type { RouteLocationRaw } from 'vue-router'

export interface IdeaCardItem {
  id: string
  title: string
  to: RouteLocationRaw
  description?: string | null
  cover?: string | null
  fallbackCover?: string
  label?: string
  author?: string | null
  avatarColor?: string
  date?: string
}
