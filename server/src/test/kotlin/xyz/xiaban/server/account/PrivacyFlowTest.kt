package xyz.xiaban.server.account

import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.client.call.body
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.ConsentUpdate
import xyz.xiaban.shared.EventBatch
import xyz.xiaban.shared.EventIn
import xyz.xiaban.shared.ProfileView
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals

class PrivacyFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `consent is recorded, data exports and the account deletes completely`() = apiTest {
        val session = session()
        val token = session.token
        assertEquals("0.1", put(ApiPaths.CONSENT, token, ConsentUpdate("0.1", overseas = true)).body<ProfileView>().termsVersion)
        clock.setBeijing("2026-10-12T21:00")
        post(ApiPaths.CHECKIN, token)
        post(ApiPaths.EVENTS, token, EventBatch(listOf(EventIn("card_generate", props = mapOf("style" to "default")), EventIn("not_allowed"))))

        val export = get(ApiPaths.ACCOUNT_EXPORT, token)
        assertContains(export.headers[HttpHeaders.ContentDisposition]!!, "xiaban-export.json")
        val json = AppJson.parseToJsonElement(export.bodyAsText()).jsonObject
        assertEquals("overtime_hero", json["checkins"]!!.jsonArray.single().jsonObject["title"]!!.jsonPrimitive.content)
        val events = json["events"]!!.jsonArray.map { it.jsonObject["name"]!!.jsonPrimitive.content }
        assertEquals(setOf("card_generate", "checkin", "unlock"), events.toSet(), "unknown event names are dropped")

        assertEquals(HttpStatusCode.OK, delete(ApiPaths.ACCOUNT, token).status)
        assertEquals(HttpStatusCode.Unauthorized, get(ApiPaths.PROFILE, token).status)
        val leftover = services.db.read { c -> c.queryOne("select count(*) from events where name = 'card_generate'") { it.getInt(1) } }
        assertEquals(0, leftover)

        val fresh = session(session.deviceId)
        assertEquals(false, fresh.profile.onboarded, "same device starts over with a new anonymous account")
    }
}
