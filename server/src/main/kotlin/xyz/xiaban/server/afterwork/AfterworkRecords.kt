package xyz.xiaban.server.afterwork

import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.MapSerializer
import kotlinx.serialization.builtins.serializer
import xyz.xiaban.server.db.*
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest
import java.time.LocalDate
import java.time.Instant

@Serializable data class AfterworkEntry(val workDay: String, val activityId: String, val selectedAt: Long)
@Serializable data class AfterworkData(val entries: List<AfterworkEntry> = emptyList(), val preferredActivityId: String? = null, val promptDismissedAt: Map<String, Int> = emptyMap())
@Serializable data class AfterworkChoice(val workDay: String, val activityId: String)
@Serializable data class AfterworkPreference(val preferredActivityChanged: Boolean = false, val preferredActivityId: String? = null, val dismissedActivityId: String? = null, val dismissedCount: Int = 0)

class AfterworkRecords(private val db: Db) {
    suspend fun data(userId: Long): AfterworkData = db.read { c ->
        val entries = c.query("select work_day,activity_id,selected_at from afterwork_choices where user_id=? order by work_day desc limit 90", userId) {
            AfterworkEntry(it.localDate("work_day").toString(), it.getString("activity_id"), it.instant("selected_at")!!.toEpochMilli()) }.reversed()
        c.queryOne("select preferred_activity_id,prompt_dismissed_at::text from afterwork_preferences where user_id=?", userId) {
            AfterworkData(entries, it.getString(1), AppJson.decodeFromString(MAP, it.getString(2))) } ?: AfterworkData(entries)
    }
    suspend fun choose(userId: Long, choice: AfterworkChoice): AfterworkData {
        val day = day(choice.workDay); activity(choice.activityId)
        db.read { c -> c.execute("insert into afterwork_choices(user_id,work_day,activity_id) values (?,?,?) on conflict(user_id,work_day) do update set activity_id=excluded.activity_id,selected_at=now()", userId, day, choice.activityId) }
        return data(userId)
    }
    suspend fun preference(userId: Long, request: AfterworkPreference): AfterworkData {
        request.preferredActivityId?.let(::activity); request.dismissedActivityId?.let(::activity)
        if (request.dismissedCount !in 0..1000) badRequest("习惯提示次数不正确")
        db.tx { c ->
            c.execute("insert into afterwork_preferences(user_id) values (?) on conflict do nothing", userId)
            if (request.preferredActivityChanged) c.execute("update afterwork_preferences set preferred_activity_id=? where user_id=?", request.preferredActivityId?.takeUnless { it == "no_plan" }, userId)
            if (request.dismissedActivityId != null) c.execute("update afterwork_preferences set prompt_dismissed_at=prompt_dismissed_at || ?::jsonb where user_id=?",
                AppJson.encodeToString(MAP, mapOf(request.dismissedActivityId to request.dismissedCount)), userId)
        }
        return data(userId)
    }
    suspend fun importData(userId: Long, incoming: AfterworkData): AfterworkData {
        if (incoming.entries.size > 90 || incoming.promptDismissedAt.size > 7) badRequest("导入记录过多")
        incoming.entries.forEach { day(it.workDay); activity(it.activityId); if (it.selectedAt !in 1..4102444800000L) badRequest("记录时间不正确") }
        incoming.preferredActivityId?.let(::activity)
        incoming.promptDismissedAt.forEach { (key, count) -> activity(key); if (count !in 0..1000) badRequest("提示次数不正确") }
        db.tx { c ->
            incoming.entries.forEach { c.execute("insert into afterwork_choices(user_id,work_day,activity_id,selected_at) values (?,?,?,?) on conflict do nothing", userId, day(it.workDay), it.activityId, Instant.ofEpochMilli(it.selectedAt)) }
            c.execute("insert into afterwork_preferences(user_id,preferred_activity_id,prompt_dismissed_at) values (?,?,?::jsonb) on conflict do nothing", userId,
                incoming.preferredActivityId?.takeUnless { it == "no_plan" }, AppJson.encodeToString(MAP, incoming.promptDismissedAt))
        }
        return data(userId)
    }
    suspend fun clear(userId: Long) = db.tx { c ->
        c.execute("delete from afterwork_choices where user_id=?", userId); c.execute("delete from afterwork_preferences where user_id=?", userId)
    }
    private fun activity(value: String) { if (value !in ACTIVITIES) badRequest("下班安排不正确") }
    private fun day(value: String): LocalDate = runCatching { LocalDate.parse(value).also { if (it.year !in 1970..2100) badRequest("日期范围不正确") } }.getOrElse { badRequest("日期格式不正确") }
    companion object {
        private val ACTIVITIES = setOf("rest", "walk", "food", "move", "entertainment", "company", "no_plan")
        private val MAP = MapSerializer(String.serializer(), Int.serializer())
    }
}
