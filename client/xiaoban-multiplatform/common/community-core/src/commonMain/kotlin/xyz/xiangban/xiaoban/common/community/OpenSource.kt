package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class OpenSourceCategory(val key: String, val label: String, val description: String = "")

@Serializable
data class OpenSourceProject(
    @SerialName("full_name") val fullName: String,
    val name: String,
    val title: String,
    val summary: String,
    val category: String,
    @SerialName("related_categories") val relatedCategories: List<String> = emptyList(),
    val tags: List<String> = emptyList(),
    val effort: String = "",
    @SerialName("cover_url") val coverUrl: String = "",
    @SerialName("cover_caption") val coverCaption: String = "",
    @SerialName("icon_url") val iconUrl: String = "",
    @SerialName("icon_source_url") val iconSourceUrl: String = "",
    @SerialName("cover_style") val coverStyle: String = "",
    @SerialName("cover_layout") val coverLayout: String = "flow",
    @SerialName("cover_accent") val coverAccent: String = "",
    @SerialName("cover_lines") val coverLines: List<String> = emptyList(),
    @SerialName("cover_template") val coverTemplate: String = "",
    @SerialName("cover_variant") val coverVariant: String = "balanced",
    @SerialName("cover_focus_x") val coverFocusX: Float = 50f,
    @SerialName("cover_focus_y") val coverFocusY: Float = 46f,
    @SerialName("icon_fit") val iconFit: String = "contain",
    @SerialName("homepage_url") val homepageUrl: String = "",
    val highlights: List<String> = emptyList(),
    val audience: String = "",
    @SerialName("getting_started") val gettingStarted: String = "",
    val notes: List<String> = emptyList(),
    @SerialName("source_url") val sourceUrl: String,
    val featured: Boolean = false,
    val order: Int = 0,
) {
    val githubUrl: String get() = "https://github.com/$fullName"
}

@Serializable
data class OpenSourceCollection(
    val key: String,
    val title: String,
    val summary: String,
    val description: String = "",
    val projects: List<String> = emptyList(),
    val notes: List<String> = emptyList(),
)

@Serializable
data class OpenSourceCatalog(
    val version: Int = 1,
    @SerialName("reviewed_at") val reviewedAt: String = "",
    val notice: String = "",
    val categories: List<OpenSourceCategory> = emptyList(),
    val projects: List<OpenSourceProject> = emptyList(),
    val collections: List<OpenSourceCollection> = emptyList(),
    val related: List<RelatedOpenSourceProject> = emptyList(),
) {
    fun project(name: String) = projects.firstOrNull { it.fullName.equals(name, ignoreCase = true) }
    fun categoryLabel(key: String) = categories.firstOrNull { it.key == key }?.label ?: key
    fun filter(category: String, query: String = "", tag: String = ""): List<OpenSourceProject> {
        val q = query.trim()
        return projects.filter { project ->
            (category == "all" || (if (category == "recommend") project.featured else project.category == category || category in project.relatedCategories)) &&
                (tag.isEmpty() || tag in project.tags) &&
                (q.isEmpty() || listOf(project.name, project.fullName, project.title, project.summary).plus(project.tags).any { it.contains(q, ignoreCase = true) })
        }.sortedBy { it.order }
    }
}

@Serializable
data class RelatedOpenSourceProject(
    @SerialName("full_name") val fullName: String,
    val name: String,
    val summary: String = "",
)

@Serializable
internal data class ProjectReadmeIndex(val projects: Map<String, String>)

@Serializable
data class ProjectReadme(
    val schema: Int,
    @SerialName("full_name") val fullName: String,
    val card: ProjectReadmeCard,
    @SerialName("source_url") val sourceUrl: String,
    val commit: String,
    @SerialName("fetched_at") val fetchedAt: String,
    val translation: String = "",
    val zh: String,
    val en: String,
    val licenses: List<ProjectReadmeLicense> = emptyList(),
)

@Serializable
data class ProjectReadmeCard(
    val name: String,
    @SerialName("summary_zh") val summaryZh: String = "",
    @SerialName("icon_source_url") val iconSourceUrl: String = "",
    @SerialName("icon_attribution") val iconAttribution: String = "",
    @SerialName("github_url") val githubUrl: String = "",
    @SerialName("primary_url") val primaryUrl: String = "",
    @SerialName("primary_label") val primaryLabel: String = "",
    val stars: Long? = null,
    @SerialName("code_updated_at") val codeUpdatedAt: String = "",
    val license: String = "",
    val platforms: List<String> = emptyList(),
    @SerialName("github_archived") val githubArchived: Boolean = false,
    @SerialName("upstream_maintenance_notice") val upstreamMaintenanceNotice: String = "",
)

@Serializable
data class ProjectReadmeLicense(val name: String, val text: String)
