import type { AiTier } from "../bridge.ts";

/** Keeps the user's pick while it is still usable; otherwise the tier in use, the default tier, then the first usable one. */
export function pickTier(tiers: AiTier[], activeTier: string, current: string): string {
  const usable = tiers.filter(t => t.available);
  if (usable.some(t => t.tier === current)) return current;
  if (usable.some(t => t.tier === activeTier)) return activeTier;
  return (usable.find(t => t.isDefault) ?? usable[0])?.tier ?? "";
}

const kindLabels: Record<string, string> = { trial: "试用点数", monthly: "月卡", topup: "加油包" };

export const bucketLabel = (kind: string) => kindLabels[kind] ?? "点数";

/** Display only: $10 of gateway quota = 1,000 小伴点. The API still uses integer quota. */
export const AI_CREDITS_PER_DOLLAR = 100;

export function formatAiCredits(quota: number, quotaPerDollar = 500_000): string {
  const unit = Number.isFinite(quotaPerDollar) && quotaPerDollar > 0 ? quotaPerDollar : 500_000;
  const credits = quota / (unit / AI_CREDITS_PER_DOLLAR);
  const value = Math.abs(credits);
  const sign = credits < 0 ? "−" : "";
  // Truncate the display only; never overstate a balance or hide a nonzero charge as zero.
  const text = value > 0 && value < 1 ? "<1" : Math.floor(value).toLocaleString("en-US");
  return `${sign}${text} 点`;
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
