package xyz.xiangban.xiaoban.common.community

import io.ktor.http.Url

data class NavigationFilter(val key: String, val label: String)
data class NavigationCategory(val key: String, val label: String, val filters: List<NavigationFilter>)
internal data class NavigationLegacy(val name: String, val vendor: String, val url: String)
internal data class NavigationSnapshot(val version: Int, val categories: List<NavigationCategory>, val items: List<NavLink>, val legacy: Map<String, NavigationLegacy>)

val navigationCategories: List<NavigationCategory> get() = bundledNavigation.categories

fun navigationUrl(raw: String): String? = runCatching {
    val url = Url(raw)
    raw.takeIf { url.protocol.name in setOf("http", "https") && url.host.isNotBlank() && url.user.isNullOrEmpty() && url.password.isNullOrEmpty() }
}.getOrNull()

fun navigationDomain(raw: String): String = runCatching { Url(raw).host.removePrefix("www.") }.getOrDefault("")

/** v2 published content is authoritative, including removals. Older servers receive the reviewed taxonomy. */
fun NavGroups.navigationDirectory(): List<NavLink> {
    val defaults = bundledNavigation.items
    val upgrading = (catalogVersion ?: 0) < bundledNavigation.version
    val entries = groups.flatMap { group -> group.items.mapNotNull { item ->
        if (navigationUrl(item.url) == null) return@mapNotNull null
        val known = defaults.firstOrNull { it.key == item.key } ?: defaults.firstOrNull { it.url.trimEnd('/') == item.url.trimEnd('/') }
        val category = item.category.takeIf { key -> navigationCategories.any { it.key == key } }
            ?: known?.category ?: navigationCategories.firstOrNull { it.label == group.name }?.key ?: "models"
        if (!upgrading || item.category.isNotBlank() || known == null) item.copy(category = category)
        else {
            val old = bundledNavigation.legacy[item.key]
            val vendor = if (old != null && item.vendor == old.vendor) known.vendor else item.vendor
            item.copy(category = category,
                name = if (old != null && item.name == old.name) known.name else item.name,
                vendor = vendor,
                url = if (old != null && item.url == old.url) known.url else item.url,
                summary = item.summary.ifBlank { known.summary }, subtype = known.subtype, region = known.region,
                logo = item.logo.ifBlank { if (vendor == known.vendor) known.logo else "" },
            )
        }
    } }.toMutableList()
    if (upgrading) {
        val keys = entries.map { it.key }.toMutableSet()
        val identities = entries.map { "${it.category}:${it.url.trimEnd('/')}" }.toMutableSet()
        defaults.forEach { item ->
            if (catalogVersion == 1 && item.key in bundledNavigation.legacy && item.key !in keys) return@forEach
            val identity = "${item.category}:${item.url.trimEnd('/')}"
            if (item.key !in keys && identity !in identities) { entries.add(item); keys.add(item.key); identities.add(identity) }
        }
        val order = defaults.mapIndexed { index, item -> item.key to index }.toMap()
        entries.sortBy { order[it.key] ?: Int.MAX_VALUE }
    }
    return entries
}

fun NavLink.matchesNavigation(category: String, filter: String = ""): Boolean = this.category == category && when {
    filter.isEmpty() -> true
    category == "models" -> region == filter
    category == "api" && filter == "thirdparty" -> subtype in setOf("router", "inference")
    else -> subtype == filter
}

val NavLink.navigationType: String get() = when {
    category == "models" -> when (region) { "cn" -> "国产原厂"; "global" -> "海外原厂"; else -> "模型原厂" }
    subtype == "router" -> "聚合路由"
    subtype == "inference" -> "推理服务"
    else -> navigationCategories.firstOrNull { it.key == category }?.filters?.firstOrNull { it.key == subtype }?.label.orEmpty()
}
