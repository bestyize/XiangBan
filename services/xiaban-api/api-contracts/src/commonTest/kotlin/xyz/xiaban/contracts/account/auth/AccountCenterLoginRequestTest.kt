package xyz.xiaban.contracts.account.auth

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class AccountCenterLoginRequestTest {
    @Test
    fun optionalFieldsDefaultSoMinimalClientsStayCompatible() {
        val request = Json.decodeFromString<AccountCenterLoginRequest>("""{"accountToken":"t","deviceId":"device-id-1234567890"}""")
        assertFalse(request.mergeGuest)
        assertEquals(null, request.termsVersion)
    }

    @Test
    fun roundTripsAndCarriesNoUsernameField() {
        val request = AccountCenterLoginRequest("t", "device-id-1234567890", mergeGuest = true, termsVersion = "0.3")
        val json = Json.encodeToString(request)
        assertFalse(json.contains("username"), json)
        assertEquals(request, Json.decodeFromString<AccountCenterLoginRequest>(json))
    }
}
