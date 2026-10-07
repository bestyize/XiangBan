package xyz.xiaban.server.games

import java.time.LocalDate
import kotlin.test.*

class SnakeWalletRulesTest {
    private val day = LocalDate.of(2026, 10, 7)
    @Test fun `score thresholds cap at eighty and under one hundred earns nothing`() {
        assertEquals(0, SnakeWalletRules.earned(99))
        assertEquals(5, SnakeWalletRules.earned(100))
        assertEquals(6, SnakeWalletRules.earned(120))
        assertEquals(80, SnakeWalletRules.earned(1_000_000_000))
    }
    @Test fun `purchase use and revive debit once and reject insufficient balance`() {
        val bought = SnakeWalletRules.apply(SnakeWalletState(day = day), null, "buy_shield", day)
        assertEquals(20, bought.wallet.coins); assertEquals(1, bought.wallet.shields)
        val used = SnakeWalletRules.apply(bought.wallet, SnakeSessionState(false), "use_shield", day)
        assertEquals(3, used.protection); assertEquals(0, used.wallet.shields)
        assertFails { SnakeWalletRules.apply(used.wallet, used.session, "use_shield", day) }
        assertFails { SnakeWalletRules.apply(used.wallet, used.session, "revive", day) }
        val revived = SnakeWalletRules.apply(SnakeWalletState(day = day), SnakeSessionState(false), "revive", day)
        assertEquals(10, revived.wallet.coins); assertEquals(5, revived.protection)
        assertFails { SnakeWalletRules.apply(revived.wallet.copy(coins = 100), revived.session, "revive", day) }
    }
    @Test fun `settlement is once per round and daily cap permits a partial last award`() {
        val initial = SnakeWalletState(day = day, earnedToday = 290)
        val round = SnakeSessionState(true, score = 9600)
        val result = SnakeWalletRules.apply(initial, round, "settle", day)
        assertEquals(10, result.earned); assertEquals(300, result.wallet.earnedToday)
        assertEquals(0, SnakeWalletRules.apply(result.wallet, result.session, "settle", day).earned)
        assertEquals(0, SnakeWalletRules.apply(result.wallet, round, "settle", day).earned)
        assertEquals(80, SnakeWalletRules.apply(result.wallet, round, "settle", day.plusDays(1)).earned)
        assertEquals(0, SnakeWalletRules.apply(initial, round.copy(score = 99), "settle", day).earned)
    }
    @Test fun `share is daily participation reward and start card is consumed once without stacking`() {
        val round = SnakeSessionState(true)
        val shared = SnakeWalletRules.apply(SnakeWalletState(day = day), round, "share_reward", day)
        assertEquals(5, shared.wallet.startShieldSeconds)
        assertFalse(shared.wallet.view(day).shareAvailable)
        val consumed = SnakeWalletRules.apply(shared.wallet, round.copy(finished = false), "consume_start_shield", day)
        assertEquals(5, consumed.protection); assertEquals(0, consumed.wallet.startShieldSeconds)
        val next = SnakeWalletRules.apply(consumed.wallet, round, "share_reward", day)
        assertEquals(0, next.wallet.startShieldSeconds)
        val tomorrow = SnakeWalletRules.apply(next.wallet, round, "share_reward", day.plusDays(1))
        assertEquals(5, tomorrow.wallet.startShieldSeconds)
        assertEquals(0, SnakeWalletRules.apply(tomorrow.wallet, consumed.session, "consume_start_shield", day.plusDays(1)).protection)
    }
    @Test fun `empty start card succeeds and active versus ended actions are isolated`() {
        val wallet = SnakeWalletState(day = day)
        assertEquals(0, SnakeWalletRules.apply(wallet, SnakeSessionState(false), "consume_start_shield", day).protection)
        for (action in listOf("settle", "share_reward")) assertFails { SnakeWalletRules.apply(wallet, SnakeSessionState(false), action, day) }
        for (action in listOf("revive", "use_shield", "consume_start_shield")) assertFails { SnakeWalletRules.apply(wallet, SnakeSessionState(true), action, day) }
        assertFails { SnakeWalletRules.apply(wallet, null, "settle", day) }
        assertFails { SnakeWalletRules.apply(wallet, null, "unknown", day) }
    }
    @Test fun `merge keeps one welcome grant inventory daily cap and claimed share state`() {
        val source = SnakeWalletState(coins = 110, shields = 2, startShieldSeconds = 5, day = day, earnedToday = 210, sharedDay = day)
        val target = SnakeWalletState(coins = 80, shields = 1, day = day, earnedToday = 120)
        val merged = SnakeWalletRules.merge(source, target, day)!!
        assertEquals(150, merged.coins)
        assertEquals(3, merged.shields)
        assertEquals(5, merged.startShieldSeconds)
        assertEquals(300, merged.earnedToday)
        assertFalse(merged.view(day).shareAvailable)
        val spent = SnakeWalletRules.merge(source.copy(coins = 0), target.copy(coins = 0), day)!!
        assertEquals(0, spent.coins)
        assertEquals(3, spent.shields, "already bought cards are preserved without future debt")
    }
    @Test fun `merge transfers a sole wallet without regranting and only today counts toward cap`() {
        val source = SnakeWalletState(coins = 10, shields = 1, day = day.minusDays(1), earnedToday = 300, sharedDay = day.minusDays(1))
        assertEquals(source.today(day), SnakeWalletRules.merge(source, null, day))
        assertEquals(source.today(day), SnakeWalletRules.merge(null, source, day))
        assertNull(SnakeWalletRules.merge(null, null, day))
        val merged = SnakeWalletRules.merge(source, SnakeWalletState(day = day, earnedToday = 80), day)!!
        assertEquals(10, merged.coins)
        assertEquals(80, merged.earnedToday)
        assertTrue(merged.view(day).shareAvailable)
        val newWallets = SnakeWalletRules.merge(SnakeWalletState(day = day), SnakeWalletState(day = day), day)!!
        assertEquals(40, newWallets.coins)
    }
}
