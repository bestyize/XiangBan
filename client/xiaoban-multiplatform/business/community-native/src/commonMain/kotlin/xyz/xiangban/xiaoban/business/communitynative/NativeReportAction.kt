package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.CommunityReportReasons
import xyz.xiangban.xiaoban.common.community.messageOf

@Composable
internal fun NativeReportAction(env: CommunityEnv, targetType: String, id: Long, open: ((Route) -> Unit)?) {
    val session by env.session.state.collectAsState()
    key(targetType, id, session.accountId) {
        var show by rememberSaveable { mutableStateOf(false) }
        var submitted by rememberSaveable { mutableStateOf(false) }
        var reason by rememberSaveable { mutableStateOf("") }
        var detail by rememberSaveable { mutableStateOf("") }
        Text(if (submitted) "已提交举报" else "举报",
            Modifier.clickable(enabled = !submitted, role = Role.Button) { show = true }.padding(vertical = 4.dp),
            style = XType.Caption, color = LocalExtraColors.current.ink3)
        if (show) {
            val scope = rememberCoroutineScope()
            var busy by remember { mutableStateOf(false) }
            var error by remember { mutableStateOf<String?>(null) }
            val title = when (targetType) {
                "post" -> "举报这条回复"
                "pack" -> "举报这个作品"
                "scheme" -> "举报这个方案"
                else -> "举报这篇帖子"
            }
            CommunityCardDialog(
                title = title, icon = XIcons.Flag, iconColor = Color(0xFF695066),
                description = if (session.loggedIn) null else "登录后可举报。",
                confirmLabel = when { !session.loggedIn -> "登录"; busy -> "提交中…"; else -> "提交举报" },
                confirmEnabled = !busy && if (session.loggedIn) reason in CommunityReportReasons else open != null,
                dismissEnabled = !busy, onDismiss = { show = false },
                onConfirm = {
                    if (!session.loggedIn) {
                        show = false
                        open?.invoke(Route.Login)
                    } else if (!busy && reason in CommunityReportReasons) {
                        busy = true
                        error = null
                        scope.launch {
                            try {
                                if (targetType == "pack" || targetType == "scheme") env.contentSocial.report(targetType, id, reason, detail)
                                else env.social.report(targetType, id, reason, detail)
                                submitted = true
                                show = false
                            } catch (e: CancellationException) { throw e }
                            catch (e: Exception) { error = messageOf(e) }
                            finally { busy = false }
                        }
                    }
                },
                content = if (!session.loggedIn) null else ({
                    Column(Modifier.fillMaxWidth().selectableGroup(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        CommunityReportReasons.entries.chunked(2).forEach { pair ->
                            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                pair.forEach { (value, label) ->
                                    DialogChoice(label, reason == value, { reason = value; error = null }, Modifier.weight(1f), enabled = !busy)
                                }
                                if (pair.size == 1) Spacer(Modifier.weight(1f))
                            }
                        }
                    }
                    Spacer(Modifier.height(15.dp))
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Text("补充说明（可选）", style = XType.Caption)
                        Text("${detail.length} / 500", style = XType.Caption)
                    }
                    Spacer(Modifier.height(6.dp))
                    AreaBox(detail, { detail = it.take(500) }, Modifier.semantics { contentDescription = "补充说明（可选）" },
                        minHeight = 65.dp, enabled = !busy, filled = true)
                    error?.let {
                        Spacer(Modifier.height(8.dp))
                        DialogNotice(it, error = true)
                    }
                }),
            )
        }
    }
}
