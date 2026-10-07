package xyz.xiaban.server.users

import xyz.xiaban.server.http.ApiException
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

class TextRulesTest {
    @Test
    fun `trims and collapses whitespace`() {
        assertEquals("大 橘", TextRules.companionName("  大   橘 "))
    }

    @Test
    fun `enforces length in characters, not bytes`() {
        assertEquals("八个字的名字好长", TextRules.companionName("八个字的名字好长"))
        assertFailsWith<ApiException> { TextRules.companionName("九个字的名字真的好长") }
        assertFailsWith<ApiException> { TextRules.companionName("猫") }
    }

    @Test
    fun `rejects contact details and links`() {
        listOf("加我微信", "qq12345", "13800138000", "a@b", "www.x").forEach {
            assertFailsWith<ApiException>(it) { TextRules.nickname(it) }
        }
    }

    @Test
    fun `blank nickname clears it`() {
        assertNull(TextRules.nickname("   "))
    }
}
