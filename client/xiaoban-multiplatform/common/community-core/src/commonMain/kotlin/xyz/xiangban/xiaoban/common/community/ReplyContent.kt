package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObjectBuilder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put

@Serializable data class WorkReference(val kind: String, val id: Long, val version: Int? = null)
@Serializable data class ResolvedWorkReference(val kind: String, val id: Long, val version: Int? = null, val available: Boolean, val content: CommunityContentCard? = null)
@Serializable data class ReplySource(val body: String, @SerialName("work_refs") val workRefs: List<WorkReference> = emptyList(), val attachments: List<ForumAttachment> = emptyList())

fun validateWorkReferences(refs: List<WorkReference>) {
    if (refs.size > 3 || refs.map { it.kind to it.id }.distinct().size != refs.size) throw ApiException(400, "最多选择 3 个不同的作品或方案")
    refs.forEach {
        requireContentTarget(it.kind, it.id)
        if ((it.version != null && it.version <= 0) || (it.kind == "scheme" && it.version != null)) throw ApiException(400, "方案不支持版本，作品版本必须大于零")
    }
}

/** Hide even malformed stale metadata whenever availability is false or identity doesn't match. */
fun safeReferencedContent(ref: ResolvedWorkReference): CommunityContentCard? = ref.content?.takeIf {
    ref.available && ref.kind in listOf("pack", "scheme") && ref.id > 0 && it.id == ref.id &&
        (ref.kind != "scheme" || ref.version == null) && (ref.version == null || (ref.version > 0 && it.version == ref.version))
}

/** Null omits fields for old-client edits; an explicit empty list clears only those associations. */
internal fun JsonObjectBuilder.replyAssociations(workRefs: List<WorkReference>?, attachments: List<ForumAttachment>?) {
    workRefs?.let { refs ->
        validateWorkReferences(refs)
        put("work_refs", buildJsonArray { refs.forEach { ref -> add(buildJsonObject {
            put("kind", ref.kind); put("id", ref.id); ref.version?.let { put("version", it) }
        }) } })
    }
    attachments?.let { files ->
        validateTopicAttachments(files)
        put("attachment_ids", buildJsonArray { files.forEach { add(JsonPrimitive(it.id)) } })
        put("attachment_prices", buildJsonArray { files.forEach { add(buildJsonObject { put("id", it.id); put("price", it.price) }) } })
    }
}

/** The server snapshots an omitted pack version at publication. No arbitrary JSON is accepted. */
data class SelectedWorkReference(val reference: WorkReference, val card: CommunityContentCard?)
class ReplyReferencesApi(client: io.ktor.client.HttpClient, private val config: CommunityConfig) {
    private val sharing = CommunitySharingApi(client, config)
    suspend fun fromUrl(raw: String): SelectedWorkReference {
        val target = communityShareTarget(raw.trim(), config)?.takeIf { it.kind in listOf("pack", "scheme") }
            ?: throw ApiException(400, "请输入本站公开作品或方案的链接")
        val info = sharing.publicInfo(target)
        return SelectedWorkReference(WorkReference(target.kind, target.id.toLong()),
            CommunityContentCard(target.id.toLong(), info.title, info.summary, kind = info.packKind))
    }
}

/** Editable controls need both server ownership and current topic state; unknown context stays closed. */
fun replyCanEdit(post: Post, accountId: String?, topic: Topic?): Boolean =
    accountId != null && post.mine && post.author.uid == accountId && post.status == "normal" && !post.blocked &&
        topic?.locked == false && topic.help?.acceptedPostId != post.id

/** Preserve source versions even when a referenced work was removed or its public card changed. */
fun replyEditSelection(source: ReplySource, publicRefs: List<ResolvedWorkReference>): List<SelectedWorkReference> =
    source.workRefs.map { ref -> SelectedWorkReference(ref, publicRefs.firstOrNull {
        it.kind == ref.kind && it.id == ref.id && it.version == ref.version
    }?.let(::safeReferencedContent)) }
