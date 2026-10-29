package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.CodeLogin
import xyz.xiangban.xiaoban.common.community.messageOf

private enum class LoginMode(val label: String) { Code("验证码登录"), Password("密码登录") }

/** 登录与完成注册共用一个弹窗，成功后回到当前页面。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun LoginDialog(env: CommunityEnv, done: () -> Unit, back: () -> Unit) {
    val scope = rememberCoroutineScope()
    var mode by remember { mutableStateOf(LoginMode.Code) }
    var identifier by remember { mutableStateOf("") }
    var code by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var registrationToken by remember { mutableStateOf<String?>(null) }
    var username by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var cooldown by remember { mutableStateOf(0) }

    LaunchedEffect(cooldown) { if (cooldown > 0) { delay(1000); cooldown -= 1 } }

    fun run(block: suspend () -> Unit) {
        if (busy) return
        busy = true
        error = null
        scope.launch {
            try {
                block()
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                error = messageOf(e)
                (e as? ApiException)?.retryAfter?.let { cooldown = it }
            } finally {
                busy = false
            }
        }
    }

    val registering = registrationToken != null
    CenterDialog(onDismiss = back) {
        BackHandler(enabled = registering) { registrationToken = null; error = null }
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            if (registering) IconTap(XIcons.ChevronLeft, "返回登录", { registrationToken = null; error = null })
            else {
                XIcon(XIcons.Sparkle, tint = Brand, size = 22.dp)
                Spacer(Modifier.size(8.dp))
            }
            Text(if (registering) "完成注册" else "登录小伴", Modifier.weight(1f), fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
            IconTap(XIcons.Close, "关闭", back)
        }
        Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 4.dp)) {
            if (!registering) {
                Row(Modifier.fillMaxWidth().padding(top = 4.dp).clip(CircleShape).background(LocalExtraColors.current.fill).padding(3.dp)) {
                    LoginMode.entries.forEach { m ->
                        val selected = mode == m
                        Box(
                            Modifier.weight(1f).height(40.dp).clip(CircleShape)
                                .background(if (selected) LocalExtraColors.current.brandSoft else Color.Transparent)
                                .clickable { mode = m; error = null },
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(m.label, fontSize = 14.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal, color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
                TextBox(identifier, { identifier = it.trim() }, Modifier.padding(top = 14.dp), label = if (mode == LoginMode.Code) "邮箱或手机号" else "用户名、邮箱或手机号", placeholder = if (mode == LoginMode.Code) "请输入邮箱或手机号" else "请输入用户名、邮箱或手机号", leadingIcon = XIcons.User, filled = true, fontSize = 14)
                if (mode == LoginMode.Code) {
                    Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Bottom) {
                        TextBox(code, { code = it.trim() }, Modifier.weight(1f), label = "验证码", placeholder = "6 位数字", keyboardType = KeyboardType.Number, leadingIcon = XIcons.Lock, filled = true, fontSize = 14)
                        GhostPillButton(
                            if (cooldown > 0) "${cooldown}s" else "获取验证码",
                            { run { env.account.sendCode(identifier); cooldown = 60 } },
                            enabled = identifier.isNotBlank() && cooldown == 0 && !busy, height = 48.dp, fontSize = 14,
                        )
                    }
                } else {
                    TextBox(password, { password = it }, Modifier.padding(top = 12.dp), label = "密码", placeholder = "请输入密码", password = true, keyboardType = KeyboardType.Password, leadingIcon = XIcons.Lock, filled = true, fontSize = 14)
                }
                error?.let { Text(it, Modifier.padding(top = 10.dp), color = Brand, fontSize = 13.sp) }
                PillButton(
                    if (busy) "登录中…" else "登录",
                    {
                        run {
                            if (mode == LoginMode.Code) {
                                when (val result = env.session.loginWithCode(identifier, code)) {
                                    is CodeLogin.Done -> done()
                                    is CodeLogin.NeedsRegistration -> { registrationToken = result.registrationToken }
                                }
                            } else {
                                env.session.loginWithPassword(identifier, password)
                                done()
                            }
                        }
                    },
                    Modifier.padding(top = 16.dp).fillMaxWidth(),
                    enabled = !busy && identifier.isNotBlank() && (if (mode == LoginMode.Code) code.isNotBlank() else password.isNotBlank()),
                    height = 44.dp,
                )
                if (mode == LoginMode.Code) Text("首次登录将引导注册", Modifier.padding(top = 10.dp).fillMaxWidth(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
                Text("登录即表示同意用户协议与隐私政策", Modifier.padding(top = 12.dp).fillMaxWidth(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
            } else {
                TextBox(username, { username = it.trim() }, Modifier.padding(top = 8.dp), label = "用户名", placeholder = "2–20 个字", leadingIcon = XIcons.User, filled = true, fontSize = 14)
                Text("用户名不能修改，可用字母、数字、汉字和下划线", Modifier.padding(top = 6.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                TextBox(password, { password = it }, Modifier.padding(top = 12.dp), label = "密码", placeholder = "至少 8 位", password = true, keyboardType = KeyboardType.Password, leadingIcon = XIcons.Lock, filled = true, fontSize = 14)
                error?.let { Text(it, Modifier.padding(top = 10.dp), color = Brand, fontSize = 13.sp) }
                PillButton(
                    if (busy) "提交中…" else "完成注册并登录",
                    { run { env.session.register(registrationToken.orEmpty(), username, password); done() } },
                    Modifier.padding(top = 16.dp).fillMaxWidth(),
                    enabled = !busy && username.isNotBlank() && password.length >= 8,
                    height = 44.dp,
                )
                Text("登录即表示同意用户协议与隐私政策", Modifier.padding(top = 12.dp).fillMaxWidth(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
            }
        }
    }
}

/** “我的”：头像和名字、三个常用入口、几行设置入口。网页端的功能给出入口。 */
@Composable
fun MineScreen(env: CommunityEnv, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val remoteState by env.remoteChat.state.collectAsState()
    var nickname by remember { mutableStateOf<String?>(null) }
    var points by remember { mutableStateOf<Int?>(null) }
    var unread by remember { mutableStateOf(0) }
    val site = env.config.forumBase.trimEnd('/')

    // 登录后向论坛确认一次身份，顺便取昵称、积分和未读数；失败不影响页面。
    LaunchedEffect(session.loggedIn) {
        if (!session.loggedIn) { nickname = null; points = null; unread = 0; return@LaunchedEffect }
        runCatching { env.forum.session().user }.onSuccess { nickname = it.nickname; points = it.points }
        runCatching { env.forum.unreadCount() }.onSuccess { unread = it }
        env.remoteChat.refreshPeers()
    }

    Column(Modifier.fillMaxSize()) {
        HomeBar("我的", trailing = { IconTap(XIcons.Sliders, "设置", { open(Route.Settings) }) })
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
            Row(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 16.dp), horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
                val name = nickname ?: session.user?.username
                Box(Modifier.size(64.dp).clip(CircleShape).background(Brand), contentAlignment = Alignment.Center) {
                    Text(name?.take(1)?.uppercase() ?: "?", color = Color.White, fontSize = 26.sp, fontWeight = FontWeight.SemiBold)
                }
                Column(Modifier.weight(1f)) {
                    if (session.loggedIn) {
                        Text(name ?: "已登录", fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        points?.let { Text("$it 积分", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    } else {
                        Text("未登录", fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        Text("登录后同步你在社区的身份", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                if (!session.loggedIn) PillButton("登录", { open(Route.Login) }, height = 36.dp)
            }

            Row(Modifier.padding(top = 8.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(vertical = 14.dp, horizontal = 4.dp)) {
                QuickTile(XIcons.Folder, "我的作品", Modifier.weight(1f)) { env.platform?.openUrl("$site/me/works") }
                QuickTile(XIcons.Message, if (unread > 0) "通知 · $unread" else "通知", Modifier.weight(1f)) { env.platform?.openUrl("$site/notifications") }
                QuickTile(XIcons.Monitor, "创作", Modifier.weight(1f)) { open(Route.AiChat) }
            }

            Column(Modifier.padding(top = 8.dp)) {
                val peers = remoteState.peers
                val computers = when {
                    peers.isEmpty() -> "未发现电脑"
                    peers.any { it.online } -> peers.first { it.online }.name + " · 在线"
                    else -> "${peers.size} 台电脑 · 离线"
                }
                NavRow("模型与额度", first = true) { open(Route.Models) }
                NavRow("我的电脑", computers) { open(Route.Devices) }
                NavRow("帮助与反馈") { env.platform?.openUrl("$site/c/site-feedback") }
            }
            Spacer(Modifier.height(NavInset))
        }
    }
}

@Composable
private fun QuickTile(icon: ImageVector, label: String, modifier: Modifier = Modifier, onClick: () -> Unit) {
    Column(modifier.clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick).padding(vertical = 4.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        IconBox(icon, 44.dp, 14.dp)
        Text(label, fontSize = 12.sp)
    }
}

/** 设置：账号资料在网页上改；这里放关于、协议和退出登录。 */
@Composable
fun SettingsScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val site = env.config.forumBase.trimEnd('/')
    Column(Modifier.fillMaxSize()) {
        ScreenBar("设置", onBack = back)
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
            if (session.loggedIn) {
                GroupLabel("账号")
                NavRow("昵称与头像", session.user?.username, first = true) { env.platform?.openUrl("$site/settings") }
                NavRow("绑定手机 / 邮箱") { env.platform?.openUrl("$site/settings") }
            }
            GroupLabel("关于")
            NavRow("用户协议", first = true) { env.platform?.openUrl("$site/terms") }
            NavRow("隐私政策") { env.platform?.openUrl("$site/privacy") }
            NavRow("社区规则") { env.platform?.openUrl("$site/rules") }
            if (session.loggedIn) {
                GhostPillButton("退出登录", { scope.launch { env.session.logout(); back() } }, Modifier.padding(start = 16.dp, end = 16.dp, top = 20.dp).fillMaxWidth(), textColor = Brand)
            } else {
                PillButton("登录", { open(Route.Login) }, Modifier.padding(start = 16.dp, end = 16.dp, top = 20.dp).fillMaxWidth())
            }
        }
    }
}
