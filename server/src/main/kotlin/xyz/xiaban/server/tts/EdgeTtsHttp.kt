package xyz.xiaban.server.tts

import io.ktor.client.HttpClient
import io.ktor.client.HttpClientConfig
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.websocket.WebSockets

/** WebSocket support and generous timeouts are both required by the read-aloud endpoint. */
fun HttpClientConfig<*>.configureEdgeTts() {
    install(WebSockets)
    install(HttpTimeout) {
        requestTimeoutMillis = 60_000
        connectTimeoutMillis = 15_000
        socketTimeoutMillis = 60_000
    }
}

/** A standalone client for callers that do not already own a configured [HttpClient]. */
fun edgeTtsHttpClient(): HttpClient = HttpClient(CIO) { configureEdgeTts() }
