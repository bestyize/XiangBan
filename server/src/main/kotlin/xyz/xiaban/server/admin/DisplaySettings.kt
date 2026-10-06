package xyz.xiaban.server.admin

import org.slf4j.LoggerFactory
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.shared.DisplayItem
import xyz.xiaban.shared.DisplaySettings

object DisplayCatalog {
    val games = listOf(
        DisplayItem("falling-blocks", true, "俄罗斯方块"),
        DisplayItem("merge-2048", true, "2048"),
        DisplayItem("block-puzzle", true, "消失高手"),
        DisplayItem("water-sort", true, "奶茶大师"),
        DisplayItem("snake-arena", true, "小伴贪吃蛇"),
    )
    val quizzes = listOf(
        DisplayItem("mbti", true, "MBTI"), DisplayItem("mbanti", true, "M班TI"),
        DisplayItem("animal", true, "精神小动物"), DisplayItem("afterhours", true, "下班第二人格"),
    )
    fun defaults() = DisplaySettings(true, games, quizzes)

    /** Retain stored order and visibility; append newly shipped items and use current titles. */
    fun normalize(settings: DisplaySettings): DisplaySettings {
        fun items(saved: List<DisplayItem>, catalog: List<DisplayItem>): List<DisplayItem> {
            val known = catalog.associateBy { it.id }
            val seen = mutableSetOf<String>()
            val ordered = saved.mapNotNull { item ->
                known[item.id]?.takeIf { seen.add(item.id) }?.copy(visible = item.visible)
            }
            return ordered + catalog.filter { it.id !in seen }
        }
        return settings.copy(games = items(settings.games, games), quizzes = items(settings.quizzes, quizzes))
    }

    fun validate(settings: DisplaySettings) {
        fun items(values: List<DisplayItem>, catalog: List<DisplayItem>, label: String) {
            if (values.size > 100) badRequest("$label 配置项目过多")
            if (values.map { it.id }.distinct().size != values.size) badRequest("$label 项目不能重复")
            val known = catalog.map { it.id }.toSet()
            if (values.any { it.id !in known }) badRequest("包含未找到的$label 项目")
        }
        items(settings.games, games, "游戏")
        items(settings.quizzes, quizzes, "测试")
    }
}

class DisplaySettingsStore internal constructor(
    private val read: suspend () -> String?,
    private val write: suspend (String, String?) -> Unit,
) {
    constructor(db: Db) : this(
        read = { db.read { c -> c.queryOne("select config::text from site_display_settings where id=1") { it.getString(1) } } },
        write = { raw, actor ->
            db.tx { c ->
                c.execute("insert into site_display_settings(id,config) values (1,?::jsonb) on conflict(id) do update set config=excluded.config,updated_at=now()", raw)
                if (actor != null) c.execute("insert into admin_audit_log(actor,action,summary) values (?,?,?)", actor, "display_settings", "更新小伴入口、游戏与趣味测试的展示和排序配置")
            }
            Unit
        },
    )

    suspend fun current(): DisplaySettings {
        val raw = read() ?: return DisplayCatalog.defaults()
        val saved = runCatching { AppJson.decodeFromString(DisplaySettings.serializer(), raw) }.getOrElse {
            log.warn("Invalid display settings; using the default catalog")
            return DisplayCatalog.defaults()
        }
        return DisplayCatalog.normalize(saved)
    }

    suspend fun save(request: DisplaySettings, actor: String): DisplaySettings {
        DisplayCatalog.validate(request)
        val next = DisplayCatalog.normalize(request)
        write(AppJson.encodeToString(DisplaySettings.serializer(), next), actor)
        return next
    }

    private companion object { val log = LoggerFactory.getLogger(DisplaySettingsStore::class.java) }
}
