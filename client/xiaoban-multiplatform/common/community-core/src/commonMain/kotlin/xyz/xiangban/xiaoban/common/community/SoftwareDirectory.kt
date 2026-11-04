package xyz.xiangban.xiaoban.common.community

import io.ktor.http.Url
import io.ktor.http.encodeURLParameter
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class SoftwareEntry(
    val project: String,
    val category: String,
    val platforms: List<String>,
    val releaseRepo: String,
    val version: String,
    val language: String = "",
    val name: String = "",
    val summary: String = "",
    val icon: String = "",
    val stars: Long? = null,
    val updatedAt: String? = null,
) {
    val releasesUrl: String get() = "https://github.com/$releaseRepo/releases"
    val displayName: String get() = name.ifBlank { project.substringAfter('/') }
}

@Serializable
data class SoftwareSummary(
    val stars: Long? = null,
    val version: String? = null,
    val updatedAt: String? = null,
    val starGain: Long? = null,
)

@Serializable
data class SoftwareSnapshot(
    val latest: SoftwareRelease? = null,
    val history: List<SoftwareRelease> = emptyList(),
)

@Serializable
data class SoftwareRelease(
    @SerialName("tag_name") val version: String,
    @SerialName("published_at") val publishedAt: String? = null,
    val prerelease: Boolean = false,
    val assets: List<SoftwareAsset> = emptyList(),
)

@Serializable
data class SoftwareAsset(
    val name: String,
    @SerialName("browser_download_url") val url: String,
    val size: Long? = null,
)

data class SoftwarePackage(val asset: SoftwareAsset, val platform: String, val architecture: String?)

private val packageExtension = Regex("""\.(exe|msi|msix|appx|dmg|pkg|appimage|deb|rpm|flatpak|apk|zip|7z|txz|tar\.(?:gz|bz2|xz|zst))$""")
private val auxiliary = Regex("""debug|dsym|pdbs|symbols|(?:^|[-_.])(?:src|source|sources|sha256|checksums?)(?:[-_.]|$)|export_templates|dont-use-this-file|(?:^|[-_.])(?:libhb|plugin)(?:[-_.]|$)""")

fun SoftwareRelease.packages(entry: SoftwareEntry): List<SoftwarePackage> = assets.mapNotNull { asset ->
    if (asset.name.any { it == '/' || it == '\\' || it.isISOControl() }) return@mapNotNull null
    val name = asset.name.lowercase()
    val format = packageExtension.find(name)?.groupValues?.get(1) ?: return@mapNotNull null
    if (auxiliary.containsMatchIn(name)) return@mapNotNull null
    val url = runCatching { Url(asset.url) }.getOrNull() ?: return@mapNotNull null
    val prefix = "/${entry.releaseRepo.lowercase()}/releases/download/"
    if (url.protocol.name != "https" || url.host != "github.com" || url.port != 443 ||
        url.user != null || url.password != null || !url.encodedPath.lowercase().startsWith(prefix)) return@mapNotNull null
    val platform = when {
        format == "apk" -> "Android"
        format in setOf("exe", "msi", "msix", "appx") || Regex("""(?:^|[-_.])win(?:dows)?(?:32|64)?(?:[-_.]|$)""").containsMatchIn(name) -> "Windows"
        format in setOf("dmg", "pkg") || Regex("""(?:^|[-_.])(?:mac(?:os)?|darwin)(?:[-_.]|$)""").containsMatchIn(name) -> "macOS"
        format in setOf("appimage", "deb", "rpm", "flatpak", "tar.zst") || Regex("""(?:^|[-_.])(?:linux|ubuntu)(?:[-_.]|$)""").containsMatchIn(name) -> "Linux"
        else -> return@mapNotNull null
    }
    val architecture = when {
        "universal" in name -> "通用"
        Regex("aarch64|arm64|macos-apple").containsMatchIn(name) -> "ARM64"
        Regex("armv7|arm32").containsMatchIn(name) -> "ARM 32 位"
        Regex("x86[-_]64|x64|amd64|win64|macos-intel").containsMatchIn(name) -> "x64"
        Regex("(?:^|[-_.])(?:x86(?:_32)?|i[3-6]86|win32)(?:[-_.]|$)").containsMatchIn(name) -> "x86"
        else -> null
    }
    SoftwarePackage(asset, platform, architecture)
}

fun SoftwareEntry.releaseUrl(version: String): String = "$releasesUrl/tag/${version.encodeURLParameter()}"

fun softwareDirectoryOrder(snapshots: Map<String, SoftwareSummary>, sort: String): Comparator<SoftwareEntry> {
    val byStars = compareByDescending<SoftwareEntry> { snapshots[it.project]?.stars ?: it.stars ?: -1L }
    val byUpdated = compareByDescending<SoftwareEntry> {
        parseIsoMillis(snapshots[it.project]?.updatedAt ?: it.updatedAt.orEmpty()) ?: 0L
    }
    return when (sort) {
        "hot" -> compareByDescending<SoftwareEntry> { snapshots[it.project]?.starGain ?: Long.MIN_VALUE }
            .then(byUpdated).then(byStars)
        "updated" -> byUpdated.then(byStars)
        else -> byStars.then(byUpdated)
    }
}
