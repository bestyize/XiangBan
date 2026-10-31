package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.ModelVendor

// 发现页（首页五个 tab、资源、讨论）共用的小部件。

private val CardShape = RoundedCornerShape(8.dp)

/** 灰色页面上的白底卡片。 */
@Composable
fun WhiteCard(modifier: Modifier = Modifier, padding: Dp = 16.dp, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier.fillMaxWidth().clip(CardShape).background(MaterialTheme.colorScheme.surface).padding(padding), content = content)
}

/** 卡片标题：左边标题加一行小字，右边一个「更多 ›」之类的入口。 */
@Composable
fun SectionHeader(title: String, modifier: Modifier = Modifier, note: String? = null, action: String? = null, onAction: (() -> Unit)? = null, icon: androidx.compose.ui.graphics.vector.ImageVector? = null) {
    Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (icon != null) XIcon(icon, tint = MaterialTheme.colorScheme.onSurface, size = 18.dp)
        Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
        if (note != null) Text(note, Modifier.weight(1f), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        else Box(Modifier.weight(1f))
        if (action != null && onAction != null) {
            Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable(onClick = onAction).padding(horizontal = 4.dp, vertical = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(action, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 14.dp)
            }
        }
    }
}

/** 页脚的灰色说明，如实标出示例数据和数据出处。 */
@Composable
fun FootNote(text: String, modifier: Modifier = Modifier) {
    Row(modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        XIcon(XIcons.Info, tint = LocalExtraColors.current.ink3, size = 14.dp, modifier = Modifier.padding(top = 2.dp))
        Text(text, fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
    }
}

/** 小标签：状态（进行中、已采纳）、许可证、类型。 */
@Composable
fun Tag(text: String, fg: Color, bg: Color, modifier: Modifier = Modifier) {
    Text(text, modifier.clip(RoundedCornerShape(6.dp)).background(bg).padding(horizontal = 8.dp, vertical = 2.dp), fontSize = 12.sp, color = fg, maxLines = 1)
}

/** 蓝色标签（进行中、修复）：稿子里是 #3E5C8A 字、#E8EEF8 底。 */
@Composable
fun BlueTag(text: String, modifier: Modifier = Modifier) =
    Tag(text, if (ThemeState.dark) Color(0xFF9FB6DC) else Color(0xFF3E5C8A), if (ThemeState.dark) Color(0xFF9FB6DC).copy(alpha = 0.16f) else Color(0xFFE8EEF8), modifier)

/** 绿色标签（已采纳、开源、新增）：#2E6B47 字、#E6F4EA 底。 */
@Composable
fun GreenTag(text: String, modifier: Modifier = Modifier) =
    Tag(text, if (ThemeState.dark) Color(0xFF7BD39A) else Color(0xFF2E6B47), if (ThemeState.dark) Color(0xFF7BD39A).copy(alpha = 0.16f) else Color(0xFFE6F4EA), modifier)

/** 灰色标签（已到期、闭源、类型）：#666 字、#F5F5F5 底。 */
@Composable
fun GrayTag(text: String, modifier: Modifier = Modifier) = Tag(text, MaterialTheme.colorScheme.onSurfaceVariant, LocalExtraColors.current.fill, modifier)

/** 琥珀色标签（优化、待确认）：#8A5B00 字、#FBF0DA 底。 */
@Composable
fun AmberTag(text: String, modifier: Modifier = Modifier) =
    Tag(text, if (ThemeState.dark) Color(0xFFE0A03A) else Color(0xFF8A5B00), if (ThemeState.dark) Color(0xFFE0A03A).copy(alpha = 0.16f) else Color(0xFFFBF0DA), modifier)

/** 图标块：没有图标图片时，用名字的首字放在品牌色的浅底上。 */
@Composable
fun LetterIcon(name: String, brand: Color, tint: Color, modifier: Modifier = Modifier, size: Dp = 40.dp) {
    Box(modifier.size(size).clip(RoundedCornerShape(size / 4)).background(tint), contentAlignment = Alignment.Center) {
        Text(name.trim().take(1).uppercase(), color = brand, fontSize = (size.value * 0.42f).sp, fontWeight = FontWeight.Bold)
    }
}

/** 厂商的图标块。[vendor] 找不到时用灰色。 */
@Composable
fun VendorIcon(name: String, vendor: ModelVendor?, modifier: Modifier = Modifier, size: Dp = 32.dp) {
    val brand = hexColor(vendor?.brandColor.orEmpty())
    LetterIcon(vendor?.name ?: name, brand, tintColor(brand, vendor?.tint.orEmpty()), modifier, size)
}

/** 站内落点（`topic:12`、`pack:3`、`tool:cc-switch`、`weekly:2026-W44`）或外链；其余忽略。 */
fun openLink(link: String, env: CommunityEnv, open: (Route) -> Unit) {
    when {
        link.startsWith("topic:") -> link.removePrefix("topic:").toLongOrNull()?.let { open(Route.Topic(it)) }
        link.startsWith("pack:") -> link.removePrefix("pack:").toLongOrNull()?.let { open(Route.Pack(it)) }
        link.startsWith("tool:") -> link.removePrefix("tool:").takeIf { it.isNotEmpty() }?.let { open(Route.Tool(it)) }
        link.startsWith("weekly:") -> link.removePrefix("weekly:").takeIf { it.isNotEmpty() }?.let { open(Route.Weekly(it)) }
        link.startsWith("http") -> env.platform?.openUrl(link)
    }
}

/** 记在偏好里的开关（收藏、提醒）。[key] 不同互不影响。 */
@Composable
fun rememberPrefFlag(env: CommunityEnv, key: String): Pair<Boolean, (Boolean) -> Unit> {
    var on by remember(key) { mutableStateOf(env.prefs.get(key) == "1") }
    LaunchedEffect(key) { on = env.prefs.get(key) == "1" }
    return on to { value -> on = value; env.prefs.put(key, if (value) "1" else "0") }
}

/** 灰色页面上的筛选胶囊：未选中白底，选中用主题色浅底。 */
@Composable
fun PageChip(text: String, selected: Boolean, onClick: () -> Unit) {
    val extra = LocalExtraColors.current
    Text(
        text,
        Modifier.height(30.dp).clip(RoundedCornerShape(15.dp)).background(if (selected) extra.brandSoft else MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(horizontal = 12.dp, vertical = 5.dp),
        fontSize = 14.sp, maxLines = 1, color = if (selected) extra.brandInk else MaterialTheme.colorScheme.onSurface, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
    )
}

/** 一行可左右滑的筛选胶囊，[items] 是（值，文字）。 */
@Composable
fun PageChipRow(items: List<Pair<String, String>>, selected: String, onSelect: (String) -> Unit, modifier: Modifier = Modifier) {
    LazyRow(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(items, key = { it.first }) { (key, label) -> PageChip(label, key == selected) { onSelect(key) } }
    }
}

/** 品牌图标块：有官方标志就用品牌色画在浅底上；没有就用 [fallback] 的首字。[dashed] 为真时画虚线占位框（官方图标待补）。 */
@Composable
fun BrandTile(logo: BrandLogo?, fallback: String, modifier: Modifier = Modifier, size: Dp = 40.dp, corner: Dp = 10.dp, dashed: Boolean = false) {
    val shape = RoundedCornerShape(corner)
    when {
        logo != null -> Box(modifier.size(size).clip(shape).background(if (ThemeState.dark) logo.color.copy(alpha = 0.22f) else logo.tint), contentAlignment = Alignment.Center) {
            XIcon(logo.icon, tint = logo.color, size = size * 0.55f)
        }
        dashed -> Box(
            modifier.size(size).clip(shape).background(MaterialTheme.colorScheme.surface).dashedBorder(LocalExtraColors.current.line, corner),
            contentAlignment = Alignment.Center,
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                XIcon(XIcons.Image, tint = LocalExtraColors.current.ink3, size = 14.dp)
                Text("图标", fontSize = 10.sp, lineHeight = 12.sp, color = LocalExtraColors.current.ink3)
            }
        }
        else -> LetterIcon(fallback, Brand, Brand.copy(alpha = 0.14f), modifier, size)
    }
}

private fun Modifier.dashedBorder(color: Color, corner: Dp): Modifier = this.drawBehind {
    drawRoundRect(
        color, cornerRadius = androidx.compose.ui.geometry.CornerRadius(corner.toPx()),
        style = androidx.compose.ui.graphics.drawscope.Stroke(width = 1.dp.toPx(), pathEffect = androidx.compose.ui.graphics.PathEffect.dashPathEffect(floatArrayOf(10f, 8f))),
    )
}
