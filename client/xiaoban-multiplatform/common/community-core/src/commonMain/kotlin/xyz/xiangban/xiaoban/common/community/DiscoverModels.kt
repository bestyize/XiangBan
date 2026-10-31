package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

// 手机端社区的发现页：与 server-forum 的 /api/discover/*、/api/tools、/api/help/stats 对应。
// 带 sample=true 的内容是后台标记的示例数据，界面要如实标出。

@Serializable
data class HomeBanner(val key: String, val tag: String = "", val title: String, val byline: String = "", val link: String = "")

@Serializable
data class NewsItem(
    val key: String,
    val title: String,
    val source: String = "",
    val url: String = "",
    @SerialName("cover_url") val coverUrl: String = "",
    /** 没有封面图时，缩略图和来源小圆点的颜色（#RRGGBB）。 */
    val accent: String = "",
    @SerialName("published_at") val publishedAt: String = "",
    val sample: Boolean = false,
)

@Serializable
data class NewsBlock(val date: String? = null, val total: Int = 0, val items: List<NewsItem> = emptyList())

@Serializable
data class HomeBundle(
    val banners: List<HomeBanner> = emptyList(),
    val news: NewsBlock = NewsBlock(),
    val featured: List<Pack> = emptyList(),
    val repos: RepoIssue? = null,
    val sample: Boolean = false,
)

// ───────────── 开源热榜 ─────────────

@Serializable
data class RepoRank(
    val rank: Int = 0,
    @SerialName("full_name") val fullName: String,
    val description: String = "",
    val language: String = "",
    @SerialName("language_color") val languageColor: String = "",
    @SerialName("star_total") val starTotal: Long = 0,
    @SerialName("star_gain") val starGain: Long = 0,
    val spark: List<Long> = emptyList(),
    /** 图标块里的小图标：term、code、doc、mic、bolt；认不出就显示仓库名首字。 */
    val icon: String = "",
    /** 图标块的底色（#RRGGBB）。 */
    val tint: String = "",
)

@Serializable
data class RepoIssue(
    val key: String = "",
    @SerialName("issue_no") val issueNo: Int = 0,
    @SerialName("week_start") val weekStart: String = "",
    @SerialName("week_end") val weekEnd: String = "",
    @SerialName("total_gain") val totalGain: Long = 0,
    @SerialName("wow_pct") val wowPct: Double = 0.0,
    @SerialName("daily_gain") val dailyGain: List<Long> = emptyList(),
    val ranks: List<RepoRank> = emptyList(),
    val sample: Boolean = false,
)

@Serializable
data class RepoIssueRef(val key: String, @SerialName("issue_no") val issueNo: Int, @SerialName("week_start") val weekStart: String = "", @SerialName("week_end") val weekEnd: String = "")

@Serializable
data class RepoIssueList(val items: List<RepoIssueRef> = emptyList())

// ───────────── 模型榜 ─────────────

@Serializable
data class ModelVendor(val key: String, val name: String, @SerialName("brand_color") val brandColor: String = "#888888", val tint: String = "#EEEEEE")

@Serializable
data class ModelEntry(
    val key: String,
    val name: String,
    val vendor: String,
    val iq: Double = 0.0,
    val code: Double = 0.0,
    val math: Double = 0.0,
    val agent: Double = 0.0,
    val price: Double = 0.0,
    val speed: Double = 0.0,
    val latency: Double = 0.0,
    val context: String = "",
    val open: Boolean = false,
    val reasoning: Boolean = false,
    val cn: Boolean = false,
    val tier: String = "C",
)

@Serializable
data class TierDef(val id: String, val label: String, val min: Double? = null)

@Serializable
data class ModelBoard(
    val vendors: List<ModelVendor> = emptyList(),
    val models: List<ModelEntry> = emptyList(),
    val tiers: List<TierDef> = emptyList(),
    val source: String = "",
    @SerialName("updated_at") val updatedAt: String? = null,
    val sample: Boolean = false,
) {
    fun vendor(key: String): ModelVendor? = vendors.firstOrNull { it.key == key }
}

// ───────────── 追踪 ─────────────

@Serializable
data class QuotaInfo(
    @SerialName("progress_pct") val progressPct: Int = 0,
    @SerialName("next_reset_at") val nextResetAt: String = "",
    @SerialName("published_at") val updatedAt: String = "",
    @SerialName("recent_intervals") val recentIntervals: List<Double> = emptyList(),
    val sample: Boolean = false,
)

@Serializable
data class LaunchItem(val key: String, val title: String, val tags: List<String> = emptyList(), @SerialName("published_at") val publishedAt: String = "", val sample: Boolean = false)

@Serializable
data class PromoItem(
    val key: String,
    val text: String,
    @SerialName("ends_at") val endsAt: String = "",
    val pct: Int = 0,
    @SerialName("days_left") val daysLeft: Int = 0,
    val sample: Boolean = false,
)

