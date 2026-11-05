package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals

class AiCreditsTest {
    @Test
    fun `ten dollars is one thousand credits and fractional usage stays visible`() {
        listOf(
            5_000_000L to "1,000 点", 500_000L to "100 点", 250_000L to "50 点",
            62_500L to "12 点", 500L to "<1 点", 1L to "<1 点",
            1_234_567L to "246 点", 75_000_000L to "15,000 点",
            145_000L to "29 点", 4_999L to "<1 点", 5_000L to "1 点", 9_999L to "1 点",
            0L to "0 点", -500L to "−<1 点",
        ).forEach { (quota, expected) -> assertEquals(expected, formatAiCredits(quota)) }
    }

    @Test
    fun `configured rates and invalid rates match the other clients`() {
        assertEquals("1,000 点", formatAiCredits(10_000_000, 1_000_000))
        assertEquals("1,000 点", formatAiCredits(5_000_000, 0))
        assertEquals("1,000 点", formatAiCredits(5_000_000, -1))
        assertEquals("<1 点", formatAiCredits(1, 2_000_000))
    }
}
