import { describe, expect, it } from "vitest";
import { bucketLabel, expiryText, formatAiCredits, pickTier } from "./ai-gateway.ts";
import type { AiTier } from "../bridge.ts";

const tier = (name: string, patch: Partial<AiTier> = {}): AiTier => ({ tier: name, displayName: name, description: "", isDefault: false, requiresPaid: false, available: true, ...patch });

describe("picking a tier", () => {
  const list = [tier("standard", { isDefault: true }), tier("premium", { requiresPaid: true, available: false }), tier("max")];

  it("keeps the current pick while it is still usable", () => {
    expect(pickTier(list, "standard", "max")).toBe("max");
  });

  it("falls back to the tier in use, then the default, then the first usable one", () => {
    expect(pickTier(list, "max", "premium")).toBe("max");
    expect(pickTier(list, "", "")).toBe("standard");
    expect(pickTier([tier("a"), tier("b")], "", "")).toBe("a");
  });

  it("picks nothing when no tier can be used", () => {
    expect(pickTier([tier("premium", { available: false })], "premium", "premium")).toBe("");
    expect(pickTier([], "", "")).toBe("");
  });
});

describe("ai gateway formatting", () => {
  it("shows 1,000 credits for $10 and preserves small charges", () => {
    expect(formatAiCredits(5_000_000)).toBe("1,000 点");
    expect(formatAiCredits(250_000)).toBe("50 点");
    expect(formatAiCredits(500_000)).toBe("100 点");
    expect(formatAiCredits(500)).toBe("<1 点");
    expect(formatAiCredits(1)).toBe("<1 点");
    expect(formatAiCredits(1_234_567)).toBe("246 点");
    expect(formatAiCredits(145_000)).toBe("29 点");
    expect(formatAiCredits(4_999)).toBe("<1 点");
    expect(formatAiCredits(5_000)).toBe("1 点");
    expect(formatAiCredits(9_999)).toBe("1 点");
    expect(formatAiCredits(0)).toBe("0 点");
    expect(formatAiCredits(75_000_000)).toBe("15,000 点");
    expect(formatAiCredits(-500)).toBe("−<1 点");
  });

  it("uses the server rate with a safe default for invalid or missing rates", () => {
    expect(formatAiCredits(10_000_000, 1_000_000)).toBe("1,000 点");
    expect(formatAiCredits(5_000_000, 0)).toBe("1,000 点");
    expect(formatAiCredits(5_000_000, NaN)).toBe("1,000 点");
    expect(formatAiCredits(5_000_000, -1)).toBe("1,000 点");
    expect(formatAiCredits(1, 2_000_000)).toBe("<1 点");
  });

  it("names the kinds the gateway hands out and falls back for new ones", () => {
    expect(bucketLabel("trial")).toBe("试用点数");
    expect(bucketLabel("monthly")).toBe("月卡");
    expect(bucketLabel("topup")).toBe("加油包");
    expect(bucketLabel("something-new")).toBe("点数");
  });

  it("warns only in the last three days before an expiry", () => {
    const now = Date.parse("2026-10-18T00:00:00Z");
    expect(expiryText(null, now)).toBe("长期有效");
    expect(expiryText("2026-10-20T00:00:00Z", now)).toMatch(/还剩 2 天/);
    expect(expiryText("2026-11-18T00:00:00Z", now)).toMatch(/到期$/);
    expect(expiryText("2026-10-17T00:00:00Z", now)).toMatch(/到期$/);
    expect(expiryText("not a date", now)).toBe("");
  });
});
