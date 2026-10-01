package xyz.xiaban.server.unlocks

import io.ktor.client.call.body
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.ProfileUpdate
import xyz.xiaban.shared.TitleCode
import xyz.xiaban.shared.UnlocksResponse
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class UnlockFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `rules cover milestones, titles, weekdays and comebacks`() {
        val base = UnlockContext(LocalDate.parse("2026-10-12"), TitleCode.OnTime, 7, 3, "rain", null, 9, previousWasCare = true)
        val earned = UnlockRules.earned(base)
        assertTrue(earned.containsAll(listOf("hello", "days_3", "days_7", "streak_3", "title_on_time", "monday", "rainy", "comeback", "redemption")))
        assertTrue("days_14" !in earned && "streak_5" !in earned)
        assertEquals(30, UnlockCatalog.all.size)
        assertTrue(UnlockCatalog.all.map { it.id }.containsAll(TitleCode.entries.map(UnlockCatalog::forTitle)))
    }

    @Test
    fun `first check-in unlocks once and the collection shows progress`() = apiTest {
        val token = session().token
        put(ApiPaths.PROFILE, token, ProfileUpdate(onboarded = true))
        clock.setBeijing("2026-10-12T21:30")
        val first = post(ApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals(listOf("hello", "title_overtime_hero", "monday"), first.newUnlocks.map { it.id })

        clock.setBeijing("2026-10-13T21:30")
        val second = post(ApiPaths.CHECKIN, token).body<CheckinView>()
        assertTrue(second.newUnlocks.none { it.id == "hello" || it.id == "title_overtime_hero" })

        val dex = get(ApiPaths.UNLOCKS, token).body<UnlocksResponse>()
        assertEquals(75, dex.appTotal)
        assertEquals(30, dex.webTotal)
        assertEquals(4, dex.unlockedCount, "named, hello, overtime hero, monday")
        assertTrue(dex.items.first { it.id == "named" }.unlockedAt != null)
    }
}
