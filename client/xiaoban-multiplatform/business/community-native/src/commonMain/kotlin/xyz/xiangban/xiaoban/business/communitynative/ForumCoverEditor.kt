package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.border
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import xyz.xiangban.xiaoban.common.community.ForumUploadFile

/** The preview is the very bitmap encoded in file; no network or model is involved. */
data class ForumCoverImage(val file: ForumUploadFile, val preview: ImageBitmap)
enum class ForumCoverTemplate(val label: String) {
    Marker("重点大字"), Memo("备忘便签"), Notebook("线圈手账"), Pop("醒目撞色"), Collage("拼贴纸条"), Editorial("杂志标题")
}
data class ForumCoverSpec(val title: String, val subtitle: String = "", val template: ForumCoverTemplate = ForumCoverTemplate.Marker, val portrait: Boolean = true, val highlight: String = "")

@Composable
internal fun ForumCoverEditor(platform: PlatformActions, initial: ForumCoverSpec, onDismiss: () -> Unit, onUse: (ForumCoverSpec, ForumCoverImage) -> Unit) {
    var title by remember { mutableStateOf(initial.title) }
    var subtitle by remember { mutableStateOf(initial.subtitle) }
    var template by remember { mutableStateOf(initial.template) }
    var portrait by remember { mutableStateOf(initial.portrait) }
    var highlight by remember { mutableStateOf(initial.highlight) }
    val spec = ForumCoverSpec(title.trim(), subtitle.trim(), template, portrait, highlight.trim())
    var rendered by remember { mutableStateOf<Pair<ForumCoverSpec, ForumCoverImage>?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    val valid = spec.title.isNotBlank() && title.length <= 80 && subtitle.length <= 100 && highlight.length <= 16 && (spec.highlight.isEmpty() || spec.title.contains(spec.highlight))
    LaunchedEffect(spec) {
        error = null
        if (spec.title.length > 80 || spec.subtitle.length > 100 || spec.highlight.length > 16) return@LaunchedEffect
        if (spec.highlight.isNotEmpty() && !spec.title.contains(spec.highlight)) { error = "重点词需要是标题中的文字"; return@LaunchedEffect }
        delay(180)
        try {
            val result = platform.renderForumCover(spec.copy(title = spec.title.ifBlank { "把你的想法\n变成作品" }))
            if (result != null) rendered = spec to result else error = "当前设备暂不支持文字封面"
        } catch (e: CancellationException) { throw e }
        catch (e: Exception) { error = e.message ?: "封面生成失败，请调整文字后重试" }
    }
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).imePadding()) {
            ScreenBar("文字封面", onBack = onDismiss, trailing = {
                PillButton("使用封面", { rendered?.takeIf { it.first == spec }?.let { onUse(spec, it.second) } },
                    Modifier.padding(end = 12.dp), enabled = valid && rendered?.first == spec && error == null, height = 36.dp)
            })
            Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Box(Modifier.fillMaxWidth().height(240.dp).clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
                    rendered?.second?.let { Image(it.preview, "封面预览", Modifier.padding(12.dp).fillMaxSize(), contentScale = ContentScale.Fit) }
                }
                CoverTemplates(platform, template) { template = it }
                CoverChoices(listOf(true to "竖版 3:4", false to "方形 1:1"), portrait) { portrait = it }
                Text("封面标题 · 可换行调整排版", fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                AreaBox(title, { title = it }, placeholder = "一句话，让人想点进来", minHeight = 80.dp, filled = true)
                Text("${title.length} / 80", fontSize = 11.sp, color = if (title.length > 80) Danger else LocalExtraColors.current.ink3)
                AreaBox(highlight, { highlight = it }, placeholder = "重点词（选填，最多 16 字）", minHeight = 48.dp, filled = true)
                Text("留空时自动强调最后一行", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                AreaBox(subtitle, { subtitle = it }, placeholder = "副标题（选填）", minHeight = 64.dp, filled = true)
                Text("${subtitle.length} / 100", fontSize = 11.sp, color = if (subtitle.length > 100) Danger else LocalExtraColors.current.ink3)
                error?.let { Banner(it, warn = true) }
            }
        }
    }
}

@Composable
internal fun ForumCoverEntry(cover: ForumCoverImage?, enabled: Boolean, onChoose: () -> Unit, onRemove: () -> Unit) {
    Row(Modifier.fillMaxWidth().padding(bottom = 12.dp).clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill)
        .clickable(enabled = enabled, onClick = onChoose).padding(12.dp), verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        if (cover != null) Image(cover.preview, "帖子封面", Modifier.size(60.dp, 80.dp).clip(RoundedCornerShape(12.dp)), contentScale = ContentScale.Fit)
        else XIcon(XIcons.Image, size = 24.dp, tint = LocalExtraColors.current.ink3)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(if (cover == null) "添加封面" else "更换封面", fontSize = 14.sp)
            Text(if (cover == null) "相册 · 文字模板" else "将作为帖子首图展示", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        }
        if (cover != null) IconTap(XIcons.Close, "移除封面", { if (enabled) onRemove() })
        else XIcon(XIcons.ChevronRight, size = 16.dp, tint = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun <T> CoverChoices(items: List<Pair<T, String>>, selected: T, onSelect: (T) -> Unit) {
    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items.forEach { (id, name) -> PageChip(name, id == selected) { onSelect(id) } }
    }
}

@Composable
private fun CoverTemplates(platform: PlatformActions, selected: ForumCoverTemplate, onSelect: (ForumCoverTemplate) -> Unit) {
    val previews = remember(platform) { mutableStateMapOf<ForumCoverTemplate, ImageBitmap>() }
    LaunchedEffect(platform) {
        ForumCoverTemplate.entries.forEach { template ->
            try {
                platform.renderForumCover(ForumCoverSpec("把你的想法\n变成作品", template = template), thumbnail = true)?.let { previews[template] = it.preview }
            } catch (e: CancellationException) { throw e }
            catch (_: Exception) { /* The named choice remains usable if its thumbnail fails. */ }
        }
    }
    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        ForumCoverTemplate.entries.forEach { template ->
            Column(Modifier.width(76.dp).clickable { onSelect(template) }, horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Box(Modifier.fillMaxWidth().aspectRatio(0.75f).clip(RoundedCornerShape(8.dp))
                    .background(LocalExtraColors.current.fill).border(if (selected == template) 2.dp else 1.dp,
                        if (selected == template) Brand else LocalExtraColors.current.line, RoundedCornerShape(8.dp)), contentAlignment = Alignment.Center) {
                    previews[template]?.let { Image(it, null, Modifier.fillMaxSize().padding(3.dp), contentScale = ContentScale.Fit) }
                }
                Text(template.label, fontSize = 11.sp, color = if (selected == template) Brand else LocalExtraColors.current.ink3)
            }
        }
    }
}
