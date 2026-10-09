package xyz.xiaban.wallpaper.data

import org.json.JSONArray
import org.json.JSONObject
import java.io.File

/** A small JSON file replaced atomically, so a crash never leaves half a list behind. */
internal class JsonFile(private val file: File) {
    fun read(): JSONArray = try {
        JSONArray(file.readText())
    } catch (_: Exception) {
        JSONArray()
    }

    fun write(array: JSONArray) {
        file.parentFile?.mkdirs()
        val staging = File(file.parentFile, file.name + ".new")
        staging.writeText(array.toString())
        if (!staging.renameTo(file)) {
            file.delete()
            staging.renameTo(file)
        }
    }
}

/** Favourites live in one file on this phone. Nothing here is ever sent to a server. */
class FavoriteStore(file: File) {
    private val json = JsonFile(file)

    @Synchronized
    fun list(): List<Favorite> {
        val array = json.read()
        return (0 until array.length()).mapNotNull {
            val o = array.optJSONObject(it) ?: return@mapNotNull null
            val item = SiteItem.fromJson(o.optJSONObject("item") ?: return@mapNotNull null) ?: return@mapNotNull null
            Favorite(item, o.optLong("at"))
        }
    }

    @Synchronized
    fun contains(id: String) = list().any { it.item.id == id }

    /** Returns true when the item is a favourite afterwards. */
    @Synchronized
    fun toggle(item: SiteItem, now: Long = System.currentTimeMillis()): Boolean {
        val all = list()
        val had = all.any { it.item.id == item.id }
        val next = if (had) all.filter { it.item.id != item.id } else listOf(Favorite(item, now)) + all
        json.write(JSONArray().also { array -> next.forEach { array.put(JSONObject().put("item", it.item.toJson()).put("at", it.savedAt)) } })
        return !had
    }
}

/** What this app has saved into the gallery. Entries whose gallery file was deleted elsewhere drop out on the next read. */
class DownloadStore(file: File, private val exists: (String) -> Boolean) {
    private val json = JsonFile(file)

    @Synchronized
    fun list(): List<Downloaded> {
        val array = json.read()
        val all = (0 until array.length()).mapNotNull { read(array.optJSONObject(it)) }
        val alive = all.filter { exists(it.uri) }
        if (alive.size != all.size) write(alive)
        return alive
    }

    @Synchronized
    fun findByHash(sha256: String) = list().firstOrNull { it.sha256 == sha256 }

    @Synchronized
    fun add(entry: Downloaded) {
        write(listOf(entry) + list().filter { it.id != entry.id })
    }

    @Synchronized
    fun remove(id: String) {
        write(list().filter { it.id != id })
    }

    private fun write(items: List<Downloaded>) {
        json.write(
            JSONArray().also { array ->
                for (d in items) {
                    array.put(
                        JSONObject().put("id", d.id).put("kind", d.kind.wire).put("title", d.title).put("uri", d.uri)
                            .put("sha", d.sha256).put("w", d.width).put("h", d.height).put("at", d.savedAt),
                    )
                }
            },
        )
    }

    private fun read(o: JSONObject?): Downloaded? {
        o ?: return null
        val kind = MediaKind.parse(o.optString("kind")) ?: return null
        val uri = o.optString("uri")
        if (uri.isEmpty() || o.optString("id").isEmpty()) return null
        return Downloaded(o.getString("id"), kind, o.optString("title"), uri, o.optString("sha"), o.optInt("w"), o.optInt("h"), o.optLong("at"))
    }
}
