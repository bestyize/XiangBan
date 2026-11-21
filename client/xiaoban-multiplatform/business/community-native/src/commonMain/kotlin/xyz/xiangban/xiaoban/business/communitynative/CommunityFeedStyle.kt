package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

internal val CommunityFeedPadding = PaddingValues(start = 6.dp, end = 6.dp, top = 8.dp, bottom = 20.dp + NavInset)
internal val feedInk @Composable get() = if (ThemeState.dark) Color(0xFFF8FAFC) else Color(0xFF111827)
internal val feedSecondary @Composable get() = if (ThemeState.dark) Color(0xFFCBD5E1) else Color(0xFF475569)
internal val feedMuted @Composable get() = if (ThemeState.dark) Color(0xFF8895AA) else Color(0xFF838E9F)
internal val feedFill @Composable get() = if (ThemeState.dark) Color(0xFF232A3B) else Color(0xFFEBEFF7)

@Composable
internal fun Modifier.communityFeedCard(corner: Dp = 12.dp): Modifier {
    val shape = RoundedCornerShape(corner)
    val dark = ThemeState.dark
    val ink = if (dark) Color.White else Color(0xFF0F172A)
    val surface = if (dark) Color(0xFF191F2C) else Color.White
    val shadow = Color(0xFF0F172A).copy(alpha = if (dark) 0.45f else 0.12f)
    return shadow(5.dp, shape, clip = false, ambientColor = shadow, spotColor = shadow)
        .clip(shape).background(surface).border(1.dp, ink.copy(alpha = if (dark) 0.075f else 0.055f), shape)
        .padding(1.dp)
}

internal fun feedText(size: Double, line: Double) = XType.Caption.copy(fontSize = size.sp, lineHeight = line.sp)
internal fun feedText(size: Int, line: Double = size * 1.4) = feedText(size.toDouble(), line)

@Composable
internal fun FeedTag(text: String, modifier: Modifier = Modifier, tint: Color = feedSecondary, fill: Color = feedFill) {
    Text(text, modifier.background(fill, CircleShape).padding(horizontal = 6.dp, vertical = 1.5.dp),
        style = feedText(10, 13.5), color = tint, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
}
