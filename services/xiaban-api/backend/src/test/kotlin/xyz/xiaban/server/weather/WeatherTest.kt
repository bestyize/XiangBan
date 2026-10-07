package xyz.xiaban.server.weather

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.meta.City
import xyz.xiaban.server.support.FakeWeather
import java.time.Clock
import java.time.LocalDate
import java.time.OffsetDateTime
import java.time.ZoneId
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation as ClientContentNegotiation

class WeatherTest {
    private val zone = ZoneId.of("Asia/Shanghai")
    private fun at(s: String) = OffsetDateTime.parse("$s:00+08:00").toInstant()

    @Test
    fun `rain on the way home is found within four hours`() {
        val f = FakeWeather.forecast("2026-10-12T21:00", wetAt = setOf(22))
        assertEquals("小雨", WeatherInsights.rainSoon(f, at("2026-10-12T21:10"))?.text)
        assertNull(WeatherInsights.rainSoon(FakeWeather.forecast("2026-10-12T21:00", wetAt = setOf(3)), at("2026-10-12T21:10")))
    }

    @Test
    fun `next morning, temperature drop and copy tag`() {
        val f = FakeWeather.forecast(
            "2026-11-20T19:00", temp = 4,
            daily = listOf(FakeWeather.day("2026-11-20", 6, 15), FakeWeather.day("2026-11-21", 1, 8)),
        )
        assertEquals(4, WeatherInsights.nextMorningTemp(f, at("2026-11-20T19:30"), zone))
        assertEquals(7, WeatherInsights.tempDrop(f, LocalDate.parse("2026-11-20")))
        assertEquals("cold", WeatherInsights.tag(f, at("2026-11-20T19:30"), zone, LocalDate.parse("2026-11-20")))
    }

    @Test
    fun `parses the QWeather v7 hourly and daily responses`() = runTest {
        val engine = MockEngine { request ->
            val body = if (request.url.encodedPath.endsWith("/24h")) {
                """{"code":"200","hourly":[{"fxTime":"2026-10-12T22:00+08:00","temp":"16","text":"小雨","pop":"85","precip":"0.4"}]}"""
            } else {
                """{"code":"200","daily":[{"fxDate":"2026-10-12","tempMax":"20","tempMin":"14","textDay":"阴","textNight":"小雨"}]}"""
            }
            assertEquals("116.41,39.90", request.url.parameters["location"])
            assertEquals("key-123", request.headers["X-QW-Api-Key"])
            respond(body, headers = headersOf(HttpHeaders.ContentType, "application/json"))
        }
        val http = HttpClient(engine) { install(ClientContentNegotiation) { json(AppJson) } }
        val forecast = QWeatherSource("api.example.com", "key-123", http, Clock.systemUTC())
            .fetch(City("beijing", "北京", "北京", 116.41, 39.90))!!
        assertEquals(16, forecast.hourly.single().temp)
        assertEquals(true, forecast.hourly.single().wet)
        assertEquals(14, forecast.daily.single().tempMin)
    }
}
