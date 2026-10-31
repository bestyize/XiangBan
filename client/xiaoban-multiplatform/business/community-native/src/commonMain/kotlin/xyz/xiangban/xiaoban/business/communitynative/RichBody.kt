package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.BodyBlock
import xyz.xiangban.xiaoban.common.community.bodyPlainText
import xyz.xiangban.xiaoban.common.community.parseBody

/**
 * 帖子、回复、项目的正文：文字和图片按原来的顺序排。文字可以长按选中复制；图片撑满宽度、保持比例，点一下放大看。
 * [onCopy] 不为空时，底下多一个“复制全文”。
 */
@Composable
fun RichBody(env: CommunityEnv, html: String, modifier: Modifier = Modifier, fontSize: TextUnit = 15.sp, lineHeight: TextUnit = 26.sp, onCopy: ((String) -> Unit)? = null) {
    val site = env.config.forumBase
    val blocks = remember(html, site) { parseBody(html, site) }
    var zoom by remember { mutableStateOf<String?>(null) }
    if (blocks.isEmpty()) return
    Column(modifier, verticalArrangement = Arrangement.spacedBy(10.dp)) {
        SelectionContainer {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                blocks.forEach { block ->
                    when (block) {
                        is BodyBlock.Text -> Text(block.text, fontSize = fontSize, lineHeight = lineHeight)
                        is BodyBlock.Image -> BodyImage(env, block) { zoom = block.src }
                    }
                }
            }
        }
        if (onCopy != null) Text(
            "复制全文",
            Modifier.clip(RoundedCornerShape(14.dp)).background(LocalExtraColors.current.fill).clickable { onCopy(bodyPlainText(blocks)) }.padding(horizontal = 12.dp, vertical = 5.dp),
            fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
    zoom?.let { ImageViewer(env, it) { zoom = null } }
}

@Composable
private fun BodyImage(env: CommunityEnv, image: BodyBlock.Image, onClick: () -> Unit) {
    AsyncImage(
        model = image.src, contentDescription = image.alt.ifEmpty { "图片" }, imageLoader = rememberImageLoader(), contentScale = ContentScale.FillWidth,
        modifier = Modifier.fillMaxWidth().heightIn(min = 120.dp, max = 640.dp).clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).clickable(onClick = onClick),
    )
}

/** 点图放大：黑底铺满，图片完整显示，点一下或按返回关闭。 */
@Composable
private fun ImageViewer(env: CommunityEnv, src: String, onClose: () -> Unit) {
    androidx.compose.ui.window.Dialog(onDismissRequest = onClose, properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)) {
        Box(Modifier.fillMaxSize().background(Color.Black).clickable(onClick = onClose), contentAlignment = Alignment.Center) {
            AsyncImage(model = src, contentDescription = "放大的图片", imageLoader = rememberImageLoader(), contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize())
            Box(Modifier.align(Alignment.TopEnd).statusBarsPadding().padding(8.dp)) { IconTap(XIcons.Close, "关闭", onClose, tint = Color.White) }
        }
    }
}
