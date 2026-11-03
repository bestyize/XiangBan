export const projectPath = (name: string) => `/open-source/${name.split('/').map(encodeURIComponent).join('/')}`
export const collectionPath = (key: string) => `/open-source/collections/${encodeURIComponent(key)}`
