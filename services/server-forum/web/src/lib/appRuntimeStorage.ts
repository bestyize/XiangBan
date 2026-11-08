const encoder = new TextEncoder()
const quota = 10 * 1024 * 1024
let database: Promise<IDBDatabase> | undefined

function openDatabase() {
  database ??= new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open('xiaoban-app-runtime', 1)
    request.onupgradeneeded = () => request.result.createObjectStore('values')
    request.onerror = () => reject(request.error)
    request.onsuccess = () => resolve(request.result)
  }).catch(error => {
    database = undefined
    throw error
  })
  return database
}

export async function appRuntimeStorage(packId: number, operation: unknown, key: unknown, value: unknown): Promise<unknown> {
  if (operation !== 'get' && operation !== 'set' && operation !== 'remove' && operation !== 'keys') throw new Error('invalid')
  if (operation !== 'keys' && (typeof key !== 'string' || !key || [...key].length > 128)) throw new Error('invalid')
  const entry: [number, string] = [packId, typeof key === 'string' ? key : '']
  const range = IDBKeyRange.bound([packId], [packId + 1], false, true)
  let json: string | undefined
  if (operation === 'set') {
    try { json = JSON.stringify(value) } catch { throw new Error('invalid') }
    if (json === undefined) throw new Error('invalid')
    if (encoder.encode(json).length + encoder.encode(entry[1]).length > quota) throw new Error('quota')
  }
  const db = await openDatabase()
  return new Promise((resolve, reject) => {
    const tx = db.transaction('values', operation === 'set' || operation === 'remove' ? 'readwrite' : 'readonly')
    const store = tx.objectStore('values')
    let result: unknown = null
    let error: Error | undefined
    tx.oncomplete = () => resolve(result)
    tx.onabort = () => reject(error ?? tx.error)
    switch (operation) {
      case 'get': {
        const request = store.get(entry)
        request.onsuccess = () => { if (request.result !== undefined) result = JSON.parse(request.result) }
        break
      }
      case 'keys': {
        const request = store.getAllKeys(range)
        request.onsuccess = () => { result = request.result.map(item => (item as [number, string])[1]) }
        break
      }
      case 'remove':
        store.delete(entry)
        break
      case 'set': {
        const text = json!
        let bytes = encoder.encode(text).length + encoder.encode(entry[1]).length
        const request = store.openCursor(range)
        request.onsuccess = () => {
          const row = request.result
          if (row) {
            const existing = (row.key as [number, string])[1]
            if (existing !== key) bytes += encoder.encode(existing).length + encoder.encode(row.value).length
            row.continue()
          } else if (bytes > quota) {
            error = new Error('quota')
            tx.abort()
          } else {
            store.put(text, entry)
          }
        }
      }
    }
  })
}
