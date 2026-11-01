export interface SkinLayer { id: string; image: string; depth: number; anchor: string; interactive?: boolean }
export interface SkinInteraction { target: string; trigger: string; action: string; label: string; repeat?: string }
export interface SkinSceneData { layers: SkinLayer[]; interactions: SkinInteraction[] }
