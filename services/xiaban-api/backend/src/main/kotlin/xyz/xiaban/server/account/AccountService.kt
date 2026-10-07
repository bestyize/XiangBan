package xyz.xiaban.server.account

import java.sql.ResultSet
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.query
import xyz.xiaban.server.guardian.GuardianService
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.users.ProfileService

/** PRD 6/11: export everything we hold about the user, delete it all, and record consent. */
class AccountService(private val db: Db, private val profiles: ProfileService, private val guardian: GuardianService) {

    suspend fun export(userId: Long): JsonObject {
        val profile = AppJson.encodeToJsonElement(ProfileView.serializer(), profiles.view(userId))
        return db.read { c ->
            fun rows(sql: String): JsonArray = JsonArray(c.query(sql, userId, map = ::rowToJson))
            buildJsonObject {
                put("exportedFormatVersion", 1)
                put("profile", profile)
                put("identity", rows("select user_id,nickname,updated_at from user_profiles where user_id=?").firstOrNull() ?: JsonNull)
                put("avatar", rows("select user_id,encode(image,'base64') as jpeg_base64,updated_at from user_avatars where user_id=?").firstOrNull() ?: JsonNull)
                put("quizAttempts", rows("select quiz_id,version,answers::text,cursor,completed_at,client_updated_at,updated_at from quiz_attempts where user_id=?"))
                put("gameSessions", rows("select id,game_id,started_at,finished_at,score,lines,max_tile from game_sessions where user_id=? order by started_at"))
                put("gameSaves", rows("select game_id,data::text as data,client_updated_at,updated_at from game_saves where user_id=? order by game_id"))
                put("afterworkChoices", rows("select work_day,activity_id,selected_at from afterwork_choices where user_id=? order by work_day"))
                put("afterworkPreferences", rows("select preferred_activity_id,prompt_dismissed_at::text from afterwork_preferences where user_id=?").firstOrNull() ?: JsonNull)
                put("account", rows("select email, created_at, terms_version, terms_accepted_at, overseas_consent_at, recall_opt_in, timezone from users where id = ?").firstOrNull() ?: JsonNull)
                put("checkins", rows("select work_day, checked_at, title, coalesce(final_reply, library_reply) as reply, reply_source, care_mode, revoked_at from checkins where user_id = ? order by work_day"))
                put("memories", rows("select kind, content, source, created_at from memories where user_id = ? order by created_at"))
                put("unlocks", rows("select expression_id, unlocked_at, reason from unlocks where user_id = ? order by unlocked_at"))
                put("guardian", rows("select contact_name, contact_email, self_name, threshold_days, paused, consented_at from guardians where user_id = ?").firstOrNull() ?: JsonNull)
                put("events", rows("select name, props::text as props, created_at from events where user_id = ? order by created_at desc limit 5000"))
            }
        }
    }

    /** Removes the account and everything that cascades from it; reports keep the text but lose the user. */
    suspend fun delete(userId: Long) {
        guardian.remove(userId)
        db.read { c -> c.execute("delete from users where id = ?", userId) }
    }

    suspend fun consent(userId: Long, termsVersion: String, overseas: Boolean) = db.read { c ->
        c.execute(
            """
            update users set terms_version = ?, terms_accepted_at = now(),
              overseas_consent_at = case when ? then coalesce(overseas_consent_at, now()) else overseas_consent_at end
            where id = ?
            """,
            termsVersion.take(20), overseas, userId,
        )
    }

    private fun rowToJson(rs: ResultSet): JsonElement {
        val meta = rs.metaData
        return JsonObject(
            (1..meta.columnCount).associate { i ->
                val value = rs.getObject(i)
                meta.getColumnLabel(i) to when (value) {
                    null -> JsonNull
                    is Boolean -> JsonPrimitive(value)
                    is Number -> JsonPrimitive(value)
                    else -> JsonPrimitive(value.toString())
                }
            },
        )
    }
}
