package xyz.xiaban.server.users

import java.sql.ResultSet
import java.time.Instant
import java.time.LocalTime
import java.time.ZoneId
import java.util.UUID
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.instant
import xyz.xiaban.server.db.intOrNull
import xyz.xiaban.server.db.localTime
import xyz.xiaban.server.db.queryOne

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
    val termsVersion: String?,
    val emailVerifiedAt: Instant?,
    val avatarKey: String?,
    /** `sub` of the unified account center; set for every account created or linked through it. */
    val accountUid: UUID? = null,
)

/** Where a new account came from: the inviter's code and the entry point, both from the QR link. */
data class Referral(val inviteCode: String?, val source: String?)

data class ProfileChanges(
    val companionName: String? = null,
    val personality: Personality? = null,
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
    suspend fun findOrCreate(deviceHash: String, timezone: String, referral: Referral? = null, guestOnly: Boolean = false): Long = db.tx { c ->
        c.queryOne("select pg_advisory_xact_lock(hashtext(?))", deviceHash) { }
        if (guestOnly) c.execute("delete from devices where device_hash=? and user_id in (select id from users where email_verified_at is not null or account_uid is not null)", deviceHash)
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
            c.execute("insert into user_profiles (user_id) values (?)", id)
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

    suspend fun exists(userId: Long): Boolean = db.read { c ->
        c.queryOne("select exists(select 1 from users where id = ?)", userId) { it.getBoolean(1) }!!
    }

    suspend fun guestForDevice(deviceHash: String): Long? = db.read { c ->
        c.queryOne("select d.user_id from devices d join users u on u.id=d.user_id where d.device_hash=? and u.email_verified_at is null and u.account_uid is null", deviceHash) { it.getLong(1) }
    }

    suspend fun authState(userId: Long): Boolean? = db.read { c ->
        c.queryOne("select (email is not null and email_verified_at is not null) or account_uid is not null from users where id=?", userId) { it.getBoolean(1) }
    }

    suspend fun findIdByAccountUid(uid: UUID): Long? = db.read { c ->
        c.queryOne("select id from users where account_uid = ?", uid) { it.getLong(1) }
    }

    suspend fun setEmail(userId: Long, email: String) = db.read { c ->
        c.execute("update users set email = ?, email_verified_at = now() where id = ?", email, userId)
    }

    /**
     * Resolves the local account for an account-center uid, in order: already linked; the caller's own guest
     * account, which becomes the registered one; otherwise a new account. [username] is the account center's
     * unique, unchangeable name; it is mirrored into the profile as the display name and re-synced on each
     * login in case an administrator renamed the account.
     */
    suspend fun claimAccount(ownerId: Long, uid: UUID, username: String, email: String?): Long = db.tx { c ->
        c.queryOne("select pg_advisory_xact_lock(hashtext(?))", "account:$uid") { }

        val linked = c.queryOne("select id from users where account_uid = ?", uid) { it.getLong(1) }
        if (linked != null) {
            c.execute("update user_profiles set nickname = ?, updated_at = now() where user_id = ? and nickname is distinct from ?", username, linked, username)
            return@tx linked
        }

        val guest = ownerId > 0 && c.queryOne("select exists(select 1 from users where id = ? and account_uid is null)", ownerId) { it.getBoolean(1) } == true
        if (guest) {
            c.execute("update user_profiles set nickname = ?, updated_at = now() where user_id = ?", username, ownerId)
            c.execute("update users set account_uid = ?, email = ?, email_verified_at = case when ?::text is null then null else now() end where id = ?", uid, email, email, ownerId)
            ownerId
        } else {
            val id = c.queryOne(
                "insert into users (account_uid, email, email_verified_at, timezone, invite_code) values (?, ?, case when ?::text is null then null else now() end, ?, ?) returning id",
                uid, email, email, DEFAULT_ZONE.id, newInviteCode(),
            ) { it.getLong(1) }!!
            c.execute("insert into companions (user_id) values (?)", id)
            c.execute("insert into user_profiles (user_id, nickname) values (?, ?)", id, username)
            id
        }
    }

    suspend fun unlinkDevice(userId: Long, deviceHash: String) = db.read { c ->
        c.execute("delete from devices where user_id = ? and device_hash = ?", userId, deviceHash)
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
        termsVersion = rs.getString("terms_version"),
        emailVerifiedAt = rs.instant("email_verified_at"),
        avatarKey = rs.getString("avatar_key"),
        accountUid = rs.getObject("account_uid", UUID::class.java),
    )

    companion object {
        val DEFAULT_ZONE: ZoneId = ZoneId.of("Asia/Shanghai")

        private const val INVITE_ALPHABET = "23456789abcdefghjkmnpqrstuvwxyz"
        private val random = java.security.SecureRandom()

        /** 10 characters from 31 symbols (~49 bits): collisions stay negligible at millions of users. */
        fun newInviteCode(): String = (1..10).map { INVITE_ALPHABET[random.nextInt(INVITE_ALPHABET.length)] }.joinToString("")

        private const val SELECT_USER = """
            select u.id, u.city_code, u.timezone, u.work_start, u.work_end, u.payday, u.onboarded_at, u.created_at,
                   u.invite_code, u.show_city_on_card, u.email, u.recall_opt_in, u.terms_version,
                   c.name, c.personality, p.nickname, u.email_verified_at, a.asset_key as avatar_key, u.account_uid
            from users u join companions c on c.user_id = u.id
            join user_profiles p on p.user_id=u.id
            left join user_avatars a on a.user_id=u.id
        """
    }
}
