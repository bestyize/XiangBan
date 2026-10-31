package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull

class PollDraftTest {
    @Test fun whitespaceAndDuplicatesMatchServerRules() {
        assertEquals(listOf("甲", "乙"), parsePollOptions(" 甲 \n\n 乙\n"))
        assertNotNull(pollDraftError(parsePollOptions("甲\n 甲 "), null))
        assertNull(pollDraftError(listOf("A", "a"), null))
    }
    @Test fun countAndExpiryBoundariesAreValidated() {
        assertNotNull(pollDraftError(listOf("甲"), null))
        assertNotNull(pollDraftError((1..7).map { "$it" }, null))
        assertNotNull(pollDraftError(listOf("甲", "乙"), 0))
        assertNull(pollDraftError(listOf("甲", "乙"), 30))
        assertNotNull(pollDraftError(listOf("甲", "乙"), 31))
    }
    @Test fun emojiUsesCharacterLimitRatherThanUtf16Units() {
        assertNull(pollDraftError(listOf("😀".repeat(40), "乙"), 7))
        assertNotNull(pollDraftError(listOf("😀".repeat(41), "乙"), 7))
    }
}
