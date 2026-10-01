package xyz.xiaban.server.users

import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.shared.SessionRequest
import xyz.xiaban.shared.SessionResponse
import java.security.MessageDigest
import java.time.ZoneId

class SessionService(
    private val users: UserRepository,
    private val profiles: ProfileService,
    private val tokens: Tokens,
) {
    /**
     * Resolves the caller to an account: a valid token wins (e.g. issued by the 相伴 App), then the device
     * cookie (survives storage resets better than localStorage), then the device id in the body.
     */
    suspend fun open(tokenUserId: Long?, cookieDeviceId: String?, req: SessionRequest): SessionResponse {
        val deviceId = cookieDeviceId?.takeIf(::isValidDeviceId)
            ?: req.deviceId.takeIf(::isValidDeviceId)
            ?: badRequest("设备标识无效", "invalid_device")
        val deviceHash = sha256(deviceId)
        val timezone = req.timezone?.takeIf(::isValidZone)

        val existing = tokenUserId?.let { users.find(it) }
        val userId = if (existing != null) {
            users.linkDevice(deviceHash, existing.id)
            existing.id
        } else {
            users.findOrCreate(deviceHash, timezone ?: UserRepository.DEFAULT_ZONE.id)
        }
        if (timezone != null) users.setTimezone(userId, timezone)

        return SessionResponse(token = tokens.issue(userId), deviceId = deviceId, profile = profiles.view(userId))
    }

    companion object {
        private val deviceIdPattern = Regex("^[A-Za-z0-9_-]{16,128}$")

        fun isValidDeviceId(value: String) = deviceIdPattern.matches(value)

        fun isValidZone(value: String) = value.length <= 64 && runCatching { ZoneId.of(value) }.isSuccess

        fun sha256(value: String): String = MessageDigest.getInstance("SHA-256")
            .digest(value.toByteArray())
            .joinToString("") { "%02x".format(it) }
    }
}
