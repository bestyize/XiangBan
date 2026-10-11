package xyz.xiaban.contracts

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.guardian.GuardianApiPaths
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.session.SessionApiPaths
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.afterwork.checkin.TitleView
import xyz.xiaban.contracts.companion.chat.ChatApiPaths
import xyz.xiaban.contracts.companion.personality.Mood
import xyz.xiaban.contracts.companion.personality.Personality

class ContractCompatibilityTest {
    @Test
    fun publicEndpointPathsRemainCompatible() {
        assertEquals("/api/session", SessionApiPaths.SESSION)
        assertEquals("/api/profile", ProfileApiPaths.PROFILE)
        assertEquals("/api/account/center/login", AccountApiPaths.ACCOUNT_CENTER_LOGIN)
        assertEquals("/api/guardian/pause", GuardianApiPaths.GUARDIAN_PAUSE)
        assertEquals("/api/checkin/today", CheckinApiPaths.CHECKIN_TODAY)
        assertEquals("/api/chat/stream", ChatApiPaths.CHAT_STREAM)
    }

    @Test
    fun titleResponseRetainsItsWireShapeAfterRemovingBusinessMetadataFromTheEnum() {
        val response = TitleView(TitleCode.ExactOnTime, "准点之神", true, Mood.Sparkle)
        val golden = """{"code":"exact_on_time","name":"准点之神","rare":true,"mood":"sparkle"}"""
        assertEquals(Json.parseToJsonElement(golden), Json.parseToJsonElement(Json.encodeToString(response)))
        assertEquals(response, Json.decodeFromString<TitleView>(golden))
    }

    @Test
    fun personalityWireCodesDoNotBecomeKotlinEnumNames() {
        assertEquals(listOf("\"toxic\"", "\"gentle\"", "\"chatty\"", "\"tsundere\""),
            Personality.entries.map { Json.encodeToString(it) })
    }
}
