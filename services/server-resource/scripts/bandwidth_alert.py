#!/usr/bin/env python3
"""资源机带宽告警：按固定间隔读网卡收发字节数，出口带宽连续超过阈值就发 webhook。

用法（systemd 常驻或 cron 每分钟一次都行，这里常驻）：
  LINK_MBPS=200 ALERT_WEBHOOK=https://hooks.example/xxx python3 scripts/bandwidth_alert.py --iface eth0

默认阈值 70%，连续 3 次采样（间隔 20 秒）都超过才告警，告警后 30 分钟内不重复；回落到阈值以下会发一条恢复通知。
"""
import argparse
import json
import os
import sys
import time
import urllib.request


def parse_proc_net_dev(text: str, iface: str):
    """从 /proc/net/dev 的内容取某个网卡的 (收字节, 发字节)；找不到返回 None。"""
    for line in text.splitlines():
        name, sep, rest = line.partition(":")
        if sep and name.strip() == iface:
            fields = rest.split()
            return int(fields[0]), int(fields[8])
    return None


def mbps(delta_bytes: int, seconds: float) -> float:
    return delta_bytes * 8 / seconds / 1_000_000 if seconds > 0 else 0.0


class Detector:
    """连续超过阈值才告警（防抖），告警后有冷却期，回落时给一次恢复通知。"""

    def __init__(self, threshold: float, consecutive: int = 3, cooldown: float = 1800.0):
        self.threshold, self.consecutive, self.cooldown = threshold, consecutive, cooldown
        self.over = 0
        self.alerting = False
        self.last_alert = float("-inf")

    def feed(self, ratio: float, now: float):
        """返回 None、'alert' 或 'recovered'。"""
        if ratio >= self.threshold:
            self.over += 1
            if self.over >= self.consecutive and (not self.alerting or now - self.last_alert >= self.cooldown):
                self.alerting, self.last_alert = True, now
                return "alert"
            return None
        self.over = 0
        if self.alerting:
            self.alerting = False
            return "recovered"
        return None


def notify(webhook: str, text: str):
    request = urllib.request.Request(webhook, data=json.dumps({"text": text}).encode(), headers={"content-type": "application/json"})
    urllib.request.urlopen(request, timeout=10).read()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--iface", default=os.environ.get("ALERT_IFACE", "eth0"))
    parser.add_argument("--interval", type=float, default=20.0)
    parser.add_argument("--threshold", type=float, default=0.7)
    args = parser.parse_args()
    link = float(os.environ.get("LINK_MBPS", "200"))
    webhook = os.environ.get("ALERT_WEBHOOK", "")
    detector = Detector(args.threshold)

    def sample():
        with open("/proc/net/dev") as f:
            return parse_proc_net_dev(f.read(), args.iface)

    previous, at = sample(), time.monotonic()
    if previous is None:
        sys.exit(f"找不到网卡 {args.iface}")
    while True:
        time.sleep(args.interval)
        current, now = sample(), time.monotonic()
        if current is None:
            continue
        out = mbps(current[1] - previous[1], now - at)
        previous, at = current, now
        ratio = out / link
        event = detector.feed(ratio, now)
        print(f"出口 {out:.1f} Mbps（{ratio:.0%}）", flush=True)
        if event and webhook:
            text = f"资源机出口带宽 {out:.0f} Mbps，占 {link:.0f} Mbps 的 {ratio:.0%}" if event == "alert" else "资源机出口带宽已回落"
            try:
                notify(webhook, text)
            except OSError as error:
                print(f"告警发送失败：{error}", file=sys.stderr)


if __name__ == "__main__":
    main()
