package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties

/** 所有普通信息、确认和危险操作弹窗统一从这里创建；业务页面只提供数据与回调。 */
data class XDialogAction(
    val label: String,
    val onClick: () -> Unit,
    val enabled: Boolean = true,
    val destructive: Boolean = false,
)

/** 小伴自己的弹窗容器：不使用 Material 默认的紫色 surfaceContainerHigh。 */
@Composable
private fun XDialogSurface(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    val shape = RoundedCornerShape(24.dp)
    Column(
        modifier.clip(shape)
            .background(MaterialTheme.colorScheme.surface)
            .border(1.dp, LocalExtraColors.current.hairline, shape)
            .padding(20.dp),
        content = content,
    )
}

/**
 * 通用信息/确认/表单弹窗。
 * - 主/次操作、忙碌禁用、危险强调在此统一处理
 * - 表单和进度区可通过 content 插槽扩展；长内容由调用方设置滚动
 * - 使用系统 Dialog 窗口，让返回键、点击遮罩和无障碍焦点遵循平台行为
 */
@Composable
fun XiaobanDialog(
    title: String,
    onDismissRequest: () -> Unit,
    primaryAction: XDialogAction? = null,
    secondaryAction: XDialogAction? = null,
    dismissEnabled: Boolean = true,
    content: @Composable ColumnScope.() -> Unit = {},
) {
    Dialog(
        onDismissRequest = { if (dismissEnabled) onDismissRequest() },
        properties = DialogProperties(
            dismissOnBackPress = dismissEnabled,
            dismissOnClickOutside = dismissEnabled,
            usePlatformDefaultWidth = false,
        ),
    ) {
        BoxWithConstraints(
            Modifier.fillMaxWidth().padding(horizontal = 24.dp).widthIn(max = 360.dp),
            contentAlignment = Alignment.Center,
        ) {
            val maxBodyHeight = (maxHeight - 152.dp).coerceIn(120.dp, 420.dp)
            XDialogSurface(Modifier.fillMaxWidth()) {
                Text(title, style = XType.Title, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                Column(
                    Modifier.fillMaxWidth().padding(top = 12.dp).heightIn(max = maxBodyHeight),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                    content = content,
                )
                if (primaryAction != null || secondaryAction != null) {
                    Row(
                        Modifier.fillMaxWidth().padding(top = 20.dp),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        secondaryAction?.let { action ->
                            GhostPillButton(action.label, action.onClick, Modifier.weight(1f), enabled = action.enabled, height = 44.dp)
                        }
                        primaryAction?.let { action ->
                            XDialogPrimaryButton(action, Modifier.weight(1f))
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun XDialogPrimaryButton(action: XDialogAction, modifier: Modifier = Modifier) {
    val fill = if (action.destructive) Danger else Brand
    Box(
        modifier.height(44.dp).alpha(if (action.enabled) 1f else 0.45f)
            .clip(CircleShape).background(fill)
            .clickable(enabled = action.enabled, role = Role.Button, onClick = action.onClick)
            .padding(horizontal = 10.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            action.label, style = XType.Body, fontWeight = FontWeight.SemiBold,
            textAlign = TextAlign.Center, color = Color.White, maxLines = 1,
        )
    }
}

/** 文本确认场景的简洁入口，避免每个页面重复写标题、按钮与布局。 */
@Composable
fun XiaobanConfirmDialog(
    title: String,
    message: String,
    confirmLabel: String,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
    dismissEnabled: Boolean = true,
    confirmEnabled: Boolean = true,
    destructive: Boolean = false,
    cancelLabel: String = "取消",
) {
    XiaobanDialog(
        title = title,
        onDismissRequest = onDismiss,
        primaryAction = XDialogAction(confirmLabel, onConfirm, enabled = confirmEnabled, destructive = destructive),
        secondaryAction = XDialogAction(cancelLabel, onDismiss, enabled = dismissEnabled),
        dismissEnabled = dismissEnabled,
    ) {
        Text(message, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/**
 * 历史 CenterDialog 调用的兼容入口，集中到同一 UI 组件文件。
 * 与原实现一样嵌入父页面（而不是再嵌套一个 Dialog 窗口）。
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun CenterDialog(onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    BackHandler(onBack = onDismiss)
    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f))
            .clickable(indication = null, interactionSource = null) { onDismiss() }
            .statusBarsPadding().navigationBarsPadding().imePadding(),
        contentAlignment = Alignment.Center,
    ) {
        XDialogSurface(
            Modifier.padding(horizontal = 28.dp).widthIn(max = 336.dp)
                .clickable(indication = null, interactionSource = null) {},
            content = content,
        )
    }
}
