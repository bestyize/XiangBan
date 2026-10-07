package xyz.xiaban.server.admin

import java.sql.ResultSet
import java.sql.SQLException
import java.time.Clock
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneId
import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.server.db.*
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.http.notFound
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.users.TextRules
import xyz.xiaban.server.users.code
import xyz.xiaban.server.users.personalityOf

@Serializable
data class AdminUserRow(
    val id: Long, val email: String?, val emailVerifiedAt: Long?, val nickname: String?, val companionName: String,
    val createdAt: Long, val onboardedAt: Long?, val lastActiveAt: Long, val deviceCount: Long,
    val checkinCount: Long, val hasNote: Boolean,
)
@Serializable data class AdminUserPage(val items: List<AdminUserRow>, val total: Long, val page: Int, val pageSize: Int, val totalPages: Int)
@Serializable data class AdminOverview(val totalUsers: Long, val emailUsers: Long, val guestUsers: Long,
    val newUsers7d: Long, val newEmailUsers7d: Long, val activeUsers7d: Long, val totalCheckins: Long,
    val sentMail24h: Long, val failedMail24h: Long)
@Serializable data class AdminDevice(val createdAt: Long, val lastSeenAt: Long)
@Serializable data class AdminCheckin(val workDay: String, val checkedAt: Long, val title: String)
@Serializable data class AdminUserDetail(val user: AdminUserRow, val cityName: String?, val timezone: String,
    val workStart: String, val workEnd: String, val inviteCode: String, val referralSource: String?, val referredBy: Long?,
    val termsVersion: String?, val termsAcceptedAt: Long?, val overseasConsentAt: Long?, val recallOptIn: Boolean,
    val guardianPaused: Boolean?, val note: String, val noteUpdatedAt: Long?, val noteUpdatedBy: String?,
    val devices: List<AdminDevice>, val recentCheckins: List<AdminCheckin>, val cityCode: String?, val personality: Personality, val payday: Int?)
@Serializable data class AdminNoteUpdate(val note: String)
@Serializable data class AdminUserUpdate(val nickname: String? = null, val companionName: String? = null, val personality: Personality? = null,
    val cityCode: String? = null, val timezone: String? = null, val workStart: String? = null, val workEnd: String? = null, val payday: Int? = null)
@Serializable data class AdminUserDelete(val confirmUserId: Long)
@Serializable data class AdminAuditRow(val id: Long, val actor: String, val action: String, val targetUserId: Long?, val summary: String, val createdAt: Long)
@Serializable data class AdminAuditPage(val items: List<AdminAuditRow>, val total: Long, val page: Int, val pageSize: Int, val totalPages: Int)

