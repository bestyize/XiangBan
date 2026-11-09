package xyz.xiangban.xiaoban

import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.abs

/** An AC codebook captured from a physical remote, indexed by the requested settings. */
internal class RuntimeV2AcCapture(data: JSONObject) {
    val id: String = data.getString("id")
    private val off = data.getJSONArray("off")
    private val settings = data.getJSONObject("settings")
    val catalog = JSONObject()
        .put("id", id)
        .put("models", data.getJSONArray("models"))
        .put("keys", JSONArray(listOf("powerOn", "powerOff", "fan", "tempUp", "tempDown") + settings.keys().asSequence().toList()))

    data class Command(val pattern: IntArray, val state: JSONObject)

    fun command(action: String, previous: JSONObject?): Command? {
        val previousMode = previous?.optString("mode").orEmpty()
        val mode = when {
            settings.has(action) -> action
            settings.has(previousMode) -> previousMode
            else -> "cool"
        }
        val speeds = settings.getJSONObject(mode)
        val speedNames = speeds.keys().asSequence().sortedBy { it.lowercase() }.toList()
        val previousSpeed = previous?.optString("fan").orEmpty()
        val speed = when {
            action == "fan" && previousSpeed in speedNames -> speedNames[(speedNames.indexOf(previousSpeed) + 1) % speedNames.size]
            previousSpeed in speedNames -> previousSpeed
            "auto" in speedNames -> "auto"
            else -> speedNames.first()
        }
        val captures = speeds.getJSONObject(speed)
        val temperatures = captures.keys().asSequence().map(String::toInt).sorted().toList()
        val requested = previous?.optInt("temperature", 26) ?: 26
        val temperature = when (action) {
            "tempUp" -> temperatures.firstOrNull { it > requested } ?: temperatures.last()
            "tempDown" -> temperatures.lastOrNull { it < requested } ?: temperatures.first()
            else -> temperatures.minBy { abs(it.toLong() - requested) }
        }
        val capture = when (action) {
            "powerOff" -> off
            "power", "powerOn", "fan", "tempUp", "tempDown" -> captures.getJSONArray(temperature.toString())
            else -> if (settings.has(action)) captures.getJSONArray(temperature.toString()) else return null
        }
        val pattern = IntArray(capture.length()) { capture.getInt(it) }
        require(pattern.size in 2..1024 && pattern.size % 2 == 0)
        require(pattern.all { it > 0 } && pattern.sumOf { it.toLong() } <= 1_000_000)
        return Command(
            pattern,
            JSONObject()
                .put("mode", mode)
                .put("fan", speed)
                .put("temperature", temperature)
                .put("on", action != "powerOff"),
        )
    }

    companion object {
        const val FREQUENCY = 38_000
    }
}
