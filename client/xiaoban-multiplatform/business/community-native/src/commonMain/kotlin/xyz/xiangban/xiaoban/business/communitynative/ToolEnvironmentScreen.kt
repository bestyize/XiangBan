package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.StateFlow

enum class ToolEnvironmentStatus(val label: String) {
    Unavailable("当前安装包不可用"), Missing("未安装"), Downloading("正在下载"),
    Installing("正在解包"), Checking("正在检测"), Ready("已安装"),
    Removing("正在移除"), Failed("操作失败"),
}

data class ToolEnvironmentState(
    val status: ToolEnvironmentStatus,
    val downloadBytes: Long = 0,
    val downloadedBytes: Long = 0,
    val installed: Boolean = false,
    val versions: Map<String, String> = emptyMap(),
    val error: String? = null,
)

interface ToolEnvironmentController {
    val state: StateFlow<ToolEnvironmentState>
    fun install()
    fun check()
    fun cancel()
    fun remove()
}

@Composable
internal fun ToolEnvironmentScreen(controller: ToolEnvironmentController?, back: () -> Unit) {
    val status = controller?.state?.collectAsState()?.value
        ?: ToolEnvironmentState(ToolEnvironmentStatus.Unavailable)
    var confirmRemove by rememberSaveable { mutableStateOf(false) }
    val busy = status.status in setOf(
        ToolEnvironmentStatus.Downloading, ToolEnvironmentStatus.Installing,
        ToolEnvironmentStatus.Checking, ToolEnvironmentStatus.Removing,
    )
    Column(Modifier.fillMaxSize()) {
        ScreenBar("环境配置", onBack = back)
        GroupLabel("本机工具环境")
        Column(
            Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(status.status.label, style = MaterialTheme.typography.titleMedium)
            Text("工具环境按需下载，包含 Ubuntu、Node.js、npm、npx、Python、pip 和 Git。", style = XType.Caption)
            if (status.downloadBytes > 0) {
                Text("下载大小约 ${(status.downloadBytes + 999_999) / 1_000_000} MB", style = XType.Caption)
            }
            if (status.status == ToolEnvironmentStatus.Downloading && status.downloadBytes > 0) {
                val progress = (status.downloadedBytes.toFloat() / status.downloadBytes).coerceIn(0f, 1f)
                LinearProgressIndicator(progress = { progress }, modifier = Modifier.fillMaxWidth())
                Text("${(progress * 100).toInt()}%", style = XType.Caption)
            } else if (busy) {
                LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            }
            status.error?.let { Text(it, color = MaterialTheme.colorScheme.error, style = XType.Caption) }
            status.versions.forEach { (tool, version) ->
                Text("$tool · $version", style = XType.Caption)
            }
            if (controller != null && status.status != ToolEnvironmentStatus.Unavailable) {
                when {
                    status.status == ToolEnvironmentStatus.Downloading || status.status == ToolEnvironmentStatus.Installing ->
                        GhostPillButton("取消安装", controller::cancel, Modifier.fillMaxWidth(), onPage = true)
                    status.installed && !busy -> {
                        PillButton("检测工具", controller::check, Modifier.fillMaxWidth())
                        if (status.status == ToolEnvironmentStatus.Failed) {
                            GhostPillButton("重新安装", controller::install, Modifier.fillMaxWidth(), onPage = true)
                        }
                        GhostPillButton("移除环境", { confirmRemove = true }, Modifier.fillMaxWidth(), onPage = true)
                    }
                    !busy -> PillButton(
                        if (status.status == ToolEnvironmentStatus.Failed) "重新下载" else "下载并安装",
                        controller::install, Modifier.fillMaxWidth(),
                    )
                }
            }
        }
    }
    if (confirmRemove && controller != null) {
        XiaobanConfirmDialog(
            title = "移除工具环境？",
            message = "已下载的运行时和其中安装的工具会被删除。",
            confirmLabel = "移除",
            destructive = true,
            onConfirm = { confirmRemove = false; controller.remove() },
            onDismiss = { confirmRemove = false },
        )
    }
}