class AdminUsers(private val db: Db, private val clock: Clock, private val cities: Cities) {
    suspend fun list(page: Int, pageSize: Int, q: String, type: String, onboarding: String, sort: String, from: String, to: String): AdminUserPage {
        validatePage(page, pageSize)
        if (q.length > 100) badRequest("搜索词最多 100 个字符")
        val where = mutableListOf("true")
        val params = mutableListOf<Any?>()
        if (q.isNotBlank()) {
            val pattern = "%${q.trim().replace("!", "!!").replace("%", "!%").replace("_", "!_")}%"
            where += "(u.email ilike ? escape '!' or up.nickname ilike ? escape '!' or c.name ilike ? escape '!' or u.invite_code ilike ? escape '!' or u.id = ?)"
            repeat(4) { params += pattern }; params += q.trim().toLongOrNull() ?: -1L
        }
        when (type) { "all" -> Unit; "email" -> where += "u.email is not null"; "guest" -> where += "u.email is null"; else -> badRequest("账号类型不正确") }
        when (onboarding) { "all" -> Unit; "done" -> where += "u.onboarded_at is not null"; "pending" -> where += "u.onboarded_at is null"; else -> badRequest("引导状态不正确") }
        val start = parseDate(from); val end = parseDate(to)
        if (start != null && end != null && start > end) badRequest("开始日期不能晚于结束日期")
        if (start != null) { where += "u.created_at >= ?"; params += start.atStartOfDay(ZONE).toInstant() }
        if (end != null) { where += "u.created_at < ?"; params += end.plusDays(1).atStartOfDay(ZONE).toInstant() }
        val order = when (sort) {
            "newest" -> "u.created_at desc, u.id desc"
            "oldest" -> "u.created_at asc, u.id asc"
            "active" -> "activity.last_active_at desc, u.id desc"
            "verified" -> "u.email_verified_at desc nulls last, u.id desc"
            else -> badRequest("排序方式不正确")
        }
        return db.read { c ->
            val filter = where.joinToString(" and ")
            val total = c.queryOne("select count(*) $BASE where $filter", *params.toTypedArray()) { it.getLong(1) }!!
            val pages = pages(total, pageSize)
            val effectivePage = page.coerceAtMost(pages)
            val items = c.query("select $ROW $BASE where $filter order by $order limit ? offset ?",
                *(params + pageSize + (effectivePage.toLong() - 1) * pageSize).toTypedArray(), map = ::row)
            AdminUserPage(items, total, effectivePage, pageSize, pages)
        }
    }

