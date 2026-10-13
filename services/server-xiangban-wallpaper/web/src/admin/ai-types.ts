export interface AiModel { id: string; name: string; description: string; enabled: boolean }
export interface AiAccess { uid: string; apikey?: string; hasKey?: boolean }
export interface AiProvider {
  id: string; provider: string; description: string; endPoint: string; protocol: string;
  type: 'third' | 'free' | 'paid'; enabled: boolean; defaultModel: string; models: AiModel[];
  apikey?: string; hasKey?: boolean; clearKey?: boolean;
  endpoints: Record<string, string>; consoleUrl: string; consoleText: string; userAccess: AiAccess[];
}
export interface AiCatalogue { version: number; revision: number; refreshAfterSeconds: number; providers: AiProvider[] }
