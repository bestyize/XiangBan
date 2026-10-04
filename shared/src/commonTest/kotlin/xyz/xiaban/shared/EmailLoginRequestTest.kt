package xyz.xiaban.shared

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull

class EmailLoginRequestTest {
    @Test
    fun legacyRequestsRemainCompatible() {
        val request = Json.decodeFromString<EmailLoginRequest>(
            """{"email":"me@example.com","code":"123456","deviceId":"device-id-1234"}""",
        )
        assertNull(request.termsVersion)
        assertNull(request.username)
        assertFalse(request.mergeGuest)
    }

    @Test
    fun loginCarriesDocumentVersionAlongsideAccountChoices() {
        val request = EmailLoginRequest("me@example.com", "123456", "device-id-1234",
            mergeGuest = true, username = "UserName", termsVersion = "0.3")
        val restored = Json.decodeFromString<EmailLoginRequest>(Json.encodeToString(request))
        assertEquals(request, restored)
        assertEquals("0.3", restored.termsVersion)
    }
}
