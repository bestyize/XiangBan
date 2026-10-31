package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class TimeTextTest {
    private val now = parseIsoMillis("2026-10-31T10:00:00Z")!!

    @Test
    fun `iso times parse with fractions and offsets`() {
        assertEquals(0L, parseIsoMillis("1970-01-01T00:00:00Z"))
        assertEquals(1_700_000_000_000L, parseIsoMillis("2023-11-14T22:13:20Z"))
        assertEquals(1_700_000_000_123L, parseIsoMillis("2023-11-14T22:13:20.123456Z"))
        assertEquals(parseIsoMillis("2023-11-14T22:13:20Z"), parseIsoMillis("2023-11-15T06:13:20+08:00"))
        assertEquals(parseIsoMillis("2024-02-29T00:00:00Z")!! + 86_400_000L, parseIsoMillis("2024-03-01T00:00:00Z"))
        assertNull(parseIsoMillis("not a time"))
        assertNull(parseIsoMillis(""))
    }

    @Test
    fun `ago text uses minutes, hours, yesterday and dates`() {
        assertEquals("刚刚", agoText("2026-10-31T09:59:40Z", now))
        assertEquals("5 分钟前", agoText("2026-10-31T09:55:00Z", now))
        assertEquals("3 小时前", agoText("2026-10-31T07:00:00Z", now))
        assertEquals("昨天", agoText("2026-10-30T02:00:00Z", now))
        assertEquals("10 月 20 日", agoText("2026-10-20T02:00:00Z", now))
        assertEquals("2025 年 12 月 3 日", agoText("2025-12-03T02:00:00Z", now))
        assertEquals("刚刚", agoText("2026-10-31T10:05:00Z", now), "时钟略有偏差时不出现负数")
        assertEquals("garbage", agoText("garbage", now))
    }

    @Test
    fun `countdown, clock and remaining text`() {
        assertEquals("3 小时 12 分", countdownText("2026-10-31T13:12:00Z", now))
        assertEquals("45 分", countdownText("2026-10-31T10:45:00Z", now))
        assertEquals("0 分", countdownText("2026-10-31T09:00:00Z", now))
        assertEquals("今天 21:12", clockText("2026-10-31T13:12:00Z", now))
        assertEquals("明天 09:00", clockText("2026-11-01T01:00:00Z", now))
        assertEquals("11 月 5 日 09:00", clockText("2026-11-05T01:00:00Z", now))
        assertEquals("剩 5 天", remainText("2026-11-05T10:00:00Z", now))
        assertEquals("剩 5 天", remainText("2026-11-04T12:00:00Z", now))
        assertEquals("剩 3 小时", remainText("2026-10-31T13:00:00Z", now))
        assertEquals("已结束", remainText("2026-10-31T09:00:00Z", now))
    }

    @Test
    fun `counts are compact`() {
        assertEquals("88", compactCount(88))
        assertEquals("4.1千", compactCount(4100))
        assertEquals("1.2万", compactCount(12_000))
    }
}
