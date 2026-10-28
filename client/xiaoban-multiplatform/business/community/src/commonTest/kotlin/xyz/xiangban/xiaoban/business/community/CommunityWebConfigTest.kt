package xyz.xiangban.xiaoban.business.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class CommunityWebConfigTest {
    @Test
    fun `loads the community site over https and keeps only its own host inside the shell`() {
        assertEquals("https://chat.xiangban.xyz", CommunityWebConfig.homeUrl)
        assertTrue(CommunityWebConfig.homeUrl.startsWith("https://"))
        assertEquals(setOf("chat.xiangban.xyz"), CommunityWebConfig.internalHosts)
    }
}
