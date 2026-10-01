package xyz.xiaban.server.support

import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.ZoneOffset

class MutableClock(@Volatile var now: Instant = Instant.now()) : Clock() {
    override fun getZone(): ZoneId = ZoneOffset.UTC
    override fun withZone(zone: ZoneId): Clock = this
    override fun instant(): Instant = now

    /** Sets the clock from a Beijing-time string such as "2026-09-29T21:37". */
    fun setBeijing(local: String) {
        now = OffsetDateTime.parse("$local:00+08:00").toInstant()
    }

    fun advance(duration: Duration) {
        now = now.plus(duration)
    }
}
