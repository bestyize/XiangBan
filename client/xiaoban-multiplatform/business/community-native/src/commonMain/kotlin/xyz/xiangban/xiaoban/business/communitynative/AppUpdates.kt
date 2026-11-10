package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
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
    AlertDialog(
        onDismissRequest = updates::dismiss,
        title = { Text(if (version == null) "检查更新" else "小伴 v$version") },
        text = {
            Column(
                Modifier.verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                if (state.notes.isNotBlank()) Text(state.notes)
                if (state.message.isNotBlank()) Text(state.message)
                state.percent?.let { percent ->
                    LinearProgressIndicator(progress = { percent / 100f }, modifier = Modifier.fillMaxWidth())
                }
            }
        },
        confirmButton = {
            if (version != null) TextButton(onClick = updates::install, enabled = !state.busy) {
                Text("安装更新")
            }
        },
        dismissButton = {
            TextButton(onClick = updates::dismiss) { Text(if (version == null) "关闭" else "稍后") }
        },
    )
}
