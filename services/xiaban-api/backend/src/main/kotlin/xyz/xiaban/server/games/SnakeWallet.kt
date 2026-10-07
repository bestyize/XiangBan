package xyz.xiaban.server.games

import kotlinx.serialization.encodeToString
import xyz.xiaban.contracts.afterwork.games.*
import xyz.xiaban.server.db.*
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.notFound
import java.sql.Connection
import java.sql.ResultSet
import java.time.Clock
import java.time.LocalDate
import java.time.ZoneId
import java.util.UUID

internal data class SnakeWalletState(
    val coins: Int = 40,
    val shields: Int = 0,
    val startShieldSeconds: Int = 0,
    val day: LocalDate,
    val earnedToday: Int = 0,
    val sharedDay: LocalDate? = null,
) {
    fun today(today: LocalDate) = if (day == today) this else copy(day = today, earnedToday = 0)
    fun view(today: LocalDate) = SnakeWalletView(coins, shields, startShieldSeconds, sharedDay != today)
}

internal data class SnakeSessionState(
    val finished: Boolean,
    val score: Long = 0,
    val revived: Boolean = false,
    val startShieldConsumed: Boolean = false,
    val settled: Boolean = false,
)
internal data class SnakeTransition(val wallet: SnakeWalletState, val session: SnakeSessionState?, val earned: Int = 0, val protection: Int = 0)

/** Pure policy; authentication, ownership, persistence and idempotency live in the service below. */
internal object SnakeWalletRules {
    val actions = setOf("buy_shield", "use_shield", "revive", "consume_start_shield", "settle", "share_reward")
    fun earned(score: Long): Int = if (score < 100) 0 else (5 + score / 120).coerceAtMost(80).toInt()
    fun merge(source: SnakeWalletState?, target: SnakeWalletState?, day: LocalDate): SnakeWalletState? {
        if (source == null) return target?.today(day)
        if (target == null) return source.today(day)
        val a = source.today(day)
        val b = target.today(day)
        return SnakeWalletState(
            // Keep one welcome grant. Already consumed inventory is never clawed back or made a debt.
            coins = (a.coins.toLong() + b.coins - 40).coerceIn(0, Int.MAX_VALUE.toLong()).toInt(),
            shields = (a.shields.toLong() + b.shields).coerceAtMost(Int.MAX_VALUE.toLong()).toInt(),
            startShieldSeconds = maxOf(a.startShieldSeconds, b.startShieldSeconds),
            day = day,
            earnedToday = (a.earnedToday + b.earnedToday).coerceAtMost(300),
            sharedDay = listOfNotNull(a.sharedDay, b.sharedDay).maxOrNull(),
        )
    }
    fun apply(raw: SnakeWalletState, session: SnakeSessionState?, action: String, day: LocalDate): SnakeTransition {
        if (action !in actions) badRequest("没有这个糖果道具操作")
        var wallet = raw.today(day)
        var round = session
        var reward = 0
        var protection = 0
        if (action != "buy_shield" && round == null) badRequest("请先开始一局贪吃蛇")
        if (action in setOf("use_shield", "revive", "consume_start_shield") && round!!.finished) badRequest("这一局已经结束啦")
        if (action in setOf("settle", "share_reward") && !round!!.finished) badRequest("本局结束后才能领取奖励")
        when (action) {
            "buy_shield" -> {
                if (wallet.coins < 20) badRequest("糖果币不够啦")
                wallet = wallet.copy(coins = wallet.coins - 20, shields = wallet.shields + 1)
            }
            "use_shield" -> {
                if (wallet.shields < 1) badRequest("先兑换一张保护卡吧")
                wallet = wallet.copy(shields = wallet.shields - 1)
                protection = 3
            }
            "revive" -> {
                if (round!!.revived) badRequest("每局只能续命一次")
                if (wallet.coins < 30) badRequest("糖果币不够啦")
                wallet = wallet.copy(coins = wallet.coins - 30)
                round = round.copy(revived = true)
                protection = 5
            }
            "consume_start_shield" -> {
                if (!round!!.startShieldConsumed) {
                    protection = wallet.startShieldSeconds
                    wallet = wallet.copy(startShieldSeconds = 0)
                    round = round.copy(startShieldConsumed = true)
                }
            }
            "settle" -> {
                if (!round!!.settled) {
                    reward = earned(round.score).coerceAtMost(300 - wallet.earnedToday)
                    wallet = wallet.copy(coins = wallet.coins + reward, earnedToday = wallet.earnedToday + reward)
                    round = round.copy(settled = true)
                }
            }
            "share_reward" -> {
                if (wallet.sharedDay != day) wallet = wallet.copy(sharedDay = day, startShieldSeconds = 5)
            }
        }
        return SnakeTransition(wallet, round, reward, protection)
    }
}

class SnakeWallet(private val db: Db, private val clock: Clock) {
    private fun day(): LocalDate = LocalDate.now(clock.withZone(ZoneId.of("Asia/Shanghai")))

    suspend fun read(userId: Long): SnakeWalletView = db.tx { c ->
        lockUser(c, userId)
        val today = day()
        loadWallet(c, userId, today).view(today)
    }

