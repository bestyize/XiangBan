package xyz.xiangban.xiaoban

/** A public runtime identity. The link cannot specify an entry file, network permission or download URL. */
internal sealed interface WebWorkLink {
    data class Builtin(val id: String) : WebWorkLink
    data class Idea(val id: String) : WebWorkLink
    data class Pack(val id: Long, val version: Int) : WebWorkLink

    companion object {
        private const val prefix = "xiaoban://community/run/"
        private val slug = Regex("[a-z0-9][a-z0-9-]{1,39}")

        fun parse(raw: String?): WebWorkLink? {
            if (raw == null || raw.length > 120 || !raw.startsWith(prefix)) return null
            val parts = raw.removePrefix(prefix).split('/')
            if (parts.size == 2 && slug.matches(parts[1])) return when (parts[0]) {
                "builtin" -> Builtin(parts[1])
                "idea" -> Idea(parts[1])
                else -> null
            }
            if (parts.size != 3 || parts[0] != "pack") return null
            val id = parts[1].toLongOrNull() ?: return null
            val version = parts[2].toIntOrNull() ?: return null
            return if (id in 1L..9_007_199_254_740_991L && id.toString() == parts[1] &&
                version > 0 && version.toString() == parts[2]) Pack(id, version) else null
        }
    }
}
