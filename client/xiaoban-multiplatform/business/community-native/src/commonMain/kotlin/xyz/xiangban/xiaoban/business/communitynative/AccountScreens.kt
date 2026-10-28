package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.CodeLogin
import xyz.xiangban.xiaoban.common.community.messageOf

private enum class LoginMode(val label: String) { Code("验证码登录"), Password("密码登录") }

/** 登录：验证码（新用户接着设置用户名和密码）或密码。登录成功后回到上一页。 */
@Composable
fun LoginScreen(env: CommunityEnv, done: () -> Unit, back: () -> Unit) {
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

    Column(Modifier.fillMaxSize()) {
        PageHeader(if (registrationToken != null) "设置账号" else "登录", onBack = back)
        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface).padding(Gutter), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (registrationToken == null) {
                Row { LoginMode.entries.forEach { m -> Chip(m.label, mode == m) { mode = m; error = null } } }
                OutlinedTextField(identifier, { identifier = it.trim() }, Modifier.fillMaxWidth(), label = { Text("邮箱或手机号") }, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email))
                if (mode == LoginMode.Code) {
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                        OutlinedTextField(code, { code = it.trim() }, Modifier.weight(1f), label = { Text("验证码") }, singleLine = true, keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number))
                        OutlinedButton(
                            { run { env.account.sendCode(identifier); cooldown = 60 } },
                            enabled = identifier.isNotBlank() && cooldown == 0 && !busy, shape = CardShape,
                        ) { Text(if (cooldown > 0) "${cooldown}s" else "获取验证码") }
                    }
                } else {
                    OutlinedTextField(password, { password = it }, Modifier.fillMaxWidth(), label = { Text("密码") }, singleLine = true, visualTransformation = PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password))
                }
                BrandButton(
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
                    Modifier.fillMaxWidth(),
                    enabled = !busy && identifier.isNotBlank() && (if (mode == LoginMode.Code) code.isNotBlank() else password.isNotBlank()),
                )
                Text("首次使用验证码登录会自动引导你创建账号。", color = LocalExtraColors.current.ink3, fontSize = 12.sp)
            } else {
                Text("验证通过，设置一个用户名和密码完成注册。用户名不可修改，也是你在社区里的名字。", color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp)
                OutlinedTextField(username, { username = it.trim() }, Modifier.fillMaxWidth(), label = { Text("用户名") }, singleLine = true)
                OutlinedTextField(password, { password = it }, Modifier.fillMaxWidth(), label = { Text("密码") }, singleLine = true, visualTransformation = PasswordVisualTransformation(), keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password))
                BrandButton(
                    if (busy) "提交中…" else "完成注册",
                    { run { env.session.register(registrationToken.orEmpty(), username, password); done() } },
                    Modifier.fillMaxWidth(),
                    enabled = !busy && username.isNotBlank() && password.length >= 8,
                )
                MetaTextWrap("密码至少 8 位。")
            }
            error?.let { Text(it, color = Brand, fontSize = 13.sp) }
        }
    }
}

@Composable
private fun MetaTextWrap(text: String) = Text(text, color = LocalExtraColors.current.ink3, fontSize = 12.sp)

/** “我的”：未登录是登录入口，已登录显示账号与论坛资料、未读通知，网页端功能给出入口。 */
@Composable
fun MineScreen(env: CommunityEnv, login: () -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var nickname by remember { mutableStateOf<String?>(null) }
    var points by remember { mutableStateOf<Int?>(null) }
    var unread by remember { mutableStateOf(0) }

    // 登录后向论坛确认一次身份，顺便取昵称、积分和未读数；失败不影响页面。
    LaunchedEffect(session.loggedIn) {
        if (!session.loggedIn) { nickname = null; points = null; unread = 0; return@LaunchedEffect }
        runCatching { env.forum.session().user }.onSuccess { nickname = it.nickname; points = it.points }
        runCatching { env.forum.unreadCount() }.onSuccess { unread = it }
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader("我的")
        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
            Row(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(Gutter), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                val name = nickname ?: session.user?.username
                Box(Modifier.size(56.dp).clip(CircleShape).background(Brand), contentAlignment = Alignment.Center) {
                    Text(name?.take(1)?.uppercase() ?: "?", color = MaterialTheme.colorScheme.onPrimary, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
                }
                if (session.loggedIn) {
                    Column(Modifier.weight(1f)) {
                        Text(name ?: "已登录", fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                        points?.let { MetaText("$it 积分") }
                    }
                } else {
                    Column(Modifier.weight(1f)) {
                        Text("未登录", fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                        MetaText("登录后同步你在社区的身份")
                    }
                    BrandButton("登录", login)
                }
            }
            Hairline()
            if (session.loggedIn) {
                MineRow("通知", if (unread > 0) "$unread 条未读" else null) { env.platform?.openUrl("${env.config.forumBase.trimEnd('/')}/notifications") }
                MineRow("我的作品", null) { env.platform?.openUrl("${env.config.forumBase.trimEnd('/')}/me/works") }
                MineRow("发布内容", "请在网页或电脑端发布") { env.platform?.openUrl("${env.config.forumBase.trimEnd('/')}/publish") }
            }
            MineRow("社区规范", null) { env.platform?.openUrl("${env.config.forumBase.trimEnd('/')}/rules") }
            MineRow("隐私政策", null) { env.platform?.openUrl("${env.config.forumBase.trimEnd('/')}/privacy") }
            if (session.loggedIn) {
                TextButton({ scope.launch { env.session.logout() } }, Modifier.padding(Gutter)) { Text("退出登录", color = Brand) }
            }
        }
    }
}

@Composable
private fun MineRow(title: String, detail: String?, onClick: () -> Unit) {
    Column(Modifier.background(MaterialTheme.colorScheme.surface)) {
        Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(Gutter), verticalAlignment = Alignment.CenterVertically) {
            Text(title, Modifier.weight(1f), fontSize = 15.sp)
            if (detail != null) MetaText(detail)
            Text("  ›", color = LocalExtraColors.current.ink3)
        }
        Hairline(Modifier.padding(start = Gutter))
    }
}

/** 小伴 AI 的占位页：手机端 Agent 能力还在规划，先把入口位置占住。 */
@Composable
fun AiPlaceholderScreen() {
    Column(Modifier.fillMaxSize()) {
        PageHeader("小伴")
        CenterMessage("手机上的小伴即将到来\n先去社区逛逛，或在电脑上使用完整的小伴。")
    }
}
