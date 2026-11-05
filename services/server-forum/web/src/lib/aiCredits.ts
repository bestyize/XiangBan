/** Display only: $10 of gateway quota = 1,000 小伴点. The API still uses integer quota. */
export const AI_CREDITS_PER_DOLLAR = 100;

export function formatAiCredits(quota: number, quotaPerDollar = 500_000): string {
  const unit = Number.isFinite(quotaPerDollar) && quotaPerDollar > 0 ? quotaPerDollar : 500_000;
  const credits = quota / unit * AI_CREDITS_PER_DOLLAR;
  const value = Math.abs(credits);
  const sign = credits < 0 ? '−' : '';
  const text = value > 0 && value < 0.0001 ? '<0.0001'
    : value.toLocaleString('en-US', { maximumFractionDigits: 4 });
  return `${sign}${text} 点`;
}

export function aiBucketLabel(kind: string): string {
  return ({ trial: '试用点数', monthly: '月卡', topup: '加油包' } as Record<string, string>)[kind] ?? '点数';
}
