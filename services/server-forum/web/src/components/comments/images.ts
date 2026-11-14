export function commentImageId(path: string): string | undefined {
  return /^\/api\/images\/([0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12})$/i.exec(path)?.[1]
}
