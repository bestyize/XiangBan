package xyz.xiaban.server.copy

import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.companion.personality.Personality

class CopySelectorTest {
    private val library = CopyLibrary.load()
    private val selector = CopySelector(library, Random(42))
    private val base = CopyContext(personality = Personality.Toxic, title = TitleCode.OvertimeHero, weekday = 2)

    @Test
    fun `library covers every personality, title and mode`() {
        assertEquals(library.entries.size, library.entries.map { it.id }.toSet().size, "ids are unique")
        assertTrue(library.entries.size >= 500, "PRD asks for an initial library of 500 lines")
        for (p in Personality.entries) {
            for (t in TitleCode.entries) {
                assertTrue(library.entries.any { it.personality == p && it.mode == CopyMode.Normal && it.title == t }, "$p/$t")
            }
            for (m in CopyMode.entries) assertTrue(library.entries.any { it.personality == p && it.mode == m }, "$p/$m")
        }
        library.entries.filter { "{n}" in it.text }.forEach { assertNotNull(it.streakMin, it.text) }
    }

    @Test
    fun `plain day picks a line for today's title and personality`() {
        repeat(50) {
            val e = selector.pickReply(base, emptySet(), emptySet())!!
            assertEquals(Personality.Toxic, e.personality)
            assertEquals(TitleCode.OvertimeHero, e.title)
        }
    }

    @Test
    fun `conditional lines join the pool only when their condition holds`() {
        val rainy = (1..200).map { selector.pickReply(base.copy(weather = "rain"), emptySet(), emptySet())!! }
        assertTrue(rainy.any { it.weather == "rain" })
        assertTrue(rainy.all { it.weather == null || it.weather == "rain" })

        val streaky = (1..200).map { selector.pickReply(base.copy(overtimeStreak = 3), emptySet(), emptySet())!! }
        assertTrue(streaky.any { it.streakMin != null })
        val calm = (1..200).map { selector.pickReply(base.copy(overtimeStreak = 1), emptySet(), emptySet())!! }
        assertTrue(calm.none { it.streakMin != null })
    }

    @Test
    fun `care mode switches to care lines`() {
        val e = selector.pickReply(base.copy(title = TitleCode.NightOwl, careMode = true), emptySet(), emptySet())!!
        assertEquals(CopyMode.Care, e.mode)
    }

    @Test
    fun `recently used and disabled lines are avoided`() {
        val titleLines = library.entries.filter { it.personality == Personality.Toxic && it.mode == CopyMode.Normal && it.title == TitleCode.OvertimeHero }
        val keep = titleLines.first()
        val used = titleLines.drop(1).map { it.id }.toSet()
        repeat(20) { assertEquals(keep.id, selector.pickReply(base, used, emptySet())!!.id) }
        repeat(20) { assertTrue(selector.pickReply(base, emptySet(), setOf(keep.id))!!.id != keep.id) }
    }

    @Test
    fun `greetings are stable for the same seed`() {
        val a = selector.pickGreeting(Personality.Gentle, CopyMode.Idle, Daypart.Night, 7, emptySet())!!
        val b = selector.pickGreeting(Personality.Gentle, CopyMode.Idle, Daypart.Night, 7, emptySet())!!
        assertEquals(a.id, b.id)
        assertEquals(Daypart.Night, a.daypart)
    }

    @Test
    fun `placeholders are filled`() {
        assertEquals("你连续 3 天加班", CopySelector.render("{name}连续 {n} 天加班", null, 3))
        assertEquals("阿明好", CopySelector.render("{name}好", "阿明", 0))
    }
}
