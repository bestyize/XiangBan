import { aquariumExhibit, galleryTab, rabbitExhibit } from './microGallery'

export const ideaPlayers = {
  'jiangshan-map': { title: '江山时序', entry: 'jiangshan-map/index.html', pointerLock: false, selfContained: false },
  saltline: { title: '运输船：Saltline', entry: 'saltline.html', pointerLock: true, selfContained: true },
  'starry-nightrider': { title: '星夜骑士：Starry Nightrider', entry: 'starry-nightrider.html', pointerLock: false, selfContained: true },
  'oil-paint': { title: 'Oil Paint', entry: 'oil-paint/index.html', pointerLock: true, selfContained: false },
  aquarium: { title: aquariumExhibit.name, entry: 'aquarium.html', pointerLock: false, selfContained: true },
  'plush-rabbit': { title: rabbitExhibit.name, entry: 'plush-rabbit.html', pointerLock: false, selfContained: true },
} as const

export type IdeaPlayerId = keyof typeof ideaPlayers

export function ideaPlayerId(value: unknown): IdeaPlayerId | undefined {
  if (typeof value === 'string' && Object.hasOwn(ideaPlayers, value)) return value as IdeaPlayerId
}

export function ideaRunLocation(slug: IdeaPlayerId) {
  return { name: 'idea-run', params: { slug } }
}

export function galleryRunLocation(exhibit: unknown) {
  if (exhibit === 'aquarium' || exhibit === 'plush-rabbit') return ideaRunLocation(exhibit)
}

export function ideaReturnLocation(id: IdeaPlayerId | undefined, hasWorkDetails: boolean) {
  if (galleryRunLocation(id)) return { name: 'home', query: { tab: galleryTab.key } }
  if (id && hasWorkDetails) return { name: 'idea-work', params: { slug: id } }
  return { name: 'home' }
}
