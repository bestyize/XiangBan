package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.StateFlow

data class AppUpdateState(
    val automatic: Boolean = true,
    val visible: Boolean = false,
    val busy: Boolean = false,
    val version: String? = null,
    val notes: String = "",
    val message: String = "",
    val percent: Int? = null,
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
    XiaobanDialog(
        title = if (version == null) "检查更新" else "发现新版本 v$version",
        onDismissRequest = updates::dismiss,
        primaryAction = version?.let { XDialogAction(if (state.busy) "安装中…" else "安装更新", updates::install, enabled = !state.busy) },
        secondaryAction = XDialogAction(if (version == null) "关闭" else "稍后", updates::dismiss),
    ) {
        Column(
            Modifier.fillMaxWidth().verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (state.notes.isNotBlank()) Text(state.notes, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
            if (state.message.isNotBlank()) Text(state.message, style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
            state.percent?.let { percent ->
                LinearProgressIndicator(progress = { (percent.coerceIn(0, 100)) / 100f }, modifier = Modifier.fillMaxWidth())
            }
        }
    }
}
