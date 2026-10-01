package xyz.xiaban.server.redis

import io.lettuce.core.ClientOptions
import io.lettuce.core.RedisClient
import io.lettuce.core.TimeoutOptions
import io.lettuce.core.api.StatefulRedisConnection
import io.lettuce.core.api.async.RedisAsyncCommands
import java.time.Duration

class Redis private constructor(private val client: RedisClient) : AutoCloseable {
    private val shared: StatefulRedisConnection<String, String> = client.connect()

    val cmd: RedisAsyncCommands<String, String> = shared.async()

    /** Blocking commands (XREADGROUP BLOCK) must not share the pipelined connection. */
    fun dedicatedConnection(): StatefulRedisConnection<String, String> = client.connect()

    override fun close() {
        shared.close()
        client.shutdown()
    }

    companion object {
        fun connect(url: String): Redis {
            val client = RedisClient.create(url)
            client.options = ClientOptions.builder()
                .timeoutOptions(TimeoutOptions.enabled(Duration.ofSeconds(5)))
                .build()
            return Redis(client)
        }
    }
}
