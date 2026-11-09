package xyz.xiangban.xiaoban

import android.content.Context
import android.hardware.ConsumerIrManager
import android.util.Base64
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.apache.commons.compress.compressors.xz.XZCompressorInputStream
import org.json.JSONArray
import org.json.JSONObject
import java.nio.ByteBuffer

internal class RuntimeV2RemoteControl(context: Context) {
    private val emitter = context.getSystemService(ConsumerIrManager::class.java)
    private val assets = context.assets
    private var library: Library? = null
    private val alternateKeys = mutableSetOf<Pair<String, String>>()
    private val acCaptures by lazy {
        assets.open(MIDEA_ASSET).use { bytes ->
            val data = XZCompressorInputStream(bytes, false, 80 * 1024)
                .bufferedReader(Charsets.UTF_8).use { JSONArray(it.readText()) }
            (0 until data.length()).map { RuntimeV2AcCapture(data.getJSONObject(it)) }.associateBy { it.id }
        }
    }

    fun supported(): Boolean = runCatching { emitter?.hasIrEmitter() == true }.getOrDefault(false)

    fun catalog(): JSONObject =
        (library ?: readLibrary().also { library = it }).catalog

    data class SendResult(val error: String? = null, val state: JSONObject? = null)

    fun send(profileId: String, action: String, state: JSONObject?): SendResult {
        val capture = acCaptures[profileId] ?: return SendResult(error = send(profileId, action))
        val manager = emitter ?: return SendResult(error = "unsupported")
        if (!manager.hasIrEmitter()) return SendResult(error = "unsupported")
        val command = capture.command(action, state) ?: return SendResult(error = "unsupported")
        manager.transmit(RuntimeV2AcCapture.FREQUENCY, command.pattern)
        return SendResult(state = command.state)
    }

    /** Only the reviewed offline snapshot supplies carrier frequencies and pulse timings. */
    fun send(profileId: String, action: String): String? {
        val profile = library?.profiles?.get(profileId) ?: return "invalid"
        val command = profile.commands.optJSONArray(action) ?: return "unsupported"
        val manager = emitter ?: return "unsupported"
        if (!manager.hasIrEmitter()) return "unsupported"
        val toggleKey = profileId to action
        val alternate = command.optString(1).takeIf { it.isNotEmpty() }
        val alternatePattern = if (alternate != null && toggleKey in alternateKeys) {
            runCatching { decode(alternate) }.getOrNull()
        } else null
        val pattern = alternatePattern ?: runCatching { decode(command.getString(0)) }.getOrNull()
            ?: return "invalid"
        manager.transmit(profile.frequency, pattern)
        if (alternate != null && !alternateKeys.add(toggleKey)) alternateKeys.remove(toggleKey)
        return null
    }

    private data class Profile(val frequency: Int, val commands: JSONObject)
    private data class Library(val catalog: JSONObject, val profiles: Map<String, Profile>)

    private fun readLibrary(): Library {
        val indexes = mutableMapOf<String, MutableMap<String, String>>()
        val brandModels = mutableMapOf<Pair<String, String>, MutableSet<String>>()
        val models = mutableMapOf<String, Profile>()

        assets.open(CODES_ASSET).buffered().use { bytes ->
            TarArchiveInputStream(XZCompressorInputStream(bytes, false, 80 * 1024)).use { tar ->
                while (true) {
                    val entry = tar.nextEntry ?: break
                    val path = entry.name.substringAfter('/')
                    val source = path.substringBefore('/').removeSuffix(".json")
                    val type = when (source) {
                        "1_TV" -> "tv"
                        "3_AC" -> "ac"
                        else -> continue
                    }
                    if (!entry.isFile || entry.size !in 1..512_000 || !path.endsWith(".json")) continue
                    val data = runCatching {
                        JSONObject(tar.readBytes().toString(Charsets.UTF_8)).opt("data")
                    }.getOrNull()
                    when {
                        path == "$source.json" && data is JSONArray -> {
                            val brands = indexes.getOrPut(type) { linkedMapOf() }
                            for (i in 0 until data.length()) {
                                val brand = data.optJSONObject(i) ?: continue
                                val name = brand.optString("name").trim()
                                val id = brand.optString("brandid")
                                if (id.isNotEmpty() && name.isNotEmpty()) brands.putIfAbsent(id, name)
                            }
                        }
                        path.startsWith("$source/models/") && data is JSONObject -> {
                            val id = path.substringAfterLast('/').removeSuffix(".json")
                            val key = "$type:$id"
                            if (key !in models) {
                                models[key] = Profile(data.getInt("frequency"), data.getJSONObject("commands"))
                            }
                        }
                        data is JSONObject -> {
                            val brandId = path.substringAfterLast('_').removeSuffix(".json")
                            brandModels.getOrPut(type to brandId) { linkedSetOf() }.addAll(modelIds(data))
                        }
                    }
                }
            }
        }

        val catalog = JSONObject()
        val profiles = linkedMapOf<String, Profile>()
        for (type in listOf("tv", "ac")) {
            val index = checkNotNull(indexes[type])
            val byName = linkedMapOf<String, MutableSet<String>>()
            for ((id, name) in index) {
                for (model in brandModels[type to id].orEmpty()) {
                    val profileId = "$type:$model"
                    val profile = models[profileId] ?: continue
                    byName.getOrPut(name) { linkedSetOf() }.add(profileId)
                    profiles[profileId] = profile
                }
            }
            catalog.put(type, JSONArray(byName.map { (name, ids) ->
                JSONObject().put("name", name).put("profiles", JSONArray(ids.map { id ->
                    val commands = profiles.getValue(id).commands
                    JSONObject().put("id", id).put("keys", JSONArray(commands.keys().asSequence().toList()))
                }))
            }))
        }
        check(profiles.isNotEmpty()) { "Empty remote code snapshot" }
        catalog.getJSONArray("ac").put(
            JSONObject().put("name", "美的").put("profiles", JSONArray(acCaptures.values.map { it.catalog })),
        )
        return Library(catalog, profiles)
    }

    private fun modelIds(brand: JSONObject): Set<String> = buildSet {
        val nodes = brand.optJSONObject("tree")?.optJSONArray("nodes") ?: JSONArray()
        for (i in 0 until nodes.length()) {
            val ids = nodes.optJSONObject(i)?.optJSONArray("keysetids") ?: continue
            for (j in 0 until ids.length()) add(ids.optString(j))
        }
        val others = brand.optJSONArray("others") ?: JSONArray()
        for (i in 0 until others.length()) others.optJSONObject(i)?.optString("_id")?.let(::add)
    }

    private fun decode(value: String): IntArray {
        val bytes = Base64.decode(value, Base64.DEFAULT)
        require(bytes.size % Int.SIZE_BYTES == 0 && bytes.size / Int.SIZE_BYTES in 2..1024)
        val pattern = IntArray(bytes.size / Int.SIZE_BYTES)
        ByteBuffer.wrap(bytes).asIntBuffer().get(pattern)
        require(pattern.all { it > 0 } && pattern.sumOf { it.toLong() } <= 1_000_000)
        return pattern
    }

    companion object {
        private const val CODES_ASSET = "game-packs-v2/remote-control/codes.tar.xz"
        private const val MIDEA_ASSET = "game-packs-v2/remote-control/midea.json.xz"
    }
}