    suspend fun act(userId: Long, request: SnakeWalletAction): SnakeWalletResult {
        if (request.action !in SnakeWalletRules.actions) badRequest("没有这个糖果道具操作")
        if (!request.requestId.matches(Regex("[A-Za-z0-9_.:-]{1,120}"))) badRequest("道具请求编号不正确")
        val sessionId = request.sessionId?.let { runCatching { UUID.fromString(it) }.getOrElse { badRequest("游戏记录编号不正确") } }
        if (request.action != "buy_shield" && sessionId == null) badRequest("请先开始一局贪吃蛇")
        return db.tx { c ->
            lockUser(c, userId)
            val today = day()
            val wallet = loadWallet(c, userId, today)
            val previous = c.queryOne("select action,session_id,response::text as response from snake_wallet_requests where user_id=? and request_id=?", userId, request.requestId) {
                Triple(it.getString("action"), it.getObject("session_id")?.toString(), it.getString("response"))
            }
            if (previous != null) {
                if (previous.first != request.action || previous.second != sessionId?.toString()) badRequest("请求编号已用于其他操作")
                return@tx AppJson.decodeFromString<SnakeWalletResult>(previous.third)
            }
            val session = sessionId?.let { id ->
                val game = c.queryOne("select game_id,score,finished_at from game_sessions where id=? and user_id=? for update", id, userId) {
                    Triple(it.getString("game_id"), it.getLong("score"), it.instant("finished_at") != null)
                } ?: notFound("没有找到你的游戏记录")
                if (game.first != "snake-arena") badRequest("只有贪吃蛇可以使用糖果道具")
                c.execute("insert into snake_wallet_sessions(session_id,user_id) values (?,?) on conflict do nothing", id, userId)
                c.queryOne("select * from snake_wallet_sessions where session_id=? and user_id=?", id, userId) {
                    SnakeSessionState(game.third, game.second, it.getBoolean("revived"), it.getBoolean("start_shield_consumed"), it.getBoolean("settled"))
                } ?: notFound("没有找到你的道具记录")
            }
            val next = SnakeWalletRules.apply(wallet, session, request.action, today)
            c.execute("update snake_wallets set coins=?,shields=?,start_shield_seconds=?,earning_day=?,earned_today=?,shared_day=?,updated_at=now() where user_id=?",
                next.wallet.coins, next.wallet.shields, next.wallet.startShieldSeconds, today, next.wallet.earnedToday, next.wallet.sharedDay, userId)
            next.session?.let { s ->
                c.execute("update snake_wallet_sessions set revived=?,start_shield_consumed=?,settled=?,coins_earned=coins_earned+? where session_id=? and user_id=?",
                    s.revived, s.startShieldConsumed, s.settled, next.earned, sessionId, userId)
            }
            val response = SnakeWalletResult(next.wallet.view(today), next.earned, next.protection)
            c.execute("insert into snake_wallet_requests(user_id,request_id,action,session_id,response) values (?,?,?,?,?::jsonb)",
                userId, request.requestId, request.action, sessionId, AppJson.encodeToString(response))
            response
        }
    }

    private fun loadWallet(c: Connection, userId: Long, today: LocalDate): SnakeWalletState {
        c.execute("insert into snake_wallets(user_id,earning_day) values (?,?) on conflict do nothing", userId, today)
        return c.queryOne("select * from snake_wallets where user_id=? for update", userId, map = ::wallet)!!.today(today)
    }
    private fun lockUser(c: Connection, userId: Long) {
        c.queryOne("select id from users where id=? for update", userId) { it.getLong("id") }
            ?: notFound("没有找到你的账号")
    }

    companion object {
        private fun wallet(rs: ResultSet) = SnakeWalletState(rs.getInt("coins"), rs.getInt("shields"), rs.getInt("start_shield_seconds"),
            rs.localDate("earning_day"), rs.getInt("earned_today"), rs.getObject("shared_day", LocalDate::class.java))

        /** Called inside AccountMerge's transaction after locking both user rows in ID order. */
        internal fun mergeAccounts(c: Connection, source: Long, target: Long, day: LocalDate) {
            val sourceWallet = c.queryOne("select * from snake_wallets where user_id=? for update", source, map = ::wallet)
            val targetWallet = c.queryOne("select * from snake_wallets where user_id=? for update", target, map = ::wallet)
            val combined = SnakeWalletRules.merge(sourceWallet, targetWallet, day)
            if (combined != null) c.execute("""insert into snake_wallets(user_id,coins,shields,start_shield_seconds,earning_day,earned_today,shared_day)
                values (?,?,?,?,?,?,?) on conflict(user_id) do update set coins=excluded.coins,shields=excluded.shields,
                start_shield_seconds=excluded.start_shield_seconds,earning_day=excluded.earning_day,
                earned_today=excluded.earned_today,shared_day=excluded.shared_day,updated_at=now()""",
                target, combined.coins, combined.shields, combined.startShieldSeconds, day, combined.earnedToday, combined.sharedDay)
            // Session IDs remain stable so own-result share references survive the account switch.
            c.execute("update snake_wallet_sessions set user_id=? where user_id=?", target, source)
            // Request IDs are scoped to an account. In a collision the target's existing response wins;
            // the transferred session flags still prevent settlement, start-card and revive duplication.
            c.execute("""insert into snake_wallet_requests(user_id,request_id,action,session_id,response,created_at)
                select ?,request_id,action,session_id,response,created_at from snake_wallet_requests where user_id=?
                on conflict(user_id,request_id) do nothing""", target, source)
        }
    }
}
