import type { AiTier } from "../bridge.ts";

/** Keeps the user's pick while it is still usable; otherwise the tier in use, the default tier, then the first usable one. */
export function pickTier(tiers: AiTier[], activeTier: string, current: string): string {
  const usable = tiers.filter(t => t.available);
  if (usable.some(t => t.tier === current)) return current;
  if (usable.some(t => t.tier === activeTier)) return activeTier;
  return (usable.find(t => t.isDefault) ?? usable[0])?.tier ?? "";
}

const kindLabels: Record<string, string> = { trial: "试用额度", monthly: "月卡", topup: "加油包" };

export const bucketLabel = (kind: string) => kindLabels[kind] ?? "额度";

/** Credit is always shown in US dollars: two decimals, three below $1 so one chat's change never reads as $0.00. */
export function money(points: number, perDollar: number): string {
  const dollars = points / Math.max(perDollar, 1);
  const abs = Math.abs(dollars);
  const min = abs >= 100 ? 0 : 2;
  const max = abs >= 100 ? 0 : abs >= 1 ? 2 : 3;
  const text = abs.toLocaleString("en-US", { minimumFractionDigits: min, maximumFractionDigits: max });
  return `${dollars < 0 ? "−" : ""}$${text}`;
}

export function expiryText(expiresAt: string | null, now = Date.now()): string {
  if (!expiresAt) return "长期有效";
  const time = new Date(expiresAt).getTime();
  if (Number.isNaN(time)) return "";
  const date = new Date(time).toLocaleDateString("zh-CN", { year: "numeric", month: "2-digit", day: "2-digit" });
  const days = Math.ceil((time - now) / 86_400_000);
  if (days <= 0) return `${date} 到期`;
  return days <= 3 ? `${date} 到期（还剩 ${days} 天）` : `${date} 到期`;
}
