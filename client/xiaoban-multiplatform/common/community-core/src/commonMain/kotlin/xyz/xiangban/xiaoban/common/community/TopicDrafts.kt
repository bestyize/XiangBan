package xyz.xiangban.xiaoban.common.community

import kotlin.io.encoding.Base64
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable

@Serializable
data class TopicDraft(
    val title: String = "",
    val body: String = "",
    val section: Int = 0,
    val boardKey: String = "",
    val tags: List<String> = emptyList(),
    val tagInput: String = "",
    val attachments: List<ForumAttachment> = emptyList(),
    val pollOptions: List<String>? = null,
    val pollDays: Int = 7,
    val cover: TopicDraftCover? = null,
    val linkedWork: TopicDraftLink? = null,
    val uploadedWork: TopicDraftUpload? = null,
    val requestId: String? = null,
)

@Serializable
data class TopicDraftCover(
    val mediaId: String,
    val name: String,
    val mime: String,
    val template: TopicDraftCoverText? = null,
    val uploaded: ForumImage? = null,
)

@Serializable
data class TopicDraftLink(val id: Long, val title: String, val kind: String)

@Serializable
data class TopicDraftCoverText(
    val title: String, val subtitle: String, val template: String, val portrait: Boolean, val highlight: String,
)

@Serializable
data class TopicDraftUpload(
    val key: String, val kind: String, val title: String, val summary: String, val originalName: String,
    val size: Long, val rightsAccepted: Boolean,
)

/** Keeps each account's compose data in app-private files, with cover bytes separate from text edits. */
class TopicDrafts(private val files: ChatHistoryFiles) {
    private val lock = Mutex()

    suspend fun load(owner: String, slot: String): Pair<TopicDraft, ForumUploadFile?>? = onDisk {
        val base = base(owner, slot)
        val draft = read(base) ?: return@onDisk null
        val cover = draft.cover?.let {
            runCatching { files.readMedia(coverPath(base, it)) }.getOrNull()
                ?.let { bytes -> ForumUploadFile(it.name, it.mime, bytes) }
        }
        draft to cover
    }

    suspend fun save(owner: String, slot: String, draft: TopicDraft, cover: ForumUploadFile? = null) = onDisk {
        val base = base(owner, slot)
        val previous = read(base)?.cover
        val next = draft.cover
        if (next != null && cover != null) {
            val path = coverPath(base, next)
            if (!files.hasMedia(path)) files.writeMedia(path, cover.bytes)
        }
        files.write("$base.json", CommunityJson.encodeToString(draft))
        if (previous != null && previous.mediaId != next?.mediaId) runCatching { files.delete(coverPath(base, previous)) }
        Unit
    }

    suspend fun clear(owner: String, slot: String) = onDisk {
        val base = base(owner, slot)
        val cover = read(base)?.cover
        files.delete("$base.json")
        if (cover != null) runCatching { files.delete(coverPath(base, cover)) }
        Unit
    }

    private suspend fun <T> onDisk(action: () -> T): T = withContext(Dispatchers.Default) { lock.withLock { action() } }
    private fun read(base: String): TopicDraft? =
        files.read("$base.json")?.let { text -> runCatching { CommunityJson.decodeFromString<TopicDraft>(text) }.getOrNull() }
    private fun key(value: String): String = Base64.UrlSafe.encode(value.encodeToByteArray()).trimEnd('=')
    private fun base(owner: String, slot: String) = "topic-drafts/v1/${key(owner)}/${key(slot)}"
    private fun coverPath(base: String, cover: TopicDraftCover) = "$base-${key(cover.mediaId)}.cover"
}
