package xyz.xiaban.server.users

import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.instant
import xyz.xiaban.server.db.intOrNull
import xyz.xiaban.server.db.localTime
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.shared.Personality
import java.sql.ResultSet
import java.time.Instant
import java.time.LocalTime
import java.time.ZoneId

data class User(
    val id: Long,
    val cityCode: String?,
    val timezone: ZoneId,
    val workStart: LocalTime,
    val workEnd: LocalTime,
    val onboardedAt: Instant?,
    val createdAt: Instant,
    val companionName: String,
    val personality: Personality,
    val nickname: String?,
    val payday: Int?,
    val inviteCode: String,
    val showCityOnCard: Boolean,
    /** Only set once verified */
    val email: String?,
    val recallOptIn: Boolean,
)

/** Where a new account came from: the inviter's code and the entry point, both from the QR link. */
data class Referral(val inviteCode: String?, val source: String?)

data class ProfileChanges(
    val companionName: String? = null,
    val personality: Personality? = null,
    val nicknameChanged: Boolean = false,
    val nickname: String? = null,
    val cityCode: String? = null,
    val workStart: LocalTime? = null,
    val workEnd: LocalTime? = null,
    val paydayChanged: Boolean = false,
    val payday: Int? = null,
    val showCityOnCard: Boolean? = null,
    val markOnboarded: Boolean = false,
)

val Personality.code: String get() = name.lowercase()

fun personalityOf(code: String): Personality = Personality.entries.firstOrNull { it.code == code } ?: Personality.Toxic

class UserRepository(private val db: Db) {

    /** Serialised per device so concurrent first visits from one browser create a single account. */
    suspend fun findOrCreate(deviceHash: String, timezone: String, referral: Referral? = null): Long = db.tx { c ->
        c.queryOne("select pg_advisory_xact_lock(hashtext(?))", deviceHash) { }
        c.queryOne("update devices set last_seen_at = now() where device_hash = ? returning user_id", deviceHash) {
            it.getLong(1)
        } ?: run {
            val referrer = referral?.inviteCode?.let { code ->
                c.queryOne("select id from users where invite_code = ?", code) { it.getLong(1) }
            }
            val id = c.queryOne(
                "insert into users (timezone, invite_code, referred_by, ref_source) values (?, ?, ?, ?) returning id",
                timezone, newInviteCode(), referrer, referral?.source,
            ) { it.getLong(1) }!!
            c.execute("insert into companions (user_id) values (?)", id)
            c.execute("insert into devices (device_hash, user_id) values (?, ?)", deviceHash, id)
            id
        }
    }

    suspend fun linkDevice(deviceHash: String, userId: Long) = db.read { c ->
        c.execute(
            """
            insert into devices (device_hash, user_id) values (?, ?)
            on conflict (device_hash) do update set user_id = excluded.user_id, last_seen_at = now()
            """,
            deviceHash, userId,
        )
    }

    suspend fun find(userId: Long): User? = db.read { c -> c.queryOne("$SELECT_USER where u.id = ?", userId, map = ::mapUser) }

    suspend fun findIdByEmail(email: String): Long? = db.read { c ->
        c.queryOne("select id from users where email = ?", email) { it.getLong(1) }
    }

    suspend fun setEmail(userId: Long, email: String) = db.read { c ->
        c.execute("update users set email = ?, email_verified_at = now() where id = ?", email, userId)
    }

    suspend fun setRecallOptIn(userId: Long, enabled: Boolean) = db.read { c ->
        c.execute("update users set recall_opt_in = ?, recall_count = 0 where id = ?", enabled, userId)
    }

    /** A check-in ends the lapse, so the next one may again get up to two reminders. */
    suspend fun resetRecall(userId: Long) = db.read { c ->
        c.execute("update users set recall_count = 0 where id = ? and recall_count <> 0", userId)
    }

    suspend fun setTimezone(userId: Long, timezone: String) = db.read { c ->
        c.execute("update users set timezone = ? where id = ? and timezone <> ?", timezone, userId, timezone)
    }

    suspend fun update(userId: Long, changes: ProfileChanges) = db.tx { c ->
        val userSets = buildList {
            changes.cityCode?.let { add("city_code = ?" to it) }
            changes.workStart?.let { add("work_start = ?" to it) }
            changes.workEnd?.let { add("work_end = ?" to it) }
            if (changes.paydayChanged) add("payday = ?" to changes.payday)
            changes.showCityOnCard?.let { add("show_city_on_card = ?" to it) }
            if (changes.markOnboarded) add("onboarded_at = coalesce(onboarded_at, now())" to null)
        }
        val companionSets = buildList {
            changes.companionName?.let { add("name = ?" to it) }
            changes.personality?.let { add("personality = ?" to it.code) }
            if (changes.nicknameChanged) add("nickname = ?" to changes.nickname)
        }
        c.applySets("users", "id", userId, userSets)
        if (companionSets.isNotEmpty()) {
            c.applySets("companions", "user_id", userId, companionSets + ("updated_at = now()" to null))
        }
    }

    private fun java.sql.Connection.applySets(table: String, key: String, id: Long, sets: List<Pair<String, Any?>>) {
        if (sets.isEmpty()) return
        val params = sets.filter { it.first.contains('?') }.map { it.second } + id
        execute("update $table set ${sets.joinToString { it.first }} where $key = ?", *params.toTypedArray())
    }

    private fun mapUser(rs: ResultSet) = User(
        id = rs.getLong("id"),
        cityCode = rs.getString("city_code"),
        timezone = runCatching { ZoneId.of(rs.getString("timezone")) }.getOrDefault(DEFAULT_ZONE),
        workStart = rs.localTime("work_start"),
        workEnd = rs.localTime("work_end"),
        onboardedAt = rs.instant("onboarded_at"),
        createdAt = rs.instant("created_at")!!,
        companionName = rs.getString("name"),
        personality = personalityOf(rs.getString("personality")),
        nickname = rs.getString("nickname"),
        payday = rs.intOrNull("payday"),
        inviteCode = rs.getString("invite_code"),
        showCityOnCard = rs.getBoolean("show_city_on_card"),
        email = rs.getString("email"),
        recallOptIn = rs.getBoolean("recall_opt_in"),
    )

    companion object {
        val DEFAULT_ZONE: ZoneId = ZoneId.of("Asia/Shanghai")

        private const val INVITE_ALPHABET = "23456789abcdefghjkmnpqrstuvwxyz"
        private val random = java.security.SecureRandom()

        /** 10 characters from 31 symbols (~49 bits): collisions stay negligible at millions of users. */
        fun newInviteCode(): String = (1..10).map { INVITE_ALPHABET[random.nextInt(INVITE_ALPHABET.length)] }.joinToString("")

        private const val SELECT_USER = """
            select u.id, u.city_code, u.timezone, u.work_start, u.work_end, u.payday, u.onboarded_at, u.created_at,
                   u.invite_code, u.show_city_on_card, u.email, u.recall_opt_in, c.name, c.personality, c.nickname
            from users u join companions c on c.user_id = u.id
        """
    }
}
