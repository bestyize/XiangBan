package xyz.xiaban.server.games

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlin.test.*
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.server.account.AccountMerge
import xyz.xiaban.server.account.bind
import xyz.xiaban.server.support.apiTest

class GameSavesFlowTest {
    private fun progress(stage: String, seals: Int) = buildJsonObject {
        put("v", 1)
        put("levels", buildJsonObject { put(stage, buildJsonObject { put("clears", 1); put("seals", seals) }) })
    }

    @Test fun `registered accounts keep one save per game and stale writes never roll it back`() = apiTest {
        val guest = session()
        val save = GameSaveRequest(progress("1-1", 7), System.currentTimeMillis())
        assertEquals(HttpStatusCode.Unauthorized, get("/api/games/saves/ink-blade", null).status)
        assertEquals(HttpStatusCode.Forbidden, put("/api/games/saves/ink-blade", guest.token, save).status)
        val a = bind(guest.token, "save-a@example.com")
        val b = bind(session().token, "save-b@example.com")
        assertEquals(GameSaveView(), get("/api/games/saves/ink-blade", a.token).body<GameSaveView>())
        assertEquals(save.data, put("/api/games/saves/ink-blade", a.token, save).body<GameSaveView>().data)
        assertEquals(GameSaveView(save.data, save.updatedAt), get("/api/games/saves/ink-blade", a.token).body<GameSaveView>())
        assertNull(get("/api/games/saves/ink-blade", b.token).body<GameSaveView>().data)

        val stale = GameSaveRequest(progress("1-1", 1), save.updatedAt - 1)
        assertEquals(save.data, put("/api/games/saves/ink-blade", a.token, stale).body<GameSaveView>().data)
        val newer = GameSaveRequest(progress("1-2", 3), save.updatedAt + 1)
        assertEquals(newer.data, put("/api/games/saves/ink-blade", a.token, newer).body<GameSaveView>().data)
    }

    @Test fun `saves reject other games, oversized documents and impossible clocks`() = apiTest {
        val a = bind(session().token, "save-limits@example.com")
        val now = System.currentTimeMillis()
        assertEquals(HttpStatusCode.BadRequest, put("/api/games/saves/merge-2048", a.token, GameSaveRequest(JsonObject(emptyMap()), now)).status)
        assertEquals(HttpStatusCode.BadRequest, get("/api/games/saves/unknown", a.token).status)
        val huge = JsonObject(mapOf("blob" to JsonPrimitive("x".repeat(GameSaves.MAX_BYTES))))
        assertEquals(HttpStatusCode.BadRequest, put("/api/games/saves/ink-blade", a.token, GameSaveRequest(huge, now)).status)
        assertEquals(HttpStatusCode.BadRequest, put("/api/games/saves/ink-blade", a.token, GameSaveRequest(JsonObject(emptyMap()), now + 3_600_000)).status)
        assertEquals(HttpStatusCode.BadRequest, put("/api/games/saves/ink-blade", a.token, GameSaveRequest(JsonObject(emptyMap()), 0)).status)
    }

    @Test fun `saves are exported, deleted with the account and kept when an anonymous account merges in`() = apiTest {
        val a = bind(session().token, "save-export@example.com")
        put("/api/games/saves/ink-blade", a.token, GameSaveRequest(progress("1-3", 5), System.currentTimeMillis()))
        val export = get(AccountApiPaths.ACCOUNT_EXPORT, a.token).body<JsonObject>()
        assertTrue(export.containsKey("gameSaves"))
        delete(AccountApiPaths.ACCOUNT, a.token)
        assertNull(services.gameSaves.get(a.profile.userId, "ink-blade").data)

        val target = bind(session().token, "save-merge@example.com")
        val anonymous = session()
        services.gameSaves.put(anonymous.profile.userId, "ink-blade", GameSaveRequest(progress("1-4", 7), System.currentTimeMillis()))
        AccountMerge(services.db).merge(anonymous.profile.userId, target.profile.userId)
        assertEquals(progress("1-4", 7), services.gameSaves.get(target.profile.userId, "ink-blade").data)
    }
}