    suspend fun overview(): AdminOverview = db.read { c ->
        val week = clock.instant().minusSeconds(7 * 86400L)
        val day = clock.instant().minusSeconds(86400L)
        val counts = c.queryOne("""select count(*), count(*) filter(where u.email is not null), count(*) filter(where u.email is null),
            count(*) filter(where u.created_at >= ?), count(*) filter(where u.email is not null and u.email_verified_at >= ?),
            count(*) filter(where activity.last_active_at >= ?) $BASE""", week, week, week) { rs -> (1..6).map { rs.getLong(it) } }!!
        val checkins = c.queryOne("select count(*) from checkins where revoked_at is null") { it.getLong(1) }!!
        val mail = c.queryOne("select count(*) filter(where ok), count(*) filter(where not ok) from mail_logs where created_at >= ?", day) { it.getLong(1) to it.getLong(2) }!!
        AdminOverview(counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], checkins, mail.first, mail.second)
    }

    suspend fun detail(id: Long): AdminUserDetail = db.read { c ->
        c.queryOne("""select $ROW, u.city_code, u.timezone, u.work_start, u.work_end, u.invite_code, u.ref_source, u.referred_by,
            u.terms_version, u.terms_accepted_at, u.overseas_consent_at, u.recall_opt_in, u.payday, c.personality,
            n.note, n.updated_at as note_updated_at, n.updated_by,
            (select paused from guardians where user_id = u.id) as guardian_paused $BASE where u.id = ?""", id) { rs ->
            val devices = c.query("select created_at, last_seen_at from devices where user_id = ? order by last_seen_at desc limit 20", id) {
                AdminDevice(it.instant("created_at")!!.toEpochMilli(), it.instant("last_seen_at")!!.toEpochMilli()) }
            val checkins = c.query("select work_day, checked_at, title from checkins where user_id = ? and revoked_at is null order by work_day desc limit 10", id) {
                AdminCheckin(it.localDate("work_day").toString(), it.instant("checked_at")!!.toEpochMilli(), it.getString("title")) }
            AdminUserDetail(row(rs), cities.find(rs.getString("city_code"))?.name, rs.getString("timezone"),
                rs.localTime("work_start").toString(), rs.localTime("work_end").toString(), rs.getString("invite_code"), rs.getString("ref_source"), rs.longOrNull("referred_by"),
                rs.getString("terms_version"), rs.instant("terms_accepted_at")?.toEpochMilli(), rs.instant("overseas_consent_at")?.toEpochMilli(), rs.getBoolean("recall_opt_in"),
                rs.getObject("guardian_paused") as Boolean?, rs.getString("note") ?: "", rs.instant("note_updated_at")?.toEpochMilli(), rs.getString("updated_by"), devices, checkins,
                rs.getString("city_code"), personalityOf(rs.getString("personality")), rs.intOrNull("payday"))
        } ?: notFound("没有找到这个用户", "user_not_found")
    }

    suspend fun saveNote(id: Long, note: String, actor: String) {
        if (note.length > 2000) badRequest("备注最多 2000 个字符")
        db.tx { c ->
            c.queryOne("select id from users where id = ? for update", id) { it.getLong(1) } ?: notFound("没有找到这个用户", "user_not_found")
            c.execute("insert into admin_user_notes(user_id,note,updated_by) values (?,?,?) on conflict(user_id) do update set note=excluded.note,updated_by=excluded.updated_by,updated_at=now()", id, note.trim(), actor)
            c.execute("insert into admin_audit_log(actor,action,target_user_id,summary) values (?,?,?,?)", actor, "user_note", id, if (note.isBlank()) "清空用户运营备注" else "更新用户运营备注")
        }
    }

    suspend fun update(id: Long, request: AdminUserUpdate, actor: String) {
        val userFields = mutableListOf<Pair<String, Any?>>()
        val companionFields = mutableListOf<Pair<String, Any?>>()
        val identityFields = mutableListOf<Pair<String, Any?>>()
        val labels = mutableListOf<String>()
        request.nickname?.let {
            val nickname = TextRules.nickname(it)
            identityFields += "nickname" to nickname; identityFields += "nickname_key" to nickname?.let(TextRules::nicknameKey); labels += "昵称"
        }
        request.companionName?.let { companionFields += "name" to TextRules.companionName(it); labels += "小伴名字" }
        request.personality?.let { companionFields += "personality" to it.code; labels += "性格" }
        request.cityCode?.let {
            val city = it.trim().ifEmpty { null }
            if (city != null && cities.find(city) == null) badRequest("没有找到这个城市")
            userFields += "city_code" to city; labels += "城市"
        }
        request.timezone?.let {
            if (it.length > 64 || runCatching { ZoneId.of(it) }.isFailure) badRequest("时区不正确")
            userFields += "timezone" to it; labels += "时区"
        }
        fun time(raw: String): LocalTime {
            if (!Regex("^[0-9]{2}:[0-9]{2}$").matches(raw)) badRequest("时间格式应为 HH:mm")
            return runCatching { LocalTime.parse(raw) }.getOrElse { badRequest("时间不正确") }
        }
        request.workStart?.let { userFields += "work_start" to time(it); labels += "上班时间" }
        request.workEnd?.let { userFields += "work_end" to time(it); labels += "下班时间" }
        request.payday?.let {
            if (it !in 0..31) badRequest("发薪日应为 1–31，或 0 表示未设置")
            userFields += "payday" to it.takeIf { day -> day != 0 }; labels += "发薪日"
        }
        if (labels.isEmpty()) badRequest("没有需要修改的资料")
        try { db.tx { c ->
            c.queryOne("select id from users where id = ? for update", id) { it.getLong(1) } ?: notFound("没有找到这个用户", "user_not_found")
            if (userFields.isNotEmpty()) c.execute("update users set ${userFields.joinToString { "${it.first}=?" }} where id=?", *(userFields.map { it.second } + id).toTypedArray())
            if (identityFields.isNotEmpty()) c.execute("update user_profiles set ${identityFields.joinToString { "${it.first}=?" }},updated_at=now() where user_id=?", *(identityFields.map { it.second } + id).toTypedArray())
            if (companionFields.isNotEmpty()) c.execute("update companions set ${companionFields.joinToString { "${it.first}=?" }},updated_at=now() where user_id=?", *(companionFields.map { it.second } + id).toTypedArray())
            c.execute("insert into admin_audit_log(actor,action,target_user_id,summary) values (?,?,?,?)", actor, "user_edit", id, "编辑用户资料：${labels.joinToString("、")}")
        } } catch (e: SQLException) { if (e.sqlState == "23505") conflict("这个昵称已有人使用，换一个吧", "nickname_taken"); throw e }
    }

    suspend fun delete(id: Long, confirmation: Long, actor: String) {
        if (confirmation != id) badRequest("确认的用户编号不一致，删除已取消")
        db.tx { c ->
            c.queryOne("select id from users where id = ? for update", id) { it.getLong(1) } ?: notFound("没有找到这个用户", "user_not_found")
            c.execute("delete from users where id=?", id)
            c.execute("insert into admin_audit_log(actor,action,summary) values (?,?,?)", actor, "user_delete", "删除用户 #$id 及其关联账号数据")
        }
    }

    suspend fun auditPage(page: Int, pageSize: Int): AdminAuditPage {
        validatePage(page, pageSize)
        return db.read { c ->
            val total = c.queryOne("select count(*) from admin_audit_log") { it.getLong(1) }!!
            val pages = pages(total, pageSize); val effectivePage = page.coerceAtMost(pages)
            val items = c.query("select id,actor,action,target_user_id,summary,created_at from admin_audit_log order by created_at desc,id desc limit ? offset ?", pageSize, (effectivePage.toLong() - 1) * pageSize) {
                AdminAuditRow(it.getLong("id"), it.getString("actor"), it.getString("action"), it.longOrNull("target_user_id"), it.getString("summary"), it.instant("created_at")!!.toEpochMilli()) }
            AdminAuditPage(items, total, effectivePage, pageSize, pages)
        }
    }

    private fun row(rs: ResultSet) = AdminUserRow(rs.getLong("id"), rs.getString("email"), rs.instant("email_verified_at")?.toEpochMilli(),
        rs.getString("nickname"), rs.getString("companion_name"), rs.instant("created_at")!!.toEpochMilli(), rs.instant("onboarded_at")?.toEpochMilli(),
        rs.instant("last_active_at")!!.toEpochMilli(), rs.getLong("device_count"), rs.getLong("checkin_count"), rs.getBoolean("has_note"))

    private fun parseDate(value: String): LocalDate? {
        if (value.isEmpty()) return null
        return runCatching { LocalDate.parse(value).also { if (it.year !in 1970..9998) badRequest("日期范围不正确") } }.getOrElse { badRequest("日期格式应为 YYYY-MM-DD") }
    }
    private fun validatePage(page: Int, size: Int) { if (page < 1 || size !in 1..100) badRequest("分页参数不正确，每页最多 100 条") }
    private fun pages(total: Long, size: Int) = ((total + size - 1) / size).coerceIn(1, Int.MAX_VALUE.toLong()).toInt()

    companion object {
        private val ZONE = ZoneId.of("Asia/Shanghai")
        private const val ROW = """u.id,u.email,u.email_verified_at,up.nickname,c.name as companion_name,u.created_at,u.onboarded_at,
            activity.last_active_at,d.device_count,p.checkin_count,(coalesce(n.note,'') <> '') as has_note"""
        private const val BASE = """from users u join companions c on c.user_id=u.id
            join user_profiles up on up.user_id=u.id
            left join lateral(select count(*) as device_count,max(last_seen_at) as last_seen_at from devices where user_id=u.id) d on true
            left join lateral(select count(*) filter(where revoked_at is null) as checkin_count,max(checked_at) as last_checkin_at from checkins where user_id=u.id) p on true
            left join lateral(select max(created_at) as last_event_at from events where user_id=u.id) e on true
            left join lateral(select greatest(u.created_at,d.last_seen_at,p.last_checkin_at,e.last_event_at) as last_active_at) activity on true
            left join admin_user_notes n on n.user_id=u.id"""
    }
}
