package xyz.xiangban.xiaoban.common.link

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper
import kotlinx.serialization.KSerializer
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json

/** 按账号、电脑和会话分开保存，数据库位于应用私有目录。 */
class SqliteRemoteCache(context: Context) : SQLiteOpenHelper(context.applicationContext, "remote_chat_cache.db", null, 1), RemoteCache {
    private val json = Json { ignoreUnknownKeys = true; encodeDefaults = true }

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL("CREATE TABLE cache (account TEXT NOT NULL, peer TEXT NOT NULL, session TEXT NOT NULL, kind TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY (account, peer, session, kind))")
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) = Unit

    private fun read(account: String, peer: String, session: String, kind: String): String? =
        readableDatabase.query("cache", arrayOf("data"), "account=? AND peer=? AND session=? AND kind=?", arrayOf(account, peer, session, kind), null, null, null).use {
            if (it.moveToFirst()) it.getString(0) else null
        }

    private fun write(account: String, peer: String, session: String, kind: String, data: String) {
        val values = ContentValues().apply {
            put("account", account); put("peer", peer); put("session", session); put("kind", kind); put("data", data)
        }
        writableDatabase.insertWithOnConflict("cache", null, values, SQLiteDatabase.CONFLICT_REPLACE)
    }

    private fun <T> decode(value: String?, serializer: KSerializer<T>): List<T>? =
        value?.let { runCatching { json.decodeFromString(ListSerializer(serializer), it) }.getOrNull() }

    override fun peers(account: String): List<PeerSummary> = decode(read(account, "", "", "peers"), PeerSummary.serializer()).orEmpty()
    override fun savePeers(account: String, peers: List<PeerSummary>) = write(account, "", "", "peers", json.encodeToString(ListSerializer(PeerSummary.serializer()), peers.map { it.copy(online = false) }))
    override fun sessions(account: String, peer: String): List<RemoteSessionSummary> = decode(read(account, peer, "", "sessions"), RemoteSessionSummary.serializer()).orEmpty()
    override fun saveSessions(account: String, peer: String, sessions: List<RemoteSessionSummary>) = write(account, peer, "", "sessions", json.encodeToString(ListSerializer(RemoteSessionSummary.serializer()), sessions))
    override fun messages(account: String, peer: String, session: String): List<RemoteTurn>? = decode(read(account, peer, session, "messages"), RemoteTurn.serializer())
    override fun saveMessages(account: String, peer: String, session: String, messages: List<RemoteTurn>) = write(account, peer, session, "messages", json.encodeToString(ListSerializer(RemoteTurn.serializer()), messages))
    override fun removePeer(account: String, peer: String) {
        writableDatabase.delete("cache", "account=? AND peer=?", arrayOf(account, peer))
        savePeers(account, peers(account).filterNot { it.id == peer })
    }
    override fun clearAccount(account: String) { writableDatabase.delete("cache", "account=?", arrayOf(account)) }
}
