package xyz.xiaban.server.games

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import xyz.xiaban.contracts.afterwork.games.*
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.server.account.AccountMerge
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.server.db.queryOne
import kotlinx.serialization.json.*
import kotlin.test.*

class SnakeWalletFlowTest {
    @Test fun `authenticated wallets isolate sessions and actions are transactional and idempotent`() = apiTest {
        val a = session(); val b = session()
        val path = SnakeWalletApiPaths.ACTIONS
        assertEquals(HttpStatusCode.Unauthorized, get(SnakeWalletApiPaths.WALLET, null).status)
        assertEquals(HttpStatusCode.Unauthorized, post(path, null, SnakeWalletAction("buy_shield", "anonymous")).status)
        assertEquals(40, get(SnakeWalletApiPaths.WALLET, a.token).body<SnakeWalletView>().coins)
        val snake = post("/api/games/sessions", a.token, GameStart("snake-arena")).body<GameSessionView>()
        val other = post("/api/games/sessions", a.token, GameStart("merge-2048")).body<GameSessionView>()
        assertEquals(HttpStatusCode.NotFound, post(path, b.token, SnakeWalletAction("consume_start_shield", "wrong-owner", snake.id)).status)
        assertEquals(HttpStatusCode.BadRequest, post(path, a.token, SnakeWalletAction("consume_start_shield", "wrong-game", other.id)).status)
        assertEquals(HttpStatusCode.BadRequest, post(path, a.token, SnakeWalletAction("settle", "early", snake.id)).status)
        val buy = SnakeWalletAction("buy_shield", "buy")
        val bought = post(path, a.token, buy).body<SnakeWalletResult>()
        assertEquals(20, bought.wallet.coins)
        assertEquals(bought, post(path, a.token, buy).body<SnakeWalletResult>())
        assertEquals(HttpStatusCode.BadRequest, post(path, a.token, buy.copy(action = "revive", sessionId = snake.id)).status)
        val used = post(path, a.token, SnakeWalletAction("use_shield", "use", snake.id)).body<SnakeWalletResult>()
        assertEquals(3, used.protectionSeconds); assertEquals(0, used.wallet.shields)
        put("/api/games/sessions/${snake.id}", a.token, GameProgress(score = 9600, finished = true))
        val settled = post(path, a.token, SnakeWalletAction("settle", "settle", snake.id)).body<SnakeWalletResult>()
        assertEquals(80, settled.coinsEarned); assertEquals(100, settled.wallet.coins)
        assertEquals(0, post(path, a.token, SnakeWalletAction("settle", "settle-again", snake.id)).body<SnakeWalletResult>().coinsEarned)
        assertEquals(HttpStatusCode.BadRequest, post(path, a.token, SnakeWalletAction("revive", "late", snake.id)).status)
        val shared = post(path, a.token, SnakeWalletAction("share_reward", "share", snake.id)).body<SnakeWalletResult>()
        assertEquals(5, shared.wallet.startShieldSeconds); assertFalse(shared.wallet.shareAvailable)
        val next = post("/api/games/sessions", a.token, GameStart("snake-arena")).body<GameSessionView>()
        assertEquals(5, post(path, a.token, SnakeWalletAction("consume_start_shield", "start", next.id)).body<SnakeWalletResult>().protectionSeconds)
        assertEquals(0, post(path, a.token, SnakeWalletAction("consume_start_shield", "start-again", next.id)).body<SnakeWalletResult>().protectionSeconds)
        val revived = post(path, a.token, SnakeWalletAction("revive", "revive", next.id)).body<SnakeWalletResult>()
        assertEquals(70, revived.wallet.coins); assertEquals(5, revived.protectionSeconds)
        assertEquals(HttpStatusCode.BadRequest, post(path, a.token, SnakeWalletAction("revive", "revive-again", next.id)).status)
        assertEquals(revived, post(path, a.token, SnakeWalletAction("revive", "revive", next.id)).body<SnakeWalletResult>())
        val purchases = coroutineScope { (1..3).map { index -> async { post(path, b.token, SnakeWalletAction("buy_shield", "parallel-$index")).status } }.awaitAll() }
        assertEquals(2, purchases.count { it == HttpStatusCode.OK })
        assertEquals(1, purchases.count { it == HttpStatusCode.BadRequest })
        val bWallet = get(SnakeWalletApiPaths.WALLET, b.token).body<SnakeWalletView>()
        assertEquals(0, bWallet.coins); assertEquals(2, bWallet.shields)
    }
    @Test fun `account merge transfers wallets and stable session flags then exports and deletes them`() = apiTest {
        val source = session(); val target = session()
        val sourceId = source.profile.userId; val targetId = target.profile.userId
        val path = SnakeWalletApiPaths.ACTIONS
        val completed = post("/api/games/sessions", source.token, GameStart("snake-arena")).body<GameSessionView>()
        post(path, source.token, SnakeWalletAction("buy_shield", "source-buy"))
        put("/api/games/sessions/${completed.id}", source.token, GameProgress(score = 9600, finished = true))
        val originalResult = post(path, source.token, SnakeWalletAction("settle", "source-settle", completed.id)).body<SnakeWalletResult>()
        post(path, source.token, SnakeWalletAction("share_reward", "source-share", completed.id))
        val live = post("/api/games/sessions", source.token, GameStart("snake-arena")).body<GameSessionView>()
        post(path, source.token, SnakeWalletAction("consume_start_shield", "source-consume", live.id))
        post(path, source.token, SnakeWalletAction("revive", "source-revive", live.id))
        post(path, source.token, SnakeWalletAction("use_shield", "source-use", live.id))
        post(path, target.token, SnakeWalletAction("buy_shield", "target-buy"))
        val targetRound = post("/api/games/sessions", target.token, GameStart("snake-arena")).body<GameSessionView>()
        put("/api/games/sessions/${targetRound.id}", target.token, GameProgress(score = 9600, finished = true))
        post(path, target.token, SnakeWalletAction("settle", "target-settle", targetRound.id))
        post(path, target.token, SnakeWalletAction("share_reward", "target-share", targetRound.id))

        AccountMerge(services.db, services.clock).merge(sourceId, targetId)
        val merged = get(SnakeWalletApiPaths.WALLET, target.token).body<SnakeWalletView>()
        assertEquals(130, merged.coins); assertEquals(1, merged.shields)
        assertEquals(5, merged.startShieldSeconds); assertFalse(merged.shareAvailable)
        assertFalse(services.users.exists(sourceId))
        assertEquals(0, post(path, target.token, SnakeWalletAction("settle", "after-merge", completed.id)).body<SnakeWalletResult>().coinsEarned)
        assertEquals(originalResult, post(path, target.token, SnakeWalletAction("settle", "source-settle", completed.id)).body<SnakeWalletResult>())
        assertEquals(HttpStatusCode.BadRequest, post(path, target.token, SnakeWalletAction("revive", "after-merge-revive", live.id)).status)
        val consume = post(path, target.token, SnakeWalletAction("consume_start_shield", "after-merge-consume", live.id)).body<SnakeWalletResult>()
        assertEquals(0, consume.protectionSeconds); assertEquals(5, consume.wallet.startShieldSeconds)
        // The completed source session remains owned by the target and still supports private share reward references.
        assertEquals(HttpStatusCode.OK, post(path, target.token, SnakeWalletAction("share_reward", "after-merge-share", completed.id)).status)
        val exported = get(AccountApiPaths.ACCOUNT_EXPORT, target.token).body<JsonObject>()
        assertEquals(160, exported.getValue("snakeWallet").jsonObject.getValue("earned_today").jsonPrimitive.int)
        assertEquals(3, exported.getValue("snakeWalletSessions").jsonArray.size)
        assertTrue(exported.getValue("snakeWalletRequests").jsonArray.any { it.jsonObject["request_id"]?.jsonPrimitive?.content == "source-revive" })
        delete(AccountApiPaths.ACCOUNT, target.token)
        services.db.read { c ->
            for (table in listOf("snake_wallets", "snake_wallet_sessions", "snake_wallet_requests"))
                assertEquals(0L, c.queryOne("select count(*) as n from $table where user_id=?", targetId) { it.getLong("n") })
        }
    }
}
