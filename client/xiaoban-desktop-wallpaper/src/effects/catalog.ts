import { parsePack } from "./parse.ts";
import type { EffectPack } from "./types.ts";

export interface LoadedPacks {
  /** Packs that passed the check, built-in ones first. */
  packs: EffectPack[];
  /** One line for each pack that could not be used, naming it and saying why. */
  errors: string[];
  /** The folder where the user's own packs go (empty when running in a browser). */
  directory: string;
}
export const noPacks = (): LoadedPacks => ({ packs: [], errors: [], directory: "" });

/** Checks what the host listed (`{packs, errors, directory}`) and keeps the usable packs. */
export function readList(raw: unknown): LoadedPacks {
  const list = typeof raw === "object" && raw !== null ? (raw as { packs?: unknown; errors?: unknown; directory?: unknown }) : {};
  const result: LoadedPacks = { packs: [], errors: Array.isArray(list.errors) ? list.errors.filter((e): e is string => typeof e === "string") : [], directory: typeof list.directory === "string" ? list.directory : "" };
  for (const item of Array.isArray(list.packs) ? list.packs : []) {
    const parsed = parsePack(item);
    if (parsed.ok) result.packs.push(parsed.pack);
    else result.errors.push(`${(item as { name?: unknown })?.name ?? (item as { id?: unknown })?.id ?? "未命名特效"}：${parsed.error}`);
  }
  return result;
}

/** The built-in packs as files of the app, for the browser preview where there is no host to ask. */
export async function loadBuiltinPacks(fetcher: typeof fetch = fetch): Promise<LoadedPacks> {
  const catalog = (await (await fetcher("/effects/catalog.json")).json()) as { effects?: unknown };
  const ids = Array.isArray(catalog.effects) ? catalog.effects.filter((id): id is string => typeof id === "string" && /^[a-z0-9_-]+$/.test(id)) : [];
  const packs = await Promise.all(ids.map(async (id) => {
    const response = await fetcher(`/effects/${id}.json`);
    return response.ok ? { ...(await response.json()), source: "builtin" } : null;
  }));
  return readList({ packs: packs.filter(Boolean) });
}
