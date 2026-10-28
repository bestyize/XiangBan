import unittest

from bandwidth_alert import Detector, mbps, parse_proc_net_dev

SAMPLE = """Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo:  100  1 0 0 0 0 0 0  100  1 0 0 0 0 0 0
  eth0: 5000 10 0 0 0 0 0 0 9000000 20 0 0 0 0 0 0
"""


class ProcNetDev(unittest.TestCase):
    def test_reads_receive_and_transmit_bytes(self):
        self.assertEqual(parse_proc_net_dev(SAMPLE, "eth0"), (5000, 9000000))
        self.assertEqual(parse_proc_net_dev(SAMPLE, "lo"), (100, 100))
        self.assertIsNone(parse_proc_net_dev(SAMPLE, "wlan0"))

    def test_mbps(self):
        self.assertAlmostEqual(mbps(25_000_000, 1), 200.0)
        self.assertAlmostEqual(mbps(25_000_000, 2), 100.0)
        self.assertEqual(mbps(1, 0), 0.0)


class Debounce(unittest.TestCase):
    def test_needs_consecutive_samples(self):
        d = Detector(0.7, consecutive=3, cooldown=100)
        self.assertIsNone(d.feed(0.9, 0))
        self.assertIsNone(d.feed(0.9, 20))
        self.assertIsNone(d.feed(0.5, 40), "回落前未告警，不发恢复")
        self.assertIsNone(d.feed(0.9, 60))
        self.assertIsNone(d.feed(0.9, 80))
        self.assertEqual(d.feed(0.9, 100), "alert")

    def test_cooldown_and_recovery(self):
        d = Detector(0.7, consecutive=1, cooldown=100)
        self.assertEqual(d.feed(0.8, 0), "alert")
        self.assertIsNone(d.feed(0.8, 50), "冷却期内不重复")
        self.assertEqual(d.feed(0.8, 120), "alert", "冷却期过后仍超阈值再提醒一次")
        self.assertEqual(d.feed(0.1, 140), "recovered")
        self.assertIsNone(d.feed(0.1, 160))


if __name__ == "__main__":
    unittest.main()
