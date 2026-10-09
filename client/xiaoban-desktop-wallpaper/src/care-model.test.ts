import { describe, expect, it } from "vitest";
import { careRuleError, deliverySummary, describeCareRule, newCareRule, weatherIsFresh, type CareWeather, type CareDelivery } from "./care-model.ts";

describe("care rule editing", () => {
  it("does not infer working weekdays", () => {
    const rule = newCareRule("off-work"); rule.repeat = "weekdays";
    expect(careRuleError(rule)).toContain("星期");
    rule.weekdays = [7, 2, 2]; expect(describeCareRule(rule)).toBe("周二、周日 18:00");
  });
  it("requires custom text but lets weather generate its own factual message", () => {
    const rule = newCareRule(); rule.text = " "; expect(careRuleError(rule)).toContain("文案");
    const weather = newCareRule("weather"); expect(careRuleError(weather)).toContain("城市");
    weather.city = { name: "北京", admin1: "北京", country: "中国", latitude: 39.9, longitude: 116.4 }; expect(careRuleError(weather)).toBeNull();
  });
  it("rejects out-of-range intervals and clock times", () => {
    const rule = newCareRule("rest"); rule.intervalMinutes = 4; expect(careRuleError(rule)).toContain("5–480");
    const timed = newCareRule(); timed.time = "24:00"; expect(careRuleError(timed)).toContain("时间");
  });
  it("validates one-off dates and keeps the selected date readable", () => {
    const rule = newCareRule(); rule.repeat = "once"; expect(careRuleError(rule)).toContain("日期");
    rule.onceAt = "2026-10-10T15:00"; expect(careRuleError(rule)).toBeNull(); expect(describeCareRule(rule)).toBe("2026-10-10 15:00");
  });
});
describe("weather freshness and consolidated deliveries", () => {
  const weather: CareWeather = { city: { name: "北京", country: "中国", admin1: "", latitude: 39.9, longitude: 116.4 }, updatedAt: 10000, modelAt: 9800, source: "fixture", text: "晴" };
  it("labels old or future fetched data unavailable", () => {
    expect(weatherIsFresh(weather, 10060)).toBe(true);
    expect(weatherIsFresh(weather, 11800)).toBe(false);
    expect(weatherIsFresh(weather, 9999)).toBe(false);
    expect(weatherIsFresh({ ...weather, modelAt: 2000 }, 10060)).toBe(false);
  });
  it("consolidates a backlog into a bounded summary", () => {
    const delivery = { items: [{ text: "喝水" }, { text: "下班" }, { text: "看天气" }] } as CareDelivery;
    expect(deliverySummary(delivery)).toBe("喝水\n下班\n还有 1 条到期提醒。");
  });
});
