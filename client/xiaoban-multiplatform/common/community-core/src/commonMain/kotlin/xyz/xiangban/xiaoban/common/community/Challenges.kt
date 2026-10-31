package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import io.ktor.http.encodeURLParameter
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.serializer

@Serializable
data class CommunityChallenge(
    val key: String,
    val title: String,
    val description: String,
    @SerialName("starts_at") val startsAt: String,
    @SerialName("ends_at") val endsAt: String,
    @SerialName("submission_tag") val submissionTag: String,
    @SerialName("topic_id") val topicId: Long? = null,
    val status: String,
    @SerialName("entry_count") val entryCount: Long = 0,
    @SerialName("created_at") val createdAt: String = "",
    @SerialName("updated_at") val updatedAt: String = "",
)
@Serializable
data class CommunityChallengeEntry(
    @SerialName("pack_id") val packId: Long,
    val pack: Pack,
    val featured: Boolean = false,
    @SerialName("vote_method") val voteMethod: String,
    @SerialName("vote_topic_id") val voteTopicId: Long? = null,
    val likes: Long? = null,
)
@Serializable
data class CommunityChallengeDetail(val challenge: CommunityChallenge, val entries: Page<CommunityChallengeEntry>)
@Serializable
data class ChallengeSubmitted(val status: String, @SerialName("pack_id") val packId: Long)

/** 状态完全由论坛时间判定，手机不按本地时钟推断可投稿窗口。 */
fun challengePhase(challenge: CommunityChallenge): String = when (challenge.status) {
    "upcoming" -> "尚未开始"
    "open" -> "征集中"
    "ended" -> "已截止"
    "closed" -> "已关闭"
    else -> "状态暂不可用"
}
fun challengeCanSubmit(challenge: CommunityChallenge): Boolean = challenge.status == "open"

/** 只筛“本人且上架”，公开阅读权限仍由提交接口作最终校验。 */
fun ownedChallengePacks(packs: List<Pack>, uid: String?): List<Pack> =
    if (uid == null) emptyList() else packs.filter { it.author?.uid == uid && it.status == "published" }

/** 赞来自作品真实讨论，缺少或不匹配的引用不当成另一个投票服务。 */
fun challengeVoteTopic(entry: CommunityChallengeEntry): Long? = entry.voteTopicId?.takeIf {
    it > 0 && entry.voteMethod == "topic_like" && entry.pack.id == entry.packId && entry.pack.topicId == it
}

class ChallengesApi(client: HttpClient, config: CommunityConfig, private val token: suspend () -> String?) {
    private val http = JsonHttp(client, config.forumBase)
    suspend fun list(page: Int = 1): Page<CommunityChallenge> = http.send(HttpMethod.Get, "/api/challenges?page=${page.coerceAtLeast(1)}", serializer())
    suspend fun detail(key: String, page: Int = 1): CommunityChallengeDetail = http.send(HttpMethod.Get,
        "/api/challenges/${key.encodeURLParameter()}?page=${page.coerceAtLeast(1)}", serializer())
    suspend fun submit(key: String, packId: Long): ChallengeSubmitted {
        if (key.isBlank() || packId <= 0) throw ApiException(400, "请选择有效挑战和作品")
        val access = token() ?: throw ApiException(401, "请登录后提交自己的作品")
        val result: ChallengeSubmitted = http.send(HttpMethod.Post, "/api/challenges/${key.encodeURLParameter()}/entries", serializer(),
            body = buildJsonObject { put("pack_id", packId) }.toString(), token = access)
        if (result.status != "submitted" || result.packId != packId) throw ApiException(200, "投稿结果无效，请刷新挑战确认")
        return result
    }
}

/** 复用已有帖子点赞，再读取服务器确认的计数；不在客户端推算票数。 */
suspend fun ForumApi.setChallengeEntryLiked(entry: CommunityChallengeEntry, liked: Boolean): Topic {
    val id = challengeVoteTopic(entry) ?: throw ApiException(400, "此作品没有可点赞的公开讨论")
    setTopicLiked(id, liked)
    return topic(id)
}
