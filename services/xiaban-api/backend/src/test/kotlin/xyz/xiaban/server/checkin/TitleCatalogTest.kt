package xyz.xiaban.server.checkin

import kotlin.test.Test
import kotlin.test.assertEquals
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.companion.personality.Mood

class TitleCatalogTest {
    @Test
    fun extractedCatalogPreservesEveryExistingTitle() {
        val expected = listOf(
            TitleDefinition("准点之神", true, Mood.Sparkle),
            TitleDefinition("周末还上班？？", true, Mood.Shocked),
            TitleDefinition("被借走的周末", true, Mood.Aggrieved),
            TitleDefinition("放假倒计时", true, Mood.Excited),
            TitleDefinition("早退王者", false, Mood.Smug),
            TitleDefinition("准点战神", false, Mood.Happy),
            TitleDefinition("正常人类", false, Mood.Welcome),
            TitleDefinition("加班侠", false, Mood.Heartache),
            TitleDefinition("夜行动物", false, Mood.Sleepy),
            TitleDefinition("修仙者", false, Mood.Worried),
        )
        assertEquals(expected, TitleCode.entries.map { TitleCatalog.definition(it) })
    }
}
