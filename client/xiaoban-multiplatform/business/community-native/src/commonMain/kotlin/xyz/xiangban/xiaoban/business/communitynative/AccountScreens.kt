package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.EdgeVoices
import xyz.xiangban.xiaoban.common.community.SpeechSource
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.CodeLogin
import xyz.xiangban.xiaoban.common.community.AccountSignupMethods
import xyz.xiangban.xiaoban.common.community.messageOf
import xyz.xiangban.xiaoban.common.community.FORUM_IMAGE_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import kotlinx.coroutines.CancellationException

private enum class LoginMode(val label: String) { Code("验证码登录"), Password("密码登录") }
internal const val CONTACT_URL = "https://chat.xiangban.xyz/contact"

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
    var methods by remember { mutableStateOf(AccountSignupMethods()) }

    LaunchedEffect(Unit) { methods = env.account.signupMethods() }
    val codeIdentifierLabel = when {
        methods.email && methods.phone -> "邮箱或手机号"
        methods.phone -> "手机号"
        else -> "邮箱"
    }
    val codeIdentifierAllowed = if ('@' in identifier) methods.email else methods.phone

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
            Text(if (registering) "完成注册" else "登录小伴", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
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
                            Text(m.label, style = XType.Secondary, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal, color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
                TextBox(identifier, { identifier = it.trim() }, Modifier.padding(top = 14.dp), label = if (mode == LoginMode.Code) codeIdentifierLabel else "用户名、邮箱或手机号", placeholder = if (mode == LoginMode.Code) "请输入$codeIdentifierLabel" else "请输入用户名、邮箱或手机号", leadingIcon = XIcons.User, filled = true, fontSize = 14)
                if (mode == LoginMode.Code) {
                    Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Bottom) {
                        TextBox(code, { code = it.trim() }, Modifier.weight(1f), label = "验证码", placeholder = "6 位数字", keyboardType = KeyboardType.Number, leadingIcon = XIcons.Lock, filled = true, fontSize = 14)
                        GhostPillButton(
                            if (cooldown > 0) "${cooldown}s" else "获取验证码",
                            { run { env.account.sendCode(identifier); cooldown = 60 } },
                            enabled = identifier.isNotBlank() && codeIdentifierAllowed && cooldown == 0 && !busy, height = 48.dp, fontSize = 14,
                        )
                    }
                } else {
                    TextBox(password, { password = it }, Modifier.padding(top = 12.dp), label = "密码", placeholder = "请输入密码", password = true, keyboardType = KeyboardType.Password, leadingIcon = XIcons.Lock, filled = true, fontSize = 14)
                }
                error?.let { Text(it, Modifier.padding(top = 10.dp), color = Danger, style = XType.Body) }
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
                    enabled = !busy && identifier.isNotBlank() && (if (mode == LoginMode.Code) code.isNotBlank() && codeIdentifierAllowed else password.isNotBlank()),
                    height = 44.dp,
                )
                if (mode == LoginMode.Code) Text("首次登录将引导注册", Modifier.padding(top = 10.dp).fillMaxWidth(), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
                Text("登录即表示同意用户协议与隐私政策", Modifier.padding(top = 12.dp).fillMaxWidth(), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
            } else {
                TextBox(username, { username = it.trim() }, Modifier.padding(top = 8.dp), label = "用户名", placeholder = "2–20 个字", leadingIcon = XIcons.User, filled = true, fontSize = 14)
                Text("用户名不能修改，可用字母、数字、汉字和下划线", Modifier.padding(top = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                TextBox(password, { password = it }, Modifier.padding(top = 12.dp), label = "密码", placeholder = "至少 8 位", password = true, keyboardType = KeyboardType.Password, leadingIcon = XIcons.Lock, filled = true, fontSize = 14)
                error?.let { Text(it, Modifier.padding(top = 10.dp), color = Danger, style = XType.Body) }
                PillButton(
                    if (busy) "提交中…" else "完成注册并登录",
                    { run { env.session.register(registrationToken.orEmpty(), username, password); done() } },
                    Modifier.padding(top = 16.dp).fillMaxWidth(),
                    enabled = !busy && username.isNotBlank() && password.length >= 8,
                    height = 44.dp,
                )
                Text("登录即表示同意用户协议与隐私政策", Modifier.padding(top = 12.dp).fillMaxWidth(), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
            }
        }
    }
}

/** 头像更换流程：供「我的」、个人主页和设置页共用。 */
internal class AvatarFlow(
    val state: AvatarEditorState,
    val start: () -> Unit,
    val pick: () -> Unit,
    val save: () -> Unit,
)

@Composable
internal fun rememberAvatarFlow(
    env: CommunityEnv,
    onToast: (String) -> Unit,
    onUpdated: () -> Unit = {},
): AvatarFlow {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val site = env.config.forumBase.trimEnd('/')
    val platform = env.platform
    val avatar = remember(session.accountId, session.generation) { AvatarEditorState() }

    fun start() {
        if (platform?.avatarEditorSupported == true) avatar.open() else platform?.openUrl("$site/settings")
    }

    fun pick() {
        val actions = platform ?: return
        if (avatar.picking || avatar.saving) return
        avatar.picking = true
        scope.launch {
            try {
                val file = actions.pickForumPhoto(AVATAR_MAX_BYTES) ?: return@launch
                avatar.use(actions.decodeAvatarImage(file) ?: throw ApiException(400, "这张图片打不开，换一张试试"))
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                onToast(messageOf(e))
            } finally {
                avatar.picking = false
            }
        }
    }

    // 同一裁剪重试时复用已上传的图片，直到论坛确认头像同步。
    fun save() {
        val actions = platform ?: return
        val image = avatar.image ?: return
        val crop = avatar.crop ?: return
        if (avatar.saving) return
        val owner = env.session.identity() ?: return
        avatar.status = AvatarStatus.Saving
        avatar.error = null
        scope.launch {
            try {
                val signature = avatar.signature()
                val uploaded = avatar.uploaded?.takeIf { it.first == signature }?.second ?: run {
                    val bytes = actions.encodeAvatarWebp(renderAvatar(image, crop, avatar.diameter)) ?: throw ApiException(0, "生成头像失败，请重试")
                    env.forum.uploadImage(ForumUploadFile("avatar.webp", "image/webp", bytes), FORUM_IMAGE_MAX_BYTES)
                        .also { avatar.uploaded = signature to it }
                }
                if (env.session.identity() != owner) throw ApiException(401, "账号已切换，请重新选择图片")
                try {
                    env.forum.holdAvatar(uploaded.id)
                } catch (e: ApiException) {
                    if (e.status == 404) avatar.uploaded = null
                    throw e
                }
                val url = env.forum.imageUrl(uploaded.url)
                env.session.setAvatar(url, owner)
                val synced = env.forum.session().user
                if (env.session.identity() != owner) throw ApiException(401, "账号已切换，请重新选择图片")
                if (synced.uid != env.session.state.value.user?.uid || synced.avatarUrl != url) {
                    throw ApiException(409, "论坛头像尚未同步，请重试保存")
                }
                avatar.clear()
                onUpdated()
                onToast("头像已更新")
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                avatar.status = AvatarStatus.Failed
                avatar.error = (e as? ApiException)?.message ?: "网络连接中断"
            }
        }
    }

    return remember(avatar, platform, site) { AvatarFlow(avatar, ::start, ::pick, ::save) }
}

@Composable
internal fun AvatarFlowOverlays(flow: AvatarFlow, name: String?, avatarUrl: String?) {
    if (flow.state.cropOpen) AvatarCropPage(flow.state, name, avatarUrl, onSave = flow.save)
    if (flow.state.sheet) AvatarPickSheet(flow.state, onPick = flow.pick)
}

/** 底部菜单中的通栏单行操作（图标 + 文字）。 */
@Composable
internal fun SheetMenuRow(
    icon: ImageVector,
    label: String,
    first: Boolean = false,
    danger: Boolean = false,
    onClick: () -> Unit,
) {
    val tint = if (danger) Danger else MaterialTheme.colorScheme.onSurfaceVariant
    val textColor = if (danger) Danger else MaterialTheme.colorScheme.onSurface
    Column(Modifier.fillMaxWidth()) {
        if (!first) Hairline(Modifier.padding(start = 16.dp))
        Row(
            Modifier.fillMaxWidth().heightIn(min = 50.dp).clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 13.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            XIcon(icon, tint = tint, size = 19.dp)
            Text(label, Modifier.weight(1f), style = XType.Body, color = textColor)
        }
    }
}

@Composable
internal fun SheetCancelFooter(onDismiss: () -> Unit) {
    Box(Modifier.padding(top = 6.dp).fillMaxWidth().height(8.dp).background(MaterialTheme.colorScheme.background))
    Text(
        "取消",
        Modifier.fillMaxWidth().clickable(onClick = onDismiss).padding(vertical = 15.dp),
        style = XType.Body,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
    )
}

/** “我的”：通栏精修（一体化头部 + 收藏/作品/通知三宫格 + 两组核心入口）。 */
@Composable
fun MineScreen(env: CommunityEnv, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val remoteState by env.remoteChat.state.collectAsState()
    var nickname by remember { mutableStateOf<String?>(null) }
    var unread by remember { mutableStateOf(0) }
    var avatarSheet by remember { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    val notificationRevision by env.social.notificationRevision.collectAsState()
    val site = env.config.forumBase.trimEnd('/')
    val extra = LocalExtraColors.current
    val avatarFlow = rememberAvatarFlow(env, onToast = { toast = it })

    // 登录后向论坛确认一次身份，取昵称和未读数；失败不影响页面。
    LaunchedEffect(session.loggedIn, session.accountId) {
        if (!session.loggedIn) { nickname = null; unread = 0; return@LaunchedEffect }
        runCatching { env.session.loadProfile() }
        runCatching { env.forum.session().user }.onSuccess { nickname = it.nickname }
        env.remoteChat.refreshPeers()
    }
    LaunchedEffect(session.loggedIn, session.accountId, notificationRevision) {
        if (session.loggedIn) runCatching { env.forum.unreadCount() }.onSuccess { unread = it }
    }

    val displayName = nickname ?: session.user?.username
    val publicId = session.user?.publicAccountId?.takeIf { it.isNotBlank() }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            HomeBar("我的", trailing = { IconTap(XIcons.Settings, "设置", { open(Route.Settings) }, size = 20.dp) })
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
                // 一体化白底头部：头像名片 + 三宫格快捷入口
                Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    Row(
                        Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, top = 6.dp, bottom = 18.dp),
                        horizontalArrangement = Arrangement.spacedBy(14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        if (session.loggedIn) {
                            AccountAvatar(
                                session.user?.avatarUrl,
                                displayName,
                                64.dp,
                                Modifier.border(1.dp, extra.hairline, CircleShape).clickable { avatarSheet = true },
                            )
                            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                                Text(displayName ?: "已登录", style = XType.Headline, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                publicId?.let { id ->
                                    Row(
                                        Modifier.clip(RoundedCornerShape(6.dp)).clickable {
                                            env.platform?.copy(id)
                                            toast = "已复制账号 ID $id"
                                        }.padding(vertical = 2.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                                    ) {
                                        Text("ID $id", style = XType.Caption, color = extra.ink3)
                                        XIcon(XIcons.Copy, tint = extra.ink3, size = 12.dp)
                                    }
                                }
                            }
                            Row(
                                Modifier.height(32.dp).clip(CircleShape)
                                    .clickable { session.accountId?.let { open(Route.User(it, "topics")) } }
                                    .padding(start = 8.dp, end = 4.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(2.dp),
                            ) {
                                Text("主页", style = XType.Secondary, color = extra.ink3)
                                XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 15.dp)
                            }
                        } else {
                            Box(
                                Modifier.size(64.dp).clip(CircleShape).background(extra.fill)
                                    .border(1.dp, extra.hairline, CircleShape).clickable { open(Route.Login) },
                                contentAlignment = Alignment.Center,
                            ) {
                                XIcon(XIcons.User, tint = extra.ink3, size = 26.dp)
                            }
                            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                Text("登录小伴", style = XType.Headline, fontWeight = FontWeight.SemiBold)
                                Text("同步作品、收藏与多端设备", style = XType.Secondary, color = extra.ink3)
                            }
                            Box(
                                Modifier.height(34.dp).clip(CircleShape).background(extra.brandSoft)
                                    .clickable { open(Route.Login) }.padding(horizontal = 16.dp),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("登录", style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
                            }
                        }
                    }

                    Hairline(Modifier.padding(horizontal = 16.dp))

                    Row(Modifier.fillMaxWidth().padding(horizontal = 10.dp, vertical = 14.dp)) {
                        QuickTile(XIcons.Bookmark, "我的收藏", Modifier.weight(1f)) { open(if (session.loggedIn) Route.Bookmarks else Route.Login) }
                        QuickTile(XIcons.Folder, "我的作品", Modifier.weight(1f)) { open(if (session.loggedIn) Route.MyWorks else Route.Login) }
                        QuickTile(XIcons.Bell, "通知", Modifier.weight(1f), badge = if (session.loggedIn) unread else 0) {
                            open(if (session.loggedIn) Route.Notifications else Route.Login)
                        }
                    }
                }

                // 组 1：AI 与设备
                Column(Modifier.padding(top = 10.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    val peers = remoteState.peers
                    val onlinePeer = peers.firstOrNull { it.online }
                    val computers = when {
                        peers.isEmpty() -> "未发现设备"
                        onlinePeer != null -> "${onlinePeer.name} · 在线"
                        else -> "${peers.size} 台设备 · 离线"
                    }
                    MineMenuRow(XIcons.Cpu, "模型配置", first = true) { open(Route.Models) }
                    MineMenuRow(XIcons.Monitor, "我的设备", value = computers, valueOnline = onlinePeer != null) { open(Route.Devices) }
                }

                // 组 2：帮助与社群
                Column(Modifier.padding(top = 10.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    MineMenuRow(XIcons.Question, "帮助与反馈", first = true) { env.platform?.openUrl("$site/discuss?tab=site") }
                    MineMenuRow(XIcons.Message, "交流群") { env.platform?.openInBrowser(CONTACT_URL) }
                }

                Spacer(Modifier.height(NavInset))
            }
        }

        if (avatarSheet && session.loggedIn) {
            BottomSheet(onDismiss = { avatarSheet = false }, flush = true, cornerRadius = 20.dp) {
                Text(
                    (displayName ?: "账号") + (publicId?.let { " · ID $it" } ?: ""),
                    Modifier.padding(start = 16.dp, end = 16.dp, bottom = 8.dp),
                    style = XType.Secondary,
                    color = extra.ink3,
                )
                SheetMenuRow(XIcons.Camera, "更换头像", first = true) {
                    avatarSheet = false
                    avatarFlow.start()
                }
                session.accountId?.let { uid ->
                    SheetMenuRow(XIcons.User, "查看个人主页") {
                        avatarSheet = false
                        open(Route.User(uid, "topics"))
                    }
                }
                publicId?.let { id ->
                    SheetMenuRow(XIcons.Copy, "复制账号 ID") {
                        avatarSheet = false
                        env.platform?.copy(id)
                        toast = "已复制账号 ID $id"
                    }
                }
                SheetCancelFooter { avatarSheet = false }
            }
        }

        AvatarFlowOverlays(avatarFlow, session.user?.username, session.user?.avatarUrl)
        Toast(toast) { toast = null }
    }
}

@Composable
private fun QuickTile(icon: ImageVector, label: String, modifier: Modifier = Modifier, badge: Int = 0, onClick: () -> Unit) {
    val extra = LocalExtraColors.current
    Column(
        modifier.clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick).padding(vertical = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(7.dp),
    ) {
        Box {
            Box(
                Modifier.size(42.dp).clip(RoundedCornerShape(14.dp)).background(extra.fill),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(icon, size = 20.dp)
            }
            if (badge > 0) {
                Box(
                    Modifier.align(Alignment.TopEnd).offset(x = 5.dp, y = (-4).dp)
                        .defaultMinSize(minWidth = 17.dp, minHeight = 17.dp)
                        .clip(CircleShape)
                        .background(Danger)
                        .border(1.5.dp, MaterialTheme.colorScheme.surface, CircleShape)
                        .padding(horizontal = 4.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        if (badge > 99) "99+" else "$badge",
                        color = Color.White,
                        fontSize = 10.sp,
                        lineHeight = 12.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                }
            }
        }
        Text(label, style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun MineMenuRow(
    icon: ImageVector,
    title: String,
    value: String? = null,
    valueOnline: Boolean = false,
    first: Boolean = false,
    onClick: () -> Unit,
) {
    val extra = LocalExtraColors.current
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        if (!first) Hairline(Modifier.padding(start = 47.dp))
        Row(
            Modifier.fillMaxWidth().heightIn(min = 52.dp).clickable(onClick = onClick)
                .padding(start = 16.dp, end = 14.dp, top = 12.dp, bottom = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            XIcon(icon, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 19.dp)
            Text(title, Modifier.weight(1f), style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (!value.isNullOrBlank()) {
                Row(
                    Modifier.widthIn(max = 170.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
                ) {
                    if (valueOnline) Box(Modifier.size(6.dp).clip(CircleShape).background(Success))
                    Text(
                        value,
                        style = XType.Secondary,
                        color = if (valueOnline) Success else extra.ink3,
                        fontWeight = if (valueOnline) FontWeight.Medium else FontWeight.Normal,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
            XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 15.dp)
        }
    }
}

/** 设置：头像在 App 内更换（平台不支持时打开网页）；其余账号资料在网页上改，这里还放关于、协议和退出登录。 */
@Composable
fun SettingsScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val site = env.config.forumBase.trimEnd('/')
    var toast by remember { mutableStateOf<String?>(null) }
    val avatarFlow = rememberAvatarFlow(env, onToast = { toast = it })

    Box(Modifier.fillMaxSize()) {
    Column(Modifier.fillMaxSize()) {
        ScreenBar("设置", onBack = back)
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
            if (session.loggedIn) {
                GroupLabel("账号")
                AvatarRow(session.user?.avatarUrl, session.user?.username, onClick = avatarFlow.start)
                if (avatarFlow.state.pending) AvatarPendingRow(onDiscard = avatarFlow.state::clear, onResume = avatarFlow.state::open)
                NavRow("绑定手机 / 邮箱") { env.platform?.openUrl("$site/settings") }
            }
            GroupLabel("外观与声音")
            NavRow("主题", ThemeState.mode.label, first = true) { open(Route.Appearance) }
            NavRow("声音", SpeechState.settings.let { if (it.source == SpeechSource.Edge) "Edge · ${EdgeVoices.byId(it.voice).name}" else it.source.label }) { open(Route.Voice) }
            env.platform?.toolEnvironment?.let { environment ->
                val tools by environment.state.collectAsState()
                GroupLabel("本机工具")
                NavRow("环境配置", tools.status.label, first = true) { open(Route.ToolEnvironment) }
            }
            GroupLabel("关于")
            val updates = env.platform?.appUpdates
            if (updates != null) {
                val update by updates.state.collectAsState()
                NavRow("检查更新", update.version?.let { "可更新至 v$it" } ?: "v${updates.currentVersion}", first = true, onClick = updates::check)
                SwitchRow("自动检查更新", "打开小伴时检查新版本", update.automatic, updates::setAutomatic)
            }
            NavRow("用户协议", first = updates == null) { env.platform?.openUrl("$site/terms") }
            NavRow("隐私政策") { env.platform?.openUrl("$site/privacy") }
            NavRow("社区规则") { env.platform?.openUrl("$site/rules") }
            NavRow("联系我们") { env.platform?.openInBrowser(CONTACT_URL) }
            if (session.loggedIn) {
                GhostPillButton("退出登录", { scope.launch { env.session.logout(); back() } }, Modifier.padding(start = 16.dp, end = 16.dp, top = 20.dp).fillMaxWidth(), textColor = Danger, onPage = true)
            } else {
                PillButton("登录", { open(Route.Login) }, Modifier.padding(start = 16.dp, end = 16.dp, top = 20.dp).fillMaxWidth())
            }
        }
    }
    AvatarFlowOverlays(avatarFlow, session.user?.username, session.user?.avatarUrl)
    Toast(toast) { toast = null }
    }
}
