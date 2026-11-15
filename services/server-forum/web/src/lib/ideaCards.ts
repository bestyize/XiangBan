import type { RouteLocationRaw } from 'vue-router'

export interface IdeaCardItem {
  id: string
  title: string
  to: RouteLocationRaw
  description?: string | null
  cover?: string | null
  label?: string
  author?: string | null
  date?: string
  dateLabel?: string
}
