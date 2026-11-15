import type { RouteLocationRaw } from 'vue-router'
import { ideaRunLocation } from './ideaPlayers'
import { galleryAssets, rabbitExhibit } from './microGallery'

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

export const rabbitIdeaCard: IdeaCardItem = {
  id: rabbitExhibit.id,
  title: rabbitExhibit.name,
  description: rabbitExhibit.description,
  cover: galleryAssets + 'assets/gallery/' + rabbitExhibit.cover + '.webp',
  label: '3D模型',
  author: 'RealWorldBuilder',
  date: '2026-11-15',
  dateLabel: '收录',
  to: ideaRunLocation(rabbitExhibit.id),
}
