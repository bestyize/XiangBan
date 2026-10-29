import { describe, expect, it } from "vitest";
import { careRuleError, deliverySummary, describeAgentRun, describeCareRule, newCareRule, weatherIsFresh, type CareWeather, type CareDelivery } from "./care-model.ts";

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
  it("lets a scheduled agent task carry a longer request but never an empty one", () => {
    const task = newCareRule("agent"); expect(careRuleError(task)).toBeNull();
    task.text = "看".repeat(1000); expect(careRuleError(task)).toBeNull();
    task.text = "看".repeat(1001); expect(careRuleError(task)).not.toBeNull();
    task.text = " "; expect(careRuleError(task)).not.toBeNull();
    const plain = newCareRule(); plain.text = "喝".repeat(241); expect(careRuleError(plain)).not.toBeNull();
  });
  it("describes a scheduled task's last run with its cost", () => {
    const now = new Date(2026, 9, 22, 12, 0);
    const run = { at: new Date(2026, 9, 22, 8, 5).getTime() / 1000, ok: true, summary: "", session: "s", inputTokens: 12_400, outputTokens: 1_250, count: 3 };
    expect(describeAgentRun(run, now)).toBe("上次运行：今天 08:05 · 已完成 · 输入 12k / 输出 1.3k token · 共 3 次");
    expect(describeAgentRun({ ...run, ok: false, inputTokens: 0, outputTokens: 0, at: new Date(2026, 9, 20, 8, 0).getTime() / 1000 }, now)).toBe("上次运行：10月20日 08:00 · 没有完成 · 共 3 次");
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
