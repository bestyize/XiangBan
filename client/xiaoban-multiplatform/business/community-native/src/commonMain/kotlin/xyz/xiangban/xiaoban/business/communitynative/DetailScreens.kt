package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.Loader
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.WeeklyEntry
import xyz.xiangban.xiaoban.common.community.formatSize
import xyz.xiangban.xiaoban.common.community.htmlToPlain
import xyz.xiangban.xiaoban.common.community.kindLabel

@Composable
private fun <T> rememberLoader(env: CommunityEnv, key: Any, fetch: suspend () -> T): Loader<T> {
    val scope = rememberCoroutineScope()
    val loader = remember(env, key) { Loader(scope, fetch) }
    LaunchedEffect(loader) { loader.reload() }
    return loader
}

/** 复制链接与（平台支持时）系统分享。 */
@Composable
private fun ShareRow(env: CommunityEnv, title: String, url: String) {
    val platform = env.platform ?: return MetaText(url)
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        OutlinedButton({ platform.copy(url) }, shape = CardShape) { Text("复制链接") }
        OutlinedButton({ platform.share(title, url) }, shape = CardShape) { Text("分享") }
    }
}

@Composable
fun PackDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit) {
    val loader = rememberLoader(env, id) { env.forum.pack(id) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        PageHeader("内容详情", onBack = back)
        LoadBox(state, loader::reload) { pack -> PackBody(env, pack) }
    }
}

@Composable
private fun PackBody(env: CommunityEnv, pack: Pack) {
    val url = env.forum.packUrl(pack.id)
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface)) {
        Cover(pack.coverUrl, pack.id, Modifier.fillMaxWidth().padding(Gutter).aspectRatio(16f / 9f))
        Column(Modifier.padding(horizontal = Gutter), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(pack.title, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
            MetaText(listOfNotNull(kindLabel(pack.kind), pack.trustLabel, pack.author?.nickname, "v${pack.version}", pack.versions.firstOrNull { it.version == pack.version }?.size?.takeIf { it > 0 }?.let(::formatSize), "${pack.applyCount} 次应用").joinToString(" · "))
            if (pack.tags.isNotEmpty()) MetaText(pack.tags.joinToString("  ") { "#$it" })
            if (pack.summary.isNotBlank()) Text(pack.summary, fontSize = 15.sp)
            ShareRow(env, pack.title, url)
        }
        // 手机上不能应用内容包：安装要回到电脑上的小伴，这里说清楚而不是留一个点不了的按钮。
        Column(Modifier.padding(Gutter).fillMaxWidth().background(MaterialTheme.colorScheme.background, CardShape).padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text("在电脑上使用", fontWeight = FontWeight.Medium)
            Text("把链接发到电脑，用桌面版小伴打开即可一键应用。", color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp)
        }
        val changes = pack.versions.filter { it.changelog.isNotBlank() }
        if (changes.isNotEmpty()) {
            Column(Modifier.padding(horizontal = Gutter), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text("更新记录", fontWeight = FontWeight.Medium)
                changes.sortedByDescending { it.version }.forEach { v ->
                    Column { Text(v.label.ifBlank { "v${v.version}" }, fontSize = 13.sp, fontWeight = FontWeight.Medium); Text(v.changelog, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                }
            }
        }
        Column(Modifier.padding(Gutter)) {
            pack.topicId?.let { MetaText("讨论帖 #$it，可在“讨论”里查看") }
        }
    }
}

@Composable
fun WeeklyDetailScreen(env: CommunityEnv, week: String, open: (Route) -> Unit, back: () -> Unit) {
    val loader = rememberLoader(env, week) { env.forum.weekly(week) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        PageHeader("周刊", onBack = back)
        LoadBox(state, loader::reload) { detail ->
            Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface)) {
                Column(Modifier.padding(Gutter), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(detail.title, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                    MetaText(detail.week)
                    if (detail.summary.isNotBlank()) Text(detail.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    ShareRow(env, detail.title, env.forum.weeklyUrl(detail.week))
                }
                Hairline()
                detail.entries().forEach { entry -> WeeklyEntryRow(entry, open); Hairline(Modifier.padding(start = Gutter)) }
            }
        }
    }
}

@Composable
private fun WeeklyEntryRow(entry: WeeklyEntry, open: (Route) -> Unit) {
    when (entry) {
        is WeeklyEntry.PackEntry -> EntryColumn(entry.title, entry.summary, entry.note, kindLabel(entry.kind)) { open(Route.Pack(entry.packId)) }
        is WeeklyEntry.TopicEntry -> EntryColumn(entry.title, entry.summary, entry.note, "讨论") { open(Route.Topic(entry.topicId)) }
        is WeeklyEntry.TextEntry -> EntryColumn(entry.title.orEmpty(), "", entry.note, null, null)
    }
}

@Composable
private fun EntryColumn(title: String, summary: String, note: String, label: String?, onClick: (() -> Unit)? = {}) {
    val base = if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier
    Column(base.fillMaxWidth().padding(Gutter), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        if (label != null) MetaText(label)
        if (title.isNotBlank()) TitleText(title)
        if (summary.isNotBlank()) Text(summary, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp, maxLines = 2)
        if (note.isNotBlank()) Text(note, fontSize = 14.sp)
    }
}

@Composable
fun ProjectDetailScreen(env: CommunityEnv, slug: String, back: () -> Unit) {
    val loader = rememberLoader(env, slug) { env.forum.project(slug) }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        PageHeader("项目", onBack = back)
        LoadBox(state, loader::reload) { project ->
            Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface).padding(Gutter), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(project.name, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                if (project.summary.isNotBlank()) Text(project.summary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                project.links.forEach { link ->
                    OutlinedButton({ env.platform?.openUrl(link.url) }, enabled = env.platform != null, shape = CardShape, modifier = Modifier.fillMaxWidth()) { Text(link.title) }
                }
                val body = htmlToPlain(project.bodyHtml)
                if (body.isNotBlank()) Text(body, fontSize = 15.sp)
                ShareRow(env, project.name, env.forum.projectUrl(project.slug))
            }
        }
    }
}
