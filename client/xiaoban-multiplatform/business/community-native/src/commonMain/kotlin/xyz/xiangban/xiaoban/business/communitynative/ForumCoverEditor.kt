package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.shape.CircleShape
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
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
    val extra = LocalExtraColors.current
    val canUse = valid && rendered?.first == spec && error == null
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).imePadding()) {
            ScreenBar("文字封面", onBack = onDismiss, trailing = {
                Box(
                    Modifier.padding(end = 10.dp).height(34.dp).clip(CircleShape)
                        .background(if (canUse) Brand else extra.fill)
                        .clickable(enabled = canUse) { rendered?.takeIf { it.first == spec }?.let { onUse(spec, it.second) } }
                        .padding(horizontal = 16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("使用封面", color = if (canUse) Color.White else extra.ink3, style = XType.Secondary, fontWeight = FontWeight.SemiBold)
                }
            })
            Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                Box(Modifier.fillMaxWidth().height(232.dp).clip(RoundedCornerShape(16.dp)).background(extra.fill), contentAlignment = Alignment.Center) {
                    rendered?.second?.let { Image(it.preview, "封面预览", Modifier.padding(12.dp).fillMaxSize(), contentScale = ContentScale.Fit) }
                }
                CoverTemplates(platform, template) { template = it }
                CoverRatioSegment(portrait) { portrait = it }
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Text("封面标题 · 可换行调整排版", Modifier.weight(1f), style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Text("${title.length} / 80", style = XType.Caption, color = if (title.length > 80) Danger else extra.ink3)
                    }
                    AreaBox(title, { title = it }, placeholder = "一句话，让人想点进来", minHeight = 76.dp, filled = true)
                }
                val hlMissing = highlight.isNotEmpty() && !title.contains(highlight)
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Text("重点词（选填，最多 16 字）", Modifier.weight(1f), style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Text(if (hlMissing) "须包含在标题内" else "留空自动强调末行", style = XType.Caption, color = if (hlMissing) Danger else extra.ink3)
                    }
                    AreaBox(highlight, { highlight = it }, placeholder = "例如：桌面新组件", minHeight = 46.dp, filled = true)
                }
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Text("副标题（选填）", Modifier.weight(1f), style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Text("${subtitle.length} / 100", style = XType.Caption, color = if (subtitle.length > 100) Danger else extra.ink3)
                    }
                    AreaBox(subtitle, { subtitle = it }, placeholder = "补充时间、版本或看点", minHeight = 56.dp, filled = true)
                }
                error?.let { Banner(it, warn = true) }
            }
        }
    }
}

@Composable
private fun CoverRatioSegment(portrait: Boolean, onSelect: (Boolean) -> Unit) {
    val extra = LocalExtraColors.current
    Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(extra.fill).padding(3.dp), horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        listOf(true to "竖版 3:4", false to "方形 1:1").forEach { (isPortrait, label) ->
            val on = portrait == isPortrait
            Box(
                Modifier.weight(1f).height(34.dp).clip(RoundedCornerShape(9.dp))
                    .background(if (on) MaterialTheme.colorScheme.surface else Color.Transparent)
                    .clickable { onSelect(isPortrait) },
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    label,
                    style = XType.Secondary,
                    fontWeight = if (on) FontWeight.SemiBold else FontWeight.Medium,
                    color = if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
                )
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
            Text(if (cover == null) "添加封面" else "更换封面", style = XType.Body)
            Text(if (cover == null) "相册 · 文字模板" else "将作为帖子首图展示", style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        if (cover != null) IconTap(XIcons.Close, "移除封面", { if (enabled) onRemove() })
        else XIcon(XIcons.ChevronRight, size = 16.dp, tint = LocalExtraColors.current.ink3)
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
                Text(template.label, style = XType.Caption, color = if (selected == template) Brand else LocalExtraColors.current.ink3)
            }
        }
    }
}
