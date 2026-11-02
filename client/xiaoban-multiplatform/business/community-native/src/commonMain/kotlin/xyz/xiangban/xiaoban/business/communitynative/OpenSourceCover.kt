package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.BiasAlignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.svg.SvgDecoder
import xyz.xiangban.xiaoban.common.community.OpenSourceProject

/** 与网页同源的封面编排；示意保持标注，详情页继续使用原始图片。 */
@Composable
internal fun OpenSourceProjectCover(env: CommunityEnv, project: OpenSourceProject) {
    val context = LocalPlatformContext.current
    val loader = remember(context) { ImageLoader.Builder(context).components { add(KtorNetworkFetcherFactory()); add(SvgDecoder.Factory()) }.build() }
    var imageFailed by remember(project.coverUrl) { mutableStateOf(false) }
    var iconFailed by remember(project.iconUrl) { mutableStateOf(false) }
    val accent = hexColor(project.coverAccent, LocalExtraColors.current.ink3)
    val line = LocalExtraColors.current.hairline
    val ink3 = LocalExtraColors.current.ink3
    val presentation = if (!imageFailed && project.coverUrl.isNotBlank() && project.coverStyle != "illustration") project.coverStyle.ifBlank { "screen" } else "illustration"
    val lines = project.coverLines.ifEmpty { project.tags }.take(3)

    BoxWithConstraints(Modifier.fillMaxWidth().aspectRatio(16f / 9f).clipToBounds().background(MaterialTheme.colorScheme.surface)) {
        val coverHeight = maxWidth * 9f / 16f
        Box(Modifier.fillMaxSize().background(accent.copy(alpha = .07f)))
        when (presentation) {
            "photo" -> AsyncImage(
                model = env.forum.imageUrl(project.coverUrl), imageLoader = loader, contentDescription = null,
                modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop, alignment = BiasAlignment((project.coverFocusX / 50f - 1f).coerceIn(-1f,1f),(project.coverFocusY / 50f - 1f).coerceIn(-1f,1f)), onError = { imageFailed = true },
            )
            "screen" -> AsyncImage(
                model = env.forum.imageUrl(project.coverUrl), imageLoader = loader, contentDescription = null,
                modifier = Modifier.align(Alignment.TopStart).offset(x = maxWidth * (if (project.coverVariant == "wide") 0f else if (project.coverVariant == "offset") .04f else .08f), y = coverHeight * (if (project.coverVariant == "wide") .25f else .28f))
                    .width(maxWidth * (if (project.coverVariant == "wide") 1f else .88f)).height(coverHeight * (if (project.coverVariant == "wide") .75f else .8f)).clip(RoundedCornerShape(topStart = 8.dp, topEnd = 8.dp)).border(1.dp, line, RoundedCornerShape(topStart = 8.dp, topEnd = 8.dp)),
                contentScale = ContentScale.Crop, alignment = Alignment.TopStart, onError = { imageFailed = true },
            )
            else -> {
                Text(when (project.coverLayout) { "terminal" -> "命令示例"; "notes" -> "内容示意"; else -> "流程示意" }, Modifier.align(Alignment.TopEnd).padding(16.dp), style = XType.Caption, color = ink3)
                Box(Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(coverHeight * .64f).padding(horizontal = 20.dp).padding(bottom = 20.dp)) {
                    when (project.coverLayout) {
                        "terminal" -> Column(
                            Modifier.fillMaxSize().clip(RoundedCornerShape(10.dp)).background(MaterialTheme.colorScheme.surface).border(1.dp, line, RoundedCornerShape(10.dp)).padding(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically),
                        ) {
                            Text(project.name.lowercase(), style = XType.Caption, color = ink3)
                            Text(lines.firstOrNull().orEmpty(), style = XType.Caption.copy(fontFamily = FontFamily.Monospace), maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text(lines.drop(1).joinToString(" · "), style = XType.Caption, color = ink3, maxLines = 1)
                        }
                        "notes" -> Column(Modifier.fillMaxSize().padding(horizontal = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically)) {
                            lines.forEachIndexed { index, text ->
                                Row(Modifier.fillMaxWidth().weight(1f).offset(x = if (project.coverVariant == "offset" && index == 1) (-12).dp else if (project.coverVariant == "offset" && index == 2) 8.dp else 0.dp)
                                    .clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).border(1.dp, line, RoundedCornerShape(8.dp)).padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                    Text("0${index + 1}", style = XType.Caption, color = ink3)
                                    Text(text, Modifier.weight(1f), style = XType.Secondary, maxLines = 1)
                                    Box(Modifier.width(24.dp).height(3.dp).background(accent.copy(alpha = .15f), RoundedCornerShape(2.dp)))
                                }
                            }
                        }
                        else -> Row(Modifier.fillMaxSize(), verticalAlignment = Alignment.CenterVertically) {
                            lines.forEachIndexed { index, text ->
                                val shift = if (project.coverVariant == "offset") when (index) { 0 -> (-8).dp; 2 -> 8.dp; else -> 0.dp } else 0.dp
                                Column(Modifier.weight(1f).offset(y = shift).height(coverHeight * .36f).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surface)
                                    .border(1.dp, accent.copy(alpha = .3f), RoundedCornerShape(12.dp)).padding(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically), horizontalAlignment = Alignment.CenterHorizontally) {
                                    Box(Modifier.size(12.dp).border(1.dp, accent.copy(alpha = .8f), RoundedCornerShape(if (index == 2) 99.dp else 4.dp)))
                                    Text(text, style = XType.Secondary, maxLines = 1)
                                }
                                if (index < lines.lastIndex) XIcon(XIcons.ArrowRight, Modifier.padding(horizontal = 4.dp), tint = accent.copy(alpha = .65f), size = 20.dp)
                            }
                        }
                    }
                }
            }
        }
        Row(
            Modifier.align(if (presentation == "photo") Alignment.BottomStart else Alignment.TopStart).padding(horizontal = 16.dp, vertical = 12.dp)
                .then(if (presentation == "photo") Modifier.clip(RoundedCornerShape(10.dp)).background(Color.White).padding(horizontal = 10.dp, vertical = 6.dp) else Modifier),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (project.iconUrl.isNotBlank() && !iconFailed) AsyncImage(
                model = env.forum.imageUrl(project.iconUrl), imageLoader = loader, contentDescription = null, onError = { iconFailed = true }, contentScale = if (project.iconFit == "mark-left") ContentScale.Crop else ContentScale.Fit, alignment = Alignment.CenterStart,
                modifier = (if (project.iconFit == "mark-left") Modifier.width(20.dp).height(28.dp) else Modifier.size(28.dp)).clip(RoundedCornerShape(8.dp)).background(Color.White).padding(if (project.iconFit == "mark-left") 0.dp else 3.dp),
            )
            Text(project.name, style = XType.Secondary, fontWeight = FontWeight.Medium, color = if (presentation == "photo") Color(0xFF222222) else MaterialTheme.colorScheme.onSurface, maxLines = 1)
        }
    }
}
