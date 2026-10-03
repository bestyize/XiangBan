package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import xyz.xiaban.server.afterwork.*
import xyz.xiaban.server.games.*
import xyz.xiaban.server.support.*
import xyz.xiaban.shared.*
import kotlin.test.*

class FeatureDataFlowTest {
    @Test fun `game and afterwork data belong to numeric IDs in their own stores`() = apiTest {
        val a = session(); val b = session()
        val game = post("/api/games/sessions", a.token, GameStart("merge-2048")).body<GameSessionView>()
        val progress = put("/api/games/sessions/${game.id}", a.token, GameProgress(score = 128, maxTile = 32, finished = true)).body<GameSessionView>()
        assertEquals(128, progress.score); assertNotNull(progress.finishedAt)
        assertEquals(HttpStatusCode.NotFound, put("/api/games/sessions/${game.id}", b.token, GameProgress(score = 999)).status)
        assertTrue(get("/api/games/summary", b.token).body<List<GameSummary>>().isEmpty())
        assertEquals(128, get("/api/games/summary", a.token).body<List<GameSummary>>().single().bestScore)
        put("/api/afterwork/choices", a.token, AfterworkChoice("2026-10-03", "rest"))
        put("/api/afterwork/preferences", a.token, AfterworkPreference(preferredActivityChanged = true, preferredActivityId = "walk"))
        val imported = post("/api/afterwork/import", a.token, AfterworkData(listOf(AfterworkEntry("2026-10-03", "food", 1), AfterworkEntry("2026-10-02", "food", 1)), "food")).body<AfterworkData>()
        assertEquals("rest", imported.entries.last().activityId); assertEquals("walk", imported.preferredActivityId)
        assertTrue(get("/api/afterwork", b.token).body<AfterworkData>().entries.isEmpty())
        assertEquals(a.profile, get(ApiPaths.PROFILE, a.token).body<ProfileView>())
        assertEquals(HttpStatusCode.BadRequest, put("/api/afterwork/choices", a.token, AfterworkChoice("not-date", "rest")).status)
        assertEquals(HttpStatusCode.BadRequest, put("/api/games/sessions/${game.id}", a.token, GameProgress(score = -1)).status)
    }

    @Test fun `merging guest data moves feature records to the existing user ID and deleting cascades`() = apiTest {
        val target = bind(session().token, "feature-merge@example.com")
        val guest = session()
        val game = post("/api/games/sessions", guest.token, GameStart("falling-blocks")).body<GameSessionView>()
        put("/api/games/sessions/${game.id}", guest.token, GameProgress(score = 42, lines = 2))
        put("/api/afterwork/choices", guest.token, AfterworkChoice("2026-10-03", "food"))
        val merged = bind(guest.token, "feature-merge@example.com")
        assertEquals(target.profile.userId, merged.profile.userId); assertTrue(merged.merged)
        assertEquals(42, get("/api/games/summary", merged.token).body<List<GameSummary>>().single().bestScore)
        assertEquals("food", get("/api/afterwork", merged.token).body<AfterworkData>().entries.single().activityId)
        assertEquals(HttpStatusCode.Unauthorized, get("/api/games/summary", guest.token).status)
        val exported = get(ApiPaths.ACCOUNT_EXPORT, merged.token).body<kotlinx.serialization.json.JsonObject>()
        assertTrue(exported.containsKey("gameSessions")); assertTrue(exported.containsKey("afterworkChoices"))
        delete(ApiPaths.ACCOUNT, merged.token)
        assertFalse(services.users.exists(merged.profile.userId))
        assertEquals(HttpStatusCode.NotFound, put("/api/games/sessions/${game.id}", session().token, GameProgress(score = 99)).status)
    }
}
