package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import xyz.xiaban.wallpaper.data.AccountException
import xyz.xiaban.wallpaper.data.AccountParser
import xyz.xiaban.wallpaper.data.AccountRules
import xyz.xiaban.wallpaper.data.AccountSession
import xyz.xiaban.wallpaper.data.AccountStore
import xyz.xiaban.wallpaper.data.AccountUser
import java.io.File

class AccountParserTest {
    private val loginBody = """
        {"access_token":"acc","refresh_token":"ref","token_type":"Bearer","expires_in":1800,"is_new_user":true,
         "user":{"uid":"u-1","nickname":"阿伴","avatar_url":"https://a.example/p.png"}}
    """.trimIndent()

    @Test
    fun loginReadsTokensUserAndExpiry() {
        val outcome = AccountParser.login(loginBody, " me@example.com ", nowSeconds = 1_000)
        assertTrue(outcome.isNewUser)
        assertEquals("acc", outcome.session.accessToken)
        assertEquals("ref", outcome.session.refreshToken)
        assertEquals(2_800L, outcome.session.expiresAt)
        assertEquals("u-1", outcome.session.user.uid)
        assertEquals("阿伴", outcome.session.user.nickname)
        assertEquals("me@example.com", outcome.session.user.email)
        assertEquals("https://a.example/p.png", outcome.session.user.avatarUrl)
    }

    @Test
    fun loginFallsBackWhenProfileIsBlank() {
        val body = """{"access_token":"a","refresh_token":"r","expires_in":60,"user":{"uid":"u","nickname":"","avatar_url":""}}"""
        val outcome = AccountParser.login(body, "x@y.zz", 0)
        assertFalse(outcome.isNewUser)
        assertEquals(AccountRules.FALLBACK_NICKNAME, outcome.session.user.nickname)
        assertNull(outcome.session.user.avatarUrl)
    }

    @Test
    fun incompleteOrUnreadableAnswersAreRejected() {
        for (body in listOf("""{"refresh_token":"r","user":{"uid":"u"}}""", """{"access_token":"a","refresh_token":"r"}""", "not json", "")) {
            try {
                AccountParser.login(body, "x@y.zz", 0)
                fail("accepted: $body")
            } catch (_: AccountException) {
            }
        }
    }

    @Test
    fun refreshKeepsTheProfileAndSwapsBothTokens() {
        val old = AccountParser.login(loginBody, "me@example.com", 0).session
        val next = AccountParser.refreshed("""{"access_token":"acc2","refresh_token":"ref2","token_type":"Bearer","expires_in":900}""", old, 5_000)
        assertEquals("acc2", next.accessToken)
        assertEquals("ref2", next.refreshToken)
        assertEquals(5_900L, next.expiresAt)
        assertEquals(old.user, next.user)
    }

    @Test
    fun cooldownUsesTheServerValueAndFallsBackToSixty() {
        assertEquals(45, AccountParser.cooldownSeconds("""{"ok":true,"cooldown_seconds":45,"expires_in":300}"""))
        assertEquals(60, AccountParser.cooldownSeconds("""{"ok":true}"""))
        assertEquals(60, AccountParser.cooldownSeconds("""{"cooldown_seconds":0}"""))
    }

    @Test
    fun failureShowsTheServersMessageOtherwiseAPlainSentence() {
        assertEquals("验证码错误或已失效", AccountParser.failure(401, """{"error":true,"code":"unauthorized","message":"验证码错误或已失效"}"""))
        assertEquals("请求过于频繁，请稍后再试", AccountParser.failure(429, "<html>busy</html>"))
        assertEquals("账号服务暂时不可用，请稍后重试", AccountParser.failure(502, ""))
        assertEquals("账号服务返回异常（418）", AccountParser.failure(418, "{}"))
    }
}

class AccountRulesTest {
    private fun session(expiresAt: Long) = AccountSession("a", "r", expiresAt, AccountUser("u", "n", "e@x.yz", null))

    @Test
    fun emailCheckAcceptsOrdinaryAddressesOnly() {
        assertTrue(AccountRules.isEmail("me@example.com"))
        assertTrue(AccountRules.isEmail("  me.name+tag@mail.example.cn  "))
        assertFalse(AccountRules.isEmail(""))
        assertFalse(AccountRules.isEmail("me@example"))
        assertFalse(AccountRules.isEmail("me example@x.com"))
        assertFalse(AccountRules.isEmail("@example.com"))
    }

    @Test
    fun refreshIsDueShortlyBeforeTheAccessTokenRunsOut() {
        val now = 10_000L
        assertFalse(AccountRules.needsRefresh(session(now + AccountRules.REFRESH_MARGIN_SECONDS), now))
        assertTrue(AccountRules.needsRefresh(session(now + AccountRules.REFRESH_MARGIN_SECONDS - 1), now))
        assertTrue(AccountRules.needsRefresh(session(now - 1), now))
    }

    @Test
    fun mobileClientIdMatchesWhatTheServersAllow() {
        assertEquals("xiaoban-mobile-wallpaper", AccountRules.CLIENT_ID)
    }
}

class AccountStoreTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val session = AccountSession("acc", "ref", 1234, AccountUser("u-1", "阿伴", "me@example.com", "https://a.example/p.png"))

    @Test
    fun savedLoginComesBackIntact() {
        val store = AccountStore(File(folder.root, "account.json"))
        assertNull(store.load())
        store.save(session)
        assertEquals(session, AccountStore(File(folder.root, "account.json")).load())
    }

    @Test
    fun missingAvatarStaysMissing() {
        val store = AccountStore(File(folder.root, "account.json"))
        val plain = session.copy(user = session.user.copy(avatarUrl = null))
        store.save(plain)
        assertEquals(plain, store.load())
    }

    @Test
    fun saveReplacesTheOldLoginAndLeavesNoStagingFile() {
        val file = File(folder.root, "account.json")
        val store = AccountStore(file)
        store.save(session)
        store.save(session.copy(accessToken = "acc2", refreshToken = "ref2"))
        assertEquals("ref2", store.load()?.refreshToken)
        assertFalse(File(folder.root, "account.json.new").exists())
    }

    @Test
    fun clearForgetsTheLogin() {
        val store = AccountStore(File(folder.root, "account.json"))
        store.save(session)
        store.clear()
        assertNull(store.load())
    }

    @Test
    fun aDamagedFileReadsAsSignedOut() {
        val file = File(folder.root, "account.json")
        file.writeText("{\"access\":\"only half")
        assertNull(AccountStore(file).load())
    }
}