@Serializable
data class Tracker(
    val quota: QuotaInfo? = null,
    val launches: List<LaunchItem> = emptyList(),
    val promos: List<PromoItem> = emptyList(),
    val now: String = "",
    val sample: Boolean = false,
)

// ───────────── 搜索 ─────────────

@Serializable
data class HotWord(val key: String = "", val text: String, val tag: String = "")

@Serializable
data class HotWords(val items: List<HotWord> = emptyList())

// ───────────── 导航与资料 ─────────────

@Serializable
data class NavLink(
    val key: String,
    val group: String = "",
    val name: String,
    val vendor: String = "",
    val url: String,
    @SerialName("brand_color") val brandColor: String = "#888888",
    val tint: String = "#EEEEEE",
)

@Serializable
data class NavGroup(val name: String, val items: List<NavLink> = emptyList())

@Serializable
data class NavGroups(val groups: List<NavGroup> = emptyList())

@Serializable
data class DocLink(val key: String, val product: String = "", val title: String, val subtitle: String = "", val label: String = "guide", val url: String) {
    val labelText: String get() = if (label == "index") "索引" else "导读"
}

@Serializable
data class DocGroup(val name: String, val items: List<DocLink> = emptyList())

@Serializable
data class DocGroups(val groups: List<DocGroup> = emptyList())

// ───────────── AI 工具 ─────────────

@Serializable
data class ToolSummary(
    val key: String,
    val name: String,
    val developer: String = "",
    @SerialName("type_label") val typeLabel: String = "",
    val license: String = "unknown",
    val platforms: List<String> = emptyList(),
    @SerialName("icon_url") val iconUrl: String = "",
    @SerialName("latest_version") val latestVersion: String? = null,
    @SerialName("latest_date") val latestDate: String? = null,
    val size: Long? = null,
    val sample: Boolean = false,
) {
    val licenseText: String get() = when (license) { "open" -> "开源"; "closed" -> "闭源"; else -> "待确认" }
}

@Serializable
data class ToolList(val items: List<ToolSummary> = emptyList())

@Serializable
data class ReleaseItem(val kind: String = "new", val text: String)

@Serializable
data class ToolRelease(val version: String, val date: String = "", val items: List<ReleaseItem> = emptyList())

@Serializable
data class ToolArtifact(
    val os: String,
    val desc: String = "",
    val size: Long = 0,
    val sha256: String = "",
    @SerialName("mirror_url") val mirrorUrl: String = "",
    @SerialName("github_url") val githubUrl: String = "",
) {
    val osLabel: String get() = when (os) { "windows" -> "Windows"; "macos" -> "macOS"; else -> "Linux" }
}

@Serializable
data class ToolDetail(
    val key: String,
    val name: String,
    val developer: String = "",
    @SerialName("type_label") val typeLabel: String = "",
    val license: String = "unknown",
    val platforms: List<String> = emptyList(),
    val description: String = "",
    @SerialName("homepage_url") val homepageUrl: String = "",
    @SerialName("icon_url") val iconUrl: String = "",
    val releases: List<ToolRelease> = emptyList(),
    val artifacts: List<ToolArtifact> = emptyList(),
    @SerialName("latest_version") val latestVersion: String? = null,
    @SerialName("latest_date") val latestDate: String? = null,
    @SerialName("discussion_count") val discussionCount: Int = 0,
    val sample: Boolean = false,
) {
    val licenseText: String get() = when (license) { "open" -> "开源"; "closed" -> "闭源"; else -> "待确认" }

    /** 至少有一个安装包带小伴镜像地址时，才提供「小伴镜像」这个下载来源。 */
    val hasMirror: Boolean get() = artifacts.any { it.mirrorUrl.isNotEmpty() }
}

// ───────────── 求助与投票 ─────────────

@Serializable
data class HelpStats(val open: Int = 0, @SerialName("solved_this_week") val solvedThisWeek: Int = 0, @SerialName("bounty_points") val bountyPoints: Long = 0)

@Serializable
data class HelpInfo(
    val bounty: Int = 0,
    val status: String = "open",
    @SerialName("accepted_post_id") val acceptedPostId: Long? = null,
    @SerialName("expires_at") val expiresAt: String = "",
)

@Serializable
data class PollOption(val text: String, val votes: Long = 0)

@Serializable
data class Poll(
    val options: List<PollOption> = emptyList(),
    val total: Long = 0,
    @SerialName("closes_at") val closesAt: String? = null,
    @SerialName("my_vote") val myVote: Int? = null,
) {
    /** 某个选项占总票数的整数百分比；没人投票时都是 0。 */
    fun percent(index: Int): Int {
        if (total <= 0 || index !in options.indices) return 0
        return (options[index].votes * 100 / total).toInt()
    }
}

@Serializable
data class VoteResult(val poll: Poll)
