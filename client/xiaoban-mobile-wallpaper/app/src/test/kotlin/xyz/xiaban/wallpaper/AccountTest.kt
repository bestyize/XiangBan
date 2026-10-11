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
import xyz.xiaban.wallpaper.data.CodeLoginResult
import java.io.File

class AccountParserTest {
    private val loginBody = """
        {"access_token":"acc","refresh_token":"ref","token_type":"Bearer","expires_in":1800,"is_new_user":true,
         "user":{"uid":"u-1","username":"Alice_01","avatar_url":"https://a.example/p.png"}}
    """.trimIndent()

    @Test
    fun loginReadsTokensUserAndExpiry() {
        val outcome = AccountParser.login(loginBody, nowSeconds = 1_000)
        assertTrue(outcome.isNewUser)
        assertEquals("acc", outcome.session.accessToken)
        assertEquals("ref", outcome.session.refreshToken)
        assertEquals(2_800L, outcome.session.expiresAt)
        assertEquals("u-1", outcome.session.user.uid)
        assertEquals("Alice_01", outcome.session.user.username)
        assertEquals("https://a.example/p.png", outcome.session.user.avatarUrl)
    }

    @Test
    fun aBlankAvatarIsNoAvatar() {
        val body = """{"access_token":"a","refresh_token":"r","expires_in":60,"user":{"uid":"u","username":"Bob_Smith","avatar_url":""}}"""
        val outcome = AccountParser.login(body, 0)
        assertFalse(outcome.isNewUser)
        assertNull(outcome.session.user.avatarUrl)
    }

    @Test
    fun incompleteOrUnreadableAnswersAreRejected() {
        val incomplete = listOf(
            """{"refresh_token":"r","user":{"uid":"u","username":"Alice_01"}}""",
            """{"access_token":"a","refresh_token":"r"}""",
            """{"access_token":"a","refresh_token":"r","user":{"uid":"u","username":""}}""",
            "not json",
            "",
        )
        for (body in incomplete) {
            try {
                AccountParser.login(body, 0)
                fail("accepted: $body")
            } catch (_: AccountException) {
            }
        }
    }

    @Test
    fun refreshKeepsTheProfileAndSwapsBothTokens() {
        val old = AccountParser.login(loginBody, 0).session
        val next = AccountParser.refreshed("""{"access_token":"acc2","refresh_token":"ref2","token_type":"Bearer","expires_in":900}""", old, 5_000)
        assertEquals("acc2", next.accessToken)
        assertEquals("ref2", next.refreshToken)
        assertEquals(5_900L, next.expiresAt)
        assertEquals(old.user, next.user)
    }

    @Test
    fun aCodeLoginEitherSignsInOrAsksForRegistration() {
        val signedIn = AccountParser.codeLogin(loginBody, 0)
        assertEquals("Alice_01", (signedIn as CodeLoginResult.SignedIn).outcome.session.user.username)

        val pending = AccountParser.codeLogin("""{"registration_required":true,"registration_token":"tok","expires_in":900}""", 0)
        assertEquals(CodeLoginResult.RegistrationRequired("tok"), pending)

        try {
            AccountParser.codeLogin("""{"registration_required":true}""", 0)
            fail("a registration without a token must be rejected")
        } catch (_: AccountException) {
        }
    }

    @Test
    fun usernameCheckReadsAvailabilityAndTheMessage() {
        assertEquals(true, AccountParser.usernameCheck("""{"available":true,"message":null}""").available)
        assertNull(AccountParser.usernameCheck("""{"available":true,"message":null}""").message)
        val taken = AccountParser.usernameCheck("""{"available":false,"message":"这个用户名已被占用，换一个吧"}""")
        assertFalse(taken.available)
        assertEquals("这个用户名已被占用，换一个吧", taken.message)
    }

    @Test
    fun errorCodeComesFromTheBodyOrIsNull() {
        assertEquals("username_taken", AccountParser.errorCode("""{"error":true,"code":"username_taken","message":"x"}"""))
        assertNull(AccountParser.errorCode("<html>busy</html>"))
        assertNull(AccountParser.errorCode("""{"message":"no code"}"""))
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
    private fun session(expiresAt: Long) = AccountSession("a", "r", expiresAt, AccountUser("u", "Alice_01", null))

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
    fun usernameRulesMatchTheAccountCenter() {
        for (ok in listOf("abcd", "Alice_01", "a2345678901234567890")) assertNull(ok, AccountRules.usernameError(ok))
        assertNull(AccountRules.usernameError("  "))
        assertEquals("用户名需以字母开头", AccountRules.usernameError("1abc"))
        assertEquals("用户名需以字母开头", AccountRules.usernameError("用户名abcd"))
        assertEquals("只能包含字母、数字和下划线", AccountRules.usernameError("ab-cd"))
        assertEquals("需为 4-20 个字符", AccountRules.usernameError("abc"))
        assertEquals("需为 4-20 个字符", AccountRules.usernameError("a23456789012345678901"))
    }

    @Test
    fun passwordRulesCheckLengthAndTheUsername() {
        assertNull(AccountRules.passwordError(""))
        assertEquals("密码至少 8 位", AccountRules.passwordError("short"))
        assertEquals("密码不能和用户名相同", AccountRules.passwordError("Alice_01", "alice_01"))
        assertNull(AccountRules.passwordError("long-enough-1", "alice_01"))
        assertEquals("密码最多 128 位", AccountRules.passwordError("x".repeat(129)))
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

    private val session = AccountSession("acc", "ref", 1234, AccountUser("u-1", "Alice_01", "https://a.example/p.png"))

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
    fun aLoginSavedByAnOlderVersionReadsAsSignedOut() {
        val file = File(folder.root, "account.json")
        file.writeText("""{"access":"a","refresh":"r","exp":1,"user":{"uid":"u","nick":"阿伴","email":"me@example.com","avatar":""}}""")
        assertNull(AccountStore(file).load())
    }

    @Test
    fun aDamagedFileReadsAsSignedOut() {
        val file = File(folder.root, "account.json")
        file.writeText("{\"access\":\"only half")
        assertNull(AccountStore(file).load())
    }
}
