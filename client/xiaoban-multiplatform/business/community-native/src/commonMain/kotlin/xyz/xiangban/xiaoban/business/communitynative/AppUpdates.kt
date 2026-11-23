package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.StateFlow

enum class AppUpdatePhase { Checking, Latest, CheckFailed, Available, Downloading, InstallPending, InstallerOpened, InstallFailed }

data class AppUpdateState(
    val automatic: Boolean = true,
    val visible: Boolean = false,
    val busy: Boolean = false,
    val version: String? = null,
    val notes: String = "",
    val message: String = "",
    val percent: Int? = null,
    val phase: AppUpdatePhase = AppUpdatePhase.Checking,
)

interface AppUpdates {
    val currentVersion: String
    val state: StateFlow<AppUpdateState>
    fun setAutomatic(enabled: Boolean)
    fun check()
    fun install()
    fun dismiss()
}

@Composable
internal fun AppUpdateDialog(updates: AppUpdates) {
    val state by updates.state.collectAsState()
    if (!state.visible) return
    val version = state.version
    val phase = state.phase
    val title = when (phase) {
        AppUpdatePhase.Checking -> "正在检查更新…"
        AppUpdatePhase.Latest -> "已是最新版本"
        AppUpdatePhase.CheckFailed -> "检查更新失败"
        else -> version?.let { "小伴 v$it" } ?: "检查更新"
    }
    val description = when (phase) {
        AppUpdatePhase.Latest -> "v${updates.currentVersion}"
        AppUpdatePhase.Available -> state.notes
        else -> null
    }
    val canInstall = version != null && phase in setOf(AppUpdatePhase.Available, AppUpdatePhase.InstallPending, AppUpdatePhase.InstallFailed)
    CommunityCardDialog(
        title = title, description = description, icon = XIcons.Phone,
        cover = if (version != null) ({ AppUpdateDialogCover() }) else null,
        confirmLabel = if (canInstall) "安装更新" else null, actionIcon = XIcons.ArrowRight,
        confirmEnabled = !state.busy, onConfirm = updates::install,
        dismissLabel = if (version == null) "关闭" else "稍后", onDismiss = updates::dismiss,
        content = when (phase) {
            AppUpdatePhase.Checking, AppUpdatePhase.Latest -> null
            AppUpdatePhase.Downloading -> ({
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text(state.message, Modifier.weight(1f), style = XType.Secondary)
                    state.percent?.let { Text("$it%", style = XType.Secondary) }
                }
                Spacer(Modifier.height(8.dp))
                val percent = state.percent
                if (percent == null) LinearProgressIndicator(modifier = Modifier.fillMaxWidth(), color = Brand)
                else LinearProgressIndicator(progress = { percent.coerceIn(0, 100) / 100f }, modifier = Modifier.fillMaxWidth(), color = Brand)
            })
            AppUpdatePhase.Available -> ({
                Row(Modifier.fillMaxWidth().background(LocalExtraColors.current.fill, RoundedCornerShape(12.dp)).padding(12.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    XIcon(XIcons.Shield, size = 16.dp, tint = LocalExtraColors.current.ink3)
                    Text(state.message, style = XType.Secondary)
                }
            })
            else -> ({ DialogNotice(state.message, error = phase == AppUpdatePhase.CheckFailed || phase == AppUpdatePhase.InstallFailed) })
        },
    )
}
