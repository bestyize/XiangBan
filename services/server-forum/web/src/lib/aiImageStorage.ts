export interface StoredAiImage { id: string; owner: string; prompt: string; created: number; blob: Blob }
function database(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('xiangban-ai-images', 1)
    request.onupgradeneeded = () => { request.result.createObjectStore('images', {keyPath:'id'}).createIndex('owner', 'owner') }
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}
export async function readAiImages(owner: string): Promise<StoredAiImage[]> {
  const db = await database()
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction('images'), request = tx.objectStore('images').index('owner').getAll(owner)
      request.onsuccess = () => resolve((request.result as StoredAiImage[]).sort((a,b) => b.created - a.created))
      request.onerror = () => reject(request.error)
    })
  } finally {db.close()}
}
export async function storeAiImages(owner: string, images: StoredAiImage[]): Promise<void> {
  const db = await database()
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction('images', 'readwrite'), store = tx.objectStore('images'), request = store.index('owner').getAll(owner)
      request.onsuccess = () => {
        const all = [...images, ...request.result as StoredAiImage[]].sort((a,b) => b.created - a.created)
        let bytes = 0
        all.forEach((image, index) => {
          bytes += image.blob.size
          if (index >= 10 || bytes > 96 * 1024 * 1024) store.delete(image.id)
          else store.put(image)
        })
      }
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
      tx.onabort = () => reject(tx.error)
    })
  } finally {db.close()}
}
