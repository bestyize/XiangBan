export const galleryTab = { key: 'gallery', label: '微观展厅' }
export const galleryAssets = '/gallery-exhibits/'

export const aquariumExhibit = {
  id: 'aquarium',
  name: '水族箱',
  description: '看龙鱼和孔雀鱼在山水客厅共游，切换鱼类、背景和灯光。',
  cover: 'living-room',
  source: 'aquarium.html',
  query: '?scene=living-room',
} as const

export const rabbitExhibit = {
  id: 'plush-rabbit',
  name: '碎花垂耳兔',
  description: '转动 3D 模型，近看绒毛、碎花和蝴蝶结。',
  cover: 'rabbit',
  source: 'objects/plush-rabbit/preview.html',
  query: '',
} as const

export const galleryExhibits = [
  aquariumExhibit,
  {
    id: 'arowana',
    name: '龙鱼',
    description: '近看鳞片反光与鱼身转向。',
    cover: 'arowana',
    source: 'fish/arowana/preview.html',
    query: '',
  },
  {
    id: 'blue-guppy',
    name: '蓝扇尾孔雀鱼',
    description: '观察尾鳍如何随水摆动。',
    cover: 'guppy',
    source: 'fish/blue-guppy/preview.html',
    query: '',
  },
  {
    id: 'guppy-school',
    name: '孔雀鱼群',
    description: '六条孔雀鱼，四种花色。',
    cover: 'school',
    source: 'fish/blue-guppy/school-preview.html',
    query: '',
  },
  rabbitExhibit,
] as const
