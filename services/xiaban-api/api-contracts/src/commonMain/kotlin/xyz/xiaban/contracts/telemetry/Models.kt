package xyz.xiaban.contracts.telemetry

import kotlinx.serialization.Serializable

@Serializable
data class EventIn(val name: String, val ts: Long? = null, val props: Map<String, String> = emptyMap())

/** [deviceId] identifies a guest; it is hashed before storage and ignored for signed-in users. */
@Serializable
data class EventBatch(val events: List<EventIn>, val deviceId: String? = null)
