package xyz.xiaban.server.checkin

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import java.time.Duration
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.contracts.afterwork.checkin.CheckinView
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.afterwork.checkin.TodayResponse
import xyz.xiaban.contracts.companion.personality.Mood
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

class CheckinFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `checking in returns the title right away and repeats are idempotent`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-09-29T21:37")
        val first = post(CheckinApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals(TitleCode.OvertimeHero, first.title.code)
        assertEquals("加班侠", first.title.name)
        assertEquals(Mood.Heartache, first.mood)
        assertEquals("21:37", first.localTime)
        assertEquals("2026-09-29", first.workDay)
        assertEquals(first.checkedAt + Duration.ofMinutes(10).toMillis(), first.undoUntil)

        clock.advance(Duration.ofHours(1))
        val again = post(CheckinApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals(first.id, again.id)
        assertNull(again.undoUntil, "undo window has passed")

        val today = get(CheckinApiPaths.CHECKIN_TODAY, token).body<TodayResponse>()
        assertEquals(first.id, today.checkin?.id)
    }

    @Test
    fun `undo works once within ten minutes`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-09-29T17:40")
        val early = post(CheckinApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals(TitleCode.EarlyLeaver, early.title.code)

        clock.advance(Duration.ofMinutes(9))
        val undone = delete(CheckinApiPaths.CHECKIN_TODAY, token).body<TodayResponse>()
        assertNull(undone.checkin)

        clock.setBeijing("2026-09-29T18:10")
        val redo = post(CheckinApiPaths.CHECKIN, token).body<CheckinView>()
        assertNotEquals(early.id, redo.id)
        assertEquals(TitleCode.OnTime, redo.title.code)
        assertNull(redo.undoUntil, "only one undo per day")
        assertEquals(HttpStatusCode.Conflict, delete(CheckinApiPaths.CHECKIN_TODAY, token).status)
    }

    @Test
    fun `undo is refused after ten minutes`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-09-29T19:00")
        post(CheckinApiPaths.CHECKIN, token)
        clock.advance(Duration.ofMinutes(11))
        assertEquals(HttpStatusCode.Conflict, delete(CheckinApiPaths.CHECKIN_TODAY, token).status)
        assertNotNull(get(CheckinApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().checkin)
    }

    @Test
    fun `a 2am check-in counts for the previous day`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-09-30T02:10")
        val late = post(CheckinApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals("2026-09-29", late.workDay)
        assertEquals(TitleCode.Immortal, late.title.code)

        clock.setBeijing("2026-09-30T06:00")
        assertNull(get(CheckinApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().checkin, "a new 下班日 has started")
    }
}
