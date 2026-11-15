package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
internal data class FeaturedIdeaCatalog(val items: List<FeaturedIdea>)

@Serializable
data class FeaturedIdea(
    val id: String,
    val kind: String,
    val title: String,
    val description: String,
    val cover: String,
    val label: String? = null,
    val author: String,
    val date: String,
    val href: String,
    @SerialName("pack_slug") val packSlug: String? = null,
    val player: String? = null,
    val introduction: String? = null,
    val controls: FeaturedIdeaControls? = null,
    val site: String? = null,
    val repository: String? = null,
    val images: List<String> = emptyList(),
)

@Serializable
data class FeaturedIdeaControls(val mobile: String? = null, val desktop: String? = null)

private val ideaPlayerId = Regex("[a-z0-9][a-z0-9-]{1,39}")

fun isIdeaPlayerId(id: String): Boolean = ideaPlayerId.matches(id)

@Serializable
internal data class IdeaRuntimeDescriptor(val kind: String, val version: String, val sha256: String, val size: Long, val url: String)

data class IdeaRuntimeDownload(
    val id: String,
    val kind: String,
    val version: String,
    val sha256: String,
    val size: Long,
    val url: String,
)

sealed interface IdeaFeedEntry {
    val key: String
    val date: String

    data class Featured(val item: FeaturedIdea) : IdeaFeedEntry {
        override val key get() = "featured:${item.id}"
        override val date get() = item.date
    }

    data class Work(val pack: Pack) : IdeaFeedEntry {
        override val key get() = "pack:${pack.id}"
        override val date get() = pack.updatedAt
    }

    data class Plan(val scheme: Scheme) : IdeaFeedEntry {
        override val key get() = "scheme:${scheme.id}"
        override val date get() = scheme.createdAt
    }

    data class Post(val topic: Topic) : IdeaFeedEntry {
        override val key get() = "topic:${topic.id}"
        override val date get() = topic.createdAt
    }
}

data class IdeaFeedPage(val entries: List<IdeaFeedEntry>, val hasMore: Boolean)

/** 与网页一样，精选按目录顺序在前，同页社区内容按时间混排。 */
fun mergeIdeaFeedPage(
    featured: List<FeaturedIdea>,
    packs: Page<Pack>?,
    schemes: Page<Scheme>?,
    topics: Page<Topic>?,
): IdeaFeedPage {
    val packTopics = packs?.items.orEmpty().mapNotNull(Pack::topicId).toSet()
    val featuredPacks = featured.mapNotNull(FeaturedIdea::packSlug).toSet()
    val community = buildList<IdeaFeedEntry> {
        topics?.items.orEmpty().filterNot { it.id in packTopics }.forEach { add(IdeaFeedEntry.Post(it)) }
        packs?.items.orEmpty().filterNot { it.slug in featuredPacks }.forEach { add(IdeaFeedEntry.Work(it)) }
        schemes?.items.orEmpty().forEach { add(IdeaFeedEntry.Plan(it)) }
    }.sortedByDescending { parseIsoMillis(it.date) ?: 0L }
    return IdeaFeedPage(
        entries = featured.map { IdeaFeedEntry.Featured(it) } + community,
        hasMore = listOfNotNull(packs, schemes, topics).any { it.hasMore },
    )
}
