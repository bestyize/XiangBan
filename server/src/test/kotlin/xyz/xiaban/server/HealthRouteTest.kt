package xyz.xiaban.server

import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpStatusCode
import io.ktor.server.testing.testApplication
import xyz.xiaban.server.support.TestEnv
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals

class HealthRouteTest {
    @Test
    fun `reports ok when database and redis are up`() {
        val services = TestEnv.services()
        testApplication {
            application { xiaban(services) }
            val response = client.get("/api/health")
            assertEquals(HttpStatusCode.OK, response.status)
            assertContains(response.bodyAsText(), "\"status\":\"ok\"")
        }
    }
}
