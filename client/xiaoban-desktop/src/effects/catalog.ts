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

/** What the host reports after a pack file was offered to the effects folder. */
export type ImportOutcome =
  /** The pack was saved; `id` is also the file name it landed under. */
  | { status: "imported"; id: string; name: string; file: string }
  /** The user closed the picker, or said no when asked about replacing a pack with the same id. */
  | { status: "cancelled" }
  | { status: "declined"; id: string; existing: string };

/** Reads the host's reply to an import. Anything unrecognised counts as nothing having happened. */
export function readImported(raw: unknown): ImportOutcome {
  const value = (typeof raw === "object" && raw !== null ? raw : {}) as { status?: unknown; id?: unknown; name?: unknown; file?: unknown; existing?: unknown };
  if (value.status === "declined" && typeof value.id === "string") return { status: "declined", id: value.id, existing: typeof value.existing === "string" ? value.existing : `${value.id}.json` };
  if (value.status !== "imported" || typeof value.id !== "string" || !value.id) return { status: "cancelled" };
  return { status: "imported", id: value.id, name: typeof value.name === "string" && value.name ? value.name : value.id, file: typeof value.file === "string" && value.file ? value.file : `${value.id}.json` };
}

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
