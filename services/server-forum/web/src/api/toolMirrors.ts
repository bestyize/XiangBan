import { request as adminRequest } from './admin'
export interface MirrorArtifact {
  asset_id: number
  filename: string
  os: string
  arch: string
  size: number
  sha256: string
  github_url: string
  verified_at: string | null
}
export interface MirrorTool {
  key: string
  name: string
  version: string
  status: string
  revision: number
  artifacts: MirrorArtifact[]
}
export interface MirrorList {
  items: MirrorTool[]
  invalid_items: number
  resource_configured: boolean
  max_asset_bytes: number
  max_storage_bytes: number
  max_storage_objects: number
}
export const toolMirrorsApi = {
  list: () => adminRequest<MirrorList>('GET', '/tool-mirrors'),
  create: (tool: string, assetId: number, expectedRevision: number) => adminRequest<{ status: 'verified'; verified_at: string; published: false }>('POST', `/tool-mirrors/${encodeURIComponent(tool)}/${assetId}`, { expected_revision: expectedRevision }),
}
