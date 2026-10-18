import { describe, expect, it } from "vitest";
import { bucketLabel, expiryText, money, pickTier } from "./ai-gateway.ts";
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
  it("shows credit in dollars with extra precision below one dollar", () => {
    expect(money(250_000, 500_000)).toBe("$0.50");
    expect(money(1_000, 500_000)).toBe("$0.002");
    expect(money(500_000, 500_000)).toBe("$1.00");
    expect(money(1_234_567, 500_000)).toBe("$2.47");
    expect(money(0, 500_000)).toBe("$0.00");
    expect(money(75_000_000, 500_000)).toBe("$150");
    expect(money(-500, 500_000)).toBe("−$0.001");
  });

  it("does not divide by zero when the server sends a broken rate", () => {
    expect(money(10, 0)).toBe("$10.00");
  });

  it("names the kinds the gateway hands out and falls back for new ones", () => {
    expect(bucketLabel("trial")).toBe("试用额度");
    expect(bucketLabel("monthly")).toBe("月卡");
    expect(bucketLabel("topup")).toBe("加油包");
    expect(bucketLabel("something-new")).toBe("额度");
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
