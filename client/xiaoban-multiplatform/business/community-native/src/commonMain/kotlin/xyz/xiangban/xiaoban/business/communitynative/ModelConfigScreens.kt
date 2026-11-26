package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import io.ktor.http.Url
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.AiBalance
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CUSTOM_MODEL_TIER
import xyz.xiangban.xiaoban.common.community.CustomModelConfig
import xyz.xiangban.xiaoban.common.community.DEFAULT_CUSTOM_REASONING_EFFORTS
import xyz.xiangban.xiaoban.common.community.customModelTier
import xyz.xiangban.xiaoban.common.community.formatAiCredits
import xyz.xiangban.xiaoban.common.community.isCustomModelTier
import xyz.xiangban.xiaoban.common.community.newCustomProviderId
import xyz.xiangban.xiaoban.common.community.remoteProviderId

private val ModelSuccess: Color get() = Color(if (ThemeState.dark) 0xFF45BE83 else 0xFF25945D)
private val ModelWarning: Color get() = Color(if (ThemeState.dark) 0xFFE8AD52 else 0xFFA66208)
internal val ModelSheetScrim: Color get() = if (ThemeState.dark) Color.Black.copy(alpha = .62f) else Color(0xFF0F1014).copy(alpha = .42f)

@Composable
internal fun ModelConfigTheme(content: @Composable () -> Unit) {
    val dark = ThemeState.dark
    val colors = MaterialTheme.colorScheme.copy(
        background = Color(if (dark) 0xFF111113 else 0xFFF7F7F8),
        surface = Color(if (dark) 0xFF1C1C1F else 0xFFFFFFFF),
        onSurface = Color(if (dark) 0xFFECECEF else 0xFF1F2024),
        onSurfaceVariant = Color(if (dark) 0xFF9A9BA3 else 0xFF666870),
    )
    val extra = LocalExtraColors.current.copy(
        ink3 = Color(if (dark) 0xFF6C6D75 else 0xFF9698A0),
        hairline = Color(if (dark) 0xFF2A2A2F else 0xFFECECEF),
        fill = Color(if (dark) 0xFF29292E else 0xFFF2F2F4),
        line = Color(if (dark) 0xFF36363D else 0xFFE2E2E6),
    )
    MaterialTheme(colorScheme = colors, typography = MaterialTheme.typography) {
        CompositionLocalProvider(LocalExtraColors provides extra, LocalContentColor provides colors.onSurface, content = content)
    }
}

private data class ProtocolOption(
    val id: String,
    val name: String,
    val desc: String,
    val fallbackBase: String,
)

private val ProtocolOptions = listOf(
    ProtocolOption("chat", "OpenAI Chat Completions", "兼容绝大多数 OpenAI 接口服务商", "https://api.openai.com/v1"),
    ProtocolOption("responses", "OpenAI Responses", "新版 OpenAI 推理与多模态响应接口", "https://api.openai.com/v1"),
    ProtocolOption("anthropic", "Anthropic Messages", "Claude 原生及 Anthropic 兼容协议", "https://api.anthropic.com"),
)

private fun protocolOption(id: String): ProtocolOption =
    ProtocolOptions.firstOrNull { it.id == id } ?: ProtocolOptions.first()

private fun apiOrigin(base: String): Triple<String, String, Int>? =
    base.trim().takeIf(String::isNotEmpty)?.let {
        runCatching { Url(it).let { url -> Triple(url.protocol.name, url.host, url.port) } }.getOrNull()
    }

/** 取额度里最早到期的一项，只显示日期；没有到期时间返回 null。 */
private fun expiryText(balance: AiBalance?): String? =
    balance?.buckets?.filter { it.remaining > 0 }?.mapNotNull { it.expiresAt }?.minOrNull()?.take(10)

// ───────────── 01 · 模型配置主页（严格控色 · 通栏精修） ─────────────

@Composable
fun ModelsScreen(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) = ModelConfigTheme {
    ModelConfigHome(env, chat, back, open)
}

@Composable
private fun ModelConfigHome(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val scope = rememberCoroutineScope()
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme

    var officialPage by rememberSaveable { mutableStateOf(false) }
    var editCustom by remember { mutableStateOf(false) }
    var editingProvider by remember { mutableStateOf<String?>(null) }
    var addingPreset by remember { mutableStateOf<String?>(null) }
    var presetSheet by remember { mutableStateOf(false) }
    var managedProvider by remember { mutableStateOf<String?>(null) }
    var presetError by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(cloud.officialAiEnabled, session.loggedIn, session.generation) {
        if (!cloud.officialAiEnabled || !session.loggedIn) officialPage = false
        else chat.loadMeta()
    }

    val officialActive = session.loggedIn && !isCustomModelTier(cloud.selectedTier) &&
        (cloud.selectedTier.isBlank() || cloud.tiers.any { it.tier == cloud.selectedTier })
    val balance = cloud.balance
    val exhausted = balance != null && balance.total <= 0
    val officialModel = cloud.defaultOfficialModel

    fun isProviderActive(providerId: String): Boolean =
        cloud.selectedTier.startsWith("$CUSTOM_MODEL_TIER:$providerId:") ||
            (cloud.selectedTier == CUSTOM_MODEL_TIER && providerId == "legacy")

    fun openOfficial() {
        if (!session.loggedIn) open(Route.Login)
        else {
            chat.clearRedeemFeedback()
            officialPage = true
        }
    }

    fun openPresets() {
        presetError = null
        presetSheet = true
    }

    Box(Modifier.fillMaxSize().background(scheme.surface)) {
        Column(Modifier.fillMaxSize().navigationBarsPadding()) {
            RefinedScreenBar("模型配置", back, titleSize = 16.5f) {
                IconTap(XIcons.Plus, "添加第三方模型服务", ::openPresets, size = 20.dp)
            }
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
                if (cloud.officialAiEnabled) {
                    ModelSectionHeader("官方服务", cloud.activeTier?.displayName?.let { "当前：$it" })
                    ModelProviderRow(
                        title = "小伴 AI",
                        description = if (session.loggedIn) "跟着账号走，手机与电脑自动同步" else "登录后查看额度与官方模型",
                        active = officialActive,
                        onOpen = ::openOfficial,
                        action = if (!session.loggedIn) ({ ModelUsePill("登录", true, ::openOfficial) })
                            else if (!officialActive) ({
                                ModelUsePill("使用", !cloud.sending && officialModel != null) {
                                    officialModel?.let { chat.selectTier(it.tier) }
                                }
                            }) else null,
                        icon = { OfficialBrandTile() },
                    )
                    if (session.loggedIn) {
                        Hairline(Modifier.padding(start = 16.dp))
                        val expiry = expiryText(balance)?.takeLast(5)
                        val quotaLabel = when {
                            exhausted -> "官方点数已用完"
                            balance != null -> "剩余 ${formatAiCredits(balance.total, balance.quotaPerUnit)}" + (expiry?.let { " · 到期 $it" } ?: "")
                            cloud.loadingMeta -> "正在读取官方点数…"
                            else -> "官方点数"
                        }
                        val linkTint = if (exhausted) extra.brandInk else scheme.onSurfaceVariant
                        Row(
                            Modifier.fillMaxWidth().heightIn(min = 46.dp).clickable(role = Role.Button, onClick = ::openOfficial)
                                .padding(start = 16.dp, end = 14.dp, top = 8.dp, bottom = 8.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Text(quotaLabel, Modifier.weight(1f), style = XType.Secondary,
                                fontWeight = if (exhausted) FontWeight.Medium else FontWeight.Normal,
                                color = if (exhausted) scheme.onSurfaceVariant else extra.ink3,
                                maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                                Text(if (exhausted) "兑换点数" else "额度与模型", style = XType.Secondary,
                                    fontWeight = if (exhausted) FontWeight.SemiBold else FontWeight.Medium, color = linkTint, maxLines = 1)
                                XIcon(XIcons.ChevronRight, size = 14.dp, tint = linkTint)
                            }
                        }
                    }
                }

                ModelSectionHeader("第三方模型服务", "仅本机保存", separated = cloud.officialAiEnabled)
                cloud.customConfigError?.let {
                    Text(it, Modifier.padding(horizontal = 16.dp, vertical = 8.dp), color = Danger, style = XType.Secondary)
                }
                cloud.customProviders.forEachIndexed { index, provider ->
                    if (index > 0) Hairline(Modifier.padding(start = 68.dp))
                    val active = provider.available && isProviderActive(provider.id)
                    ModelProviderRow(
                        title = provider.name,
                        description = if (provider.available) "默认 ${provider.defaultModel} · 共 ${provider.models.size} 个模型" else "暂不可用",
                        active = active,
                        onOpen = {
                            if (provider.remoteType in listOf("free", "paid")) managedProvider = provider.id
                            else { editingProvider = provider.id; addingPreset = null; editCustom = true }
                        },
                        action = if (provider.available && !active) ({
                            ModelUsePill("使用", !cloud.sending) { chat.selectTier(customModelTier(provider.id, provider.defaultModel)) }
                        }) else null,
                        icon = { ProviderInitial(provider.name) },
                    )
                }
                if (cloud.customProviders.isNotEmpty()) Hairline(Modifier.padding(start = 16.dp))
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 52.dp).alpha(if (cloud.customConfigError == null) 1f else .45f)
                        .clickable(enabled = cloud.customConfigError == null, role = Role.Button, onClick = ::openPresets)
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    XIcon(XIcons.Plus, tint = scheme.onSurfaceVariant, size = 18.dp)
                    Text("添加第三方模型服务", style = XType.Body.copy(fontSize = 14.5.sp),
                        fontWeight = FontWeight.Medium, color = scheme.onSurfaceVariant)
                }
                ModelPageNote("第三方服务密钥仅保存在本机，不会上传到小伴账号；对话中也可以随时切换不同服务商的模型。")
            }
        }

        // 02 · 小伴 AI 官方配置二级页
        if (officialPage && cloud.officialAiEnabled) {
            OfficialAiConfigScreen(
                env = env,
                chat = chat,
                onBack = { officialPage = false },
            )
        }

        // 04 · 第三方服务商配置二级页
        if (editCustom) {
            CustomProviderEditorScreen(
                chat = chat,
                providerId = editingProvider,
                presetId = addingPreset,
                openUrl = { env.platform?.openInBrowser(it) },
                onBack = { editCustom = false },
            )
        }

        // 托管服务商（免费/付费预置）详情弹层
        managedProvider?.let { id ->
            cloud.customProviders.firstOrNull { it.id == id }?.let { provider ->
                ManagedModelSheet(
                    chat = chat,
                    providerId = provider.id,
                    providerName = provider.name,
                    models = provider.models,
                    selectedTier = cloud.selectedTier,
                    sending = cloud.sending,
                    onDismiss = { managedProvider = null },
                )
            }
        }

        // 03 · 添加第三方服务选服务商弹层
        OverlayTransition(presetSheet) { panel ->
            AddProviderSheet(
                presets = cloud.remotePresets,
                loading = cloud.catalogueLoading,
                error = presetError ?: cloud.catalogueError?.let { "未能刷新最新预置目录" },
                canRetryCatalogue = cloud.catalogueError != null,
                onRetryCatalogue = { scope.launch { chat.refreshModelCatalogue(force = true) } },
                onSelectPreset = { preset ->
                    val existing = cloud.customProviders.firstOrNull { it.id == remoteProviderId(preset.id) }
                    if (existing != null) {
                        presetSheet = false
                        if (preset.type == "third") {
                            editingProvider = existing.id; addingPreset = null; editCustom = true
                        } else {
                            managedProvider = existing.id
                        }
                    } else if (preset.type == "third") {
                        presetSheet = false; editingProvider = null; addingPreset = preset.id; editCustom = true
                    } else {
                        try {
                            chat.addManagedPreset(preset.id)
                            presetSheet = false
                        } catch (e: Exception) {
                            presetError = e.message ?: "添加失败"
                        }
                    }
                },
                onSelectCustom = {
                    presetSheet = false; editingProvider = null; addingPreset = null; editCustom = true
                },
                onDismiss = { presetSheet = false },
                panelModifier = panel,
            )
        }
    }
}

// ───────────── 02 · 小伴 AI 官方配置二级页 ─────────────

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun OfficialAiConfigScreen(env: CommunityEnv, chat: CloudChatController, onBack: () -> Unit) {
    val cloud by chat.state.collectAsState()
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    var code by remember { mutableStateOf("") }
    val balance = cloud.balance
    val selected = cloud.defaultOfficialModel

    LaunchedEffect(cloud.redeemMessage) { if (cloud.redeemMessage != null) code = "" }
    BackHandler(onBack = onBack)

    Column(Modifier.fillMaxSize().background(scheme.surface).navigationBarsPadding().imePadding()) {
        RefinedScreenBar("小伴 AI", onBack, titleSize = 16.5f) {
            RefinedBarAction("获取额度", { env.platform?.openInBrowser(CONTACT_URL) })
        }
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
            ModelSectionHeader("小伴点余额", "随账号同步")
            Column(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 18.dp, bottom = 16.dp)) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Row(Modifier.weight(1f).alignByBaseline(), horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                        Text(
                            balance?.let { formatAiCredits(it.total, it.quotaPerUnit).removeSuffix(" 点") } ?: "—",
                            Modifier.weight(1f, fill = false).alignByBaseline(),
                            style = XType.Headline.copy(fontSize = 32.sp, lineHeight = 36.sp, letterSpacing = (-.96).sp),
                            fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis,
                        )
                        Text("点", Modifier.alignByBaseline(), style = XType.Secondary, fontWeight = FontWeight.Medium,
                            color = scheme.onSurfaceVariant)
                    }
                    Text(
                        expiryText(balance)?.let { "$it 到期" } ?: if ((balance?.total ?: 0) > 0) "长期有效" else "暂无有效期",
                        Modifier.alignByBaseline(), style = XType.Caption, color = extra.ink3,
                    )
                }
                val composition = when {
                    balance == null && cloud.loadingMeta -> "正在读取账号余额…"
                    balance == null -> "暂时无法读取账号余额"
                    balance.total <= 0 -> "兑换点数或获取额度后可使用官方模型"
                    else -> "付费 ${formatAiCredits(balance.paid, balance.quotaPerUnit)} · 赠送 ${formatAiCredits((balance.total - balance.paid).coerceAtLeast(0), balance.quotaPerUnit)}"
                }
                Text(composition, Modifier.padding(top = 6.dp), style = XType.Secondary.copy(fontSize = 12.5.sp), color = extra.ink3)
                if (balance?.trial?.available == true) {
                    Text("首次使用可领取 ${formatAiCredits(balance.trial.quota, balance.quotaPerUnit)}试用点数，${balance.trial.days} 天内有效。",
                        Modifier.padding(top = 4.dp), style = XType.Caption, color = extra.ink3)
                }
                RedeemCodeBar(code, { code = it }, cloud.redeeming) { chat.redeem(code) }
                cloud.redeemMessage?.let { Text(it, Modifier.padding(top = 8.dp), style = XType.Secondary, color = ModelSuccess) }
                cloud.redeemError?.let { Text(it, Modifier.padding(top = 8.dp), style = XType.Secondary, color = Danger) }
            }

            ModelSectionHeader("默认对话模型", "点选即生效", separated = true)
            if (cloud.tiers.isEmpty()) {
                Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(if (cloud.loadingMeta) "读取中…" else (cloud.metaError ?: "暂时没有可选的官方模型"),
                        Modifier.weight(1f), style = XType.Secondary, color = extra.ink3)
                    if (cloud.metaError != null) RefinedBarAction("重试", chat::loadMeta)
                }
            }
            Column(Modifier.selectableGroup()) {
                cloud.tiers.forEachIndexed { index, tier ->
                    if (index > 0) Hairline(Modifier.padding(start = 16.dp))
                    val checked = selected?.tier == tier.tier
                    Row(
                        Modifier.fillMaxWidth().heightIn(min = 54.dp).selectable(checked, enabled = tier.available && !cloud.sending, role = Role.RadioButton) {
                            chat.configureOfficialModel(tier.tier)
                        }.alpha(if (tier.available) 1f else .45f).padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                Text(tier.displayName, Modifier.weight(1f, fill = false),
                                    style = XType.Body.copy(fontSize = 14.5.sp, lineHeight = 20.sp),
                                    fontWeight = if (checked) FontWeight(650) else FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                if (tier.isDefault) ModelNeutralTag("推荐")
                            }
                            val description = if (!tier.available && tier.requiresPaid) "需要充值后使用" else tier.description
                            if (description.isNotBlank()) Text(description, style = XType.Caption.copy(lineHeight = 17.sp), color = extra.ink3)
                        }
                        ModelCheck(checked)
                    }
                }
            }

            if (selected != null) {
                ModelSectionHeader("思考强度", selected.displayName, separated = true)
                Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp)) {
                    if (selected.reasoningEfforts.isEmpty()) {
                        Text("当前所选模型不支持调节思考强度", style = XType.Secondary.copy(fontSize = 12.5.sp), color = extra.ink3)
                    } else {
                        val levels = orderedReasoningLevels(selected.reasoningEfforts)
                        val effort = cloud.officialReasoningByTier[selected.tier]?.takeIf { it in levels }
                        OfficialReasoningSegments(levels, effort, !cloud.sending) {
                            chat.configureOfficialModel(selected.tier, it)
                        }
                    }
                }
            }
            cloud.customConfigError?.let { Text(it, Modifier.padding(16.dp), style = XType.Secondary, color = Danger) }
            ModelPageNote("默认模型与思考强度实时生效；对话中也可以随时临时切换。")
        }
    }
}

@Composable
private fun RedeemCodeBar(code: String, onCode: (String) -> Unit, redeeming: Boolean, onRedeem: () -> Unit) {
    val scheme = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val shape = RoundedCornerShape(12.dp)
    var focused by remember { mutableStateOf(false) }
    val ready = !redeeming && code.isNotBlank()
    Row(
        Modifier.padding(top = 16.dp).fillMaxWidth().height(44.dp).clip(shape)
            .background(if (focused) scheme.background else extra.fill)
            .then(if (focused) Modifier.border(1.dp, extra.line, shape) else Modifier)
            .padding(start = 14.dp, end = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        BasicTextField(
            code, onCode,
            Modifier.weight(1f).onFocusChanged { focused = it.isFocused }.semantics { contentDescription = "兑换码" },
            singleLine = true, enabled = !redeeming,
            textStyle = XType.Body.copy(fontSize = 14.sp, lineHeight = 20.sp, color = scheme.onSurface),
            cursorBrush = SolidColor(scheme.onSurface),
            decorationBox = { input ->
                Box {
                    if (code.isEmpty()) Text("输入兑换码补充点数", style = XType.Body.copy(fontSize = 14.sp, lineHeight = 20.sp),
                        color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    input()
                }
            },
        )
        Box(
            Modifier.height(36.dp).clip(RoundedCornerShape(9.dp)).background(if (ready) Brand else Color.Transparent)
                .clickable(enabled = ready, role = Role.Button, onClick = onRedeem).padding(horizontal = 16.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text(if (redeeming) "兑换中…" else "兑换", style = XType.Secondary.copy(fontSize = 13.5.sp),
                fontWeight = FontWeight.SemiBold, color = if (ready) Color.White else extra.ink3, maxLines = 1)
        }
    }
}

@Composable
private fun OfficialReasoningSegments(levels: List<String>, selected: String?, enabled: Boolean, onSelect: (String?) -> Unit) {
    val scheme = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().background(extra.fill, RoundedCornerShape(11.dp)).padding(3.dp).selectableGroup(),
        horizontalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        val shape = RoundedCornerShape(8.dp)
        (listOf(null) + levels).forEach { effort ->
            val checked = effort == selected
            val fill = if (ThemeState.dark) Color(0xFF323236) else scheme.surface
            Box(
                Modifier.weight(1f).heightIn(min = 34.dp)
                    .then(if (checked) Modifier.shadow(1.dp, shape).background(fill, shape) else Modifier)
                    .clip(shape).selectable(checked, enabled = enabled, role = Role.RadioButton) { onSelect(effort) }
                    .padding(vertical = 4.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(effortTitle(effort), style = XType.Secondary,
                    fontWeight = if (checked) FontWeight(650) else FontWeight.Medium,
                    color = if (checked) scheme.onSurface else scheme.onSurfaceVariant,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

// ───────────── 03 · 添加第三方服务选服务商弹层 ─────────────

@Composable
private fun AddProviderSheet(
    presets: List<xyz.xiangban.xiaoban.common.community.RemotePresetView>,
    loading: Boolean,
    error: String?,
    canRetryCatalogue: Boolean,
    onRetryCatalogue: () -> Unit,
    onSelectPreset: (xyz.xiangban.xiaoban.common.community.RemotePresetView) -> Unit,
    onSelectCustom: () -> Unit,
    onDismiss: () -> Unit,
    panelModifier: Modifier = Modifier,
) {
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme

    BottomSheet(
        onDismiss = onDismiss,
        panelModifier = panelModifier,
        flush = true,
        topPadding = 0.dp,
        scrim = ModelSheetScrim,
        showHandle = true,
        cancelLabel = "取消",
    ) {
        Text(
            "添加第三方服务",
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 10.dp),
            style = XType.Section,
            fontWeight = FontWeight.SemiBold,
        )
        Column(Modifier.fillMaxWidth().weight(1f, fill = false).heightIn(max = 460.dp).verticalScroll(rememberScrollState())) {
            presets.forEachIndexed { idx, preset ->
                if (idx > 0) Hairline(Modifier.padding(start = 68.dp))
                Row(
                    Modifier.fillMaxWidth().clickable { onSelectPreset(preset) }
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    ProviderInitial(preset.name, compact = true)
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text(preset.name, Modifier.weight(1f, fill = false), style = XType.Body.copy(fontSize = 14.5.sp),
                                fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            if (preset.added) ModelNeutralTag("已添加")
                        }
                        if (preset.description.isNotBlank()) {
                            Text(preset.description, style = XType.Caption, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        }
                    }
                    XIcon(XIcons.ChevronRight, size = 16.dp, tint = extra.ink3)
                }
            }
            if (loading) {
                Text("正在刷新预置目录…", Modifier.padding(horizontal = 16.dp, vertical = 10.dp), style = XType.Caption, color = extra.ink3)
            }
        }
        if (error != null) {
            Row(
                Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 12.dp)
                    .background(extra.fill, RoundedCornerShape(10.dp)).padding(horizontal = 12.dp, vertical = 5.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                XIcon(XIcons.Info, size = 15.dp, tint = ModelWarning)
                Text(error, Modifier.weight(1f), style = XType.Caption, color = ModelWarning)
                if (canRetryCatalogue) RefinedBarAction("重试", onRetryCatalogue, enabled = !loading)
            }
        }
        Spacer(Modifier.height(6.dp))
        Hairline()
        Row(
            Modifier.fillMaxWidth().clickable(onClick = onSelectCustom)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(
                Modifier.size(38.dp).clip(RoundedCornerShape(11.dp)).background(extra.fill),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(XIcons.Globe, size = 18.dp, tint = scheme.onSurface)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("自定义模型服务", style = XType.Body, fontWeight = FontWeight.SemiBold)
                Text("手动填写 API 地址、密钥与模型名称", style = XType.Caption, color = extra.ink3)
            }
            XIcon(XIcons.ChevronRight, size = 16.dp, tint = extra.ink3)
        }
        Spacer(Modifier.height(4.dp))
    }
}

// ───────────── 04 · 第三方服务商配置二级页（统一 46dp 输入框 · 协议独立选择 · 思考等级多选） ─────────────

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun CustomProviderEditorScreen(
    chat: CloudChatController,
    providerId: String?,
    presetId: String?,
    openUrl: (String) -> Unit,
    onBack: () -> Unit,
) {
    val saved = remember(providerId) { providerId?.let(chat::customProvider) }
    val initial = remember(providerId, presetId) { saved ?: presetId?.let(chat::presetDraft) }
    val cataloguePreset = remember(initial?.remoteId) { initial?.remoteId?.let(chat::presetDraft) }
    val id = remember(providerId, presetId) { initial?.id ?: newCustomProviderId() }
    var providerName by remember { mutableStateOf(initial?.name.orEmpty()) }
    var address by remember { mutableStateOf(initial?.baseUrl.orEmpty()) }
    var key by remember { mutableStateOf(initial?.apiKey.orEmpty()) }
    var models by remember { mutableStateOf(initial?.models.orEmpty()) }
    var defaultModel by remember { mutableStateOf(initial?.model.orEmpty()) }
    var modelDraft by remember { mutableStateOf("") }
    var protocol by remember { mutableStateOf(initial?.protocol ?: "chat") }
    var authMode by remember { mutableStateOf(saved?.authMode ?: "api-key") }
    var context by remember { mutableStateOf(saved?.contextTokens?.toString().orEmpty()) }
    var maxOutput by remember { mutableStateOf(saved?.maxOutputTokens?.toString().orEmpty()) }
    var supportsTools by remember { mutableStateOf(saved?.supportsTools ?: true) }
    var supportsVision by remember { mutableStateOf(saved?.supportsVision ?: true) }
    var reasoningEfforts by remember {
        mutableStateOf(saved?.effectiveReasoningEfforts() ?: initial?.effectiveReasoningEfforts() ?: DEFAULT_CUSTOM_REASONING_EFFORTS)
    }
    var advanced by rememberSaveable { mutableStateOf(true) }
    val scope = rememberCoroutineScope()
    var fetching by remember { mutableStateOf(false) }
    var fetchedModels by remember { mutableStateOf<List<String>>(emptyList()) }
    var fetchNote by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var revealKey by remember { mutableStateOf(false) }
    val keysByOrigin = remember { mutableMapOf<Triple<String, String, Int>, String>() }
    var confirmingDelete by remember { mutableStateOf(false) }
    var showProtoSheet by remember { mutableStateOf(false) }
    var showEffortSheet by remember { mutableStateOf(false) }

    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme

    fun addModel(value: String) {
        val item = value.trim()
        if (item.isEmpty()) return
        if (item !in models) models = models + item
        if (defaultModel.isEmpty()) defaultModel = item
        modelDraft = ""; error = null
    }

    fun save(activate: Boolean) {
        try {
            fun number(value: String): Int? = if (value.isBlank()) null else value.toIntOrNull() ?: throw IllegalArgumentException("请填写有效的 token 数量")
            val pending = modelDraft.trim()
            val all = (models + listOfNotNull(pending.takeIf { it.isNotEmpty() })).distinct()
            val selected = defaultModel.ifBlank { all.firstOrNull().orEmpty() }
            val config = (initial ?: CustomModelConfig(address, key, selected, id = id)).copy(
                baseUrl = address,
                apiKey = key,
                model = selected,
                protocol = protocol,
                contextTokens = number(context),
                maxOutputTokens = number(maxOutput),
                supportsTools = supportsTools,
                supportsVision = supportsVision,
                authMode = authMode,
                name = providerName,
                models = all,
                followRecommended = initial?.remoteId != null && selected == initial.recommendedDefault,
                reasoningEfforts = orderedReasoningLevels(reasoningEfforts),
            )
            chat.saveCustomProvider(config, activate)
            onBack()
        } catch (e: Exception) {
            error = e.message ?: "保存失败"
        }
    }

    BackHandler {
        when {
            showProtoSheet -> showProtoSheet = false
            showEffortSheet -> showEffortSheet = false
            else -> onBack()
        }
    }

    val activeProto = protocolOption(protocol)

    fun presetBase(option: ProtocolOption): String =
        initial?.endpoints?.get(option.id)?.takeIf(String::isNotBlank)
            ?: cataloguePreset?.baseUrl?.takeIf { cataloguePreset.protocol == option.id && it.isNotBlank() }
            ?: option.fallbackBase

    fun selectProtocol(option: ProtocolOption) {
        showProtoSheet = false
        if (option.id == protocol) return
        val currentBase = address.trim().trimEnd('/')
        if (currentBase.isBlank() || currentBase == presetBase(activeProto).trimEnd('/')) {
            val next = presetBase(option)
            val oldOrigin = apiOrigin(address)
            val nextOrigin = apiOrigin(next)
            if (oldOrigin != nextOrigin) {
                if (oldOrigin != null) keysByOrigin[oldOrigin] = key
                key = nextOrigin?.let(keysByOrigin::get).orEmpty()
                revealKey = false
            }
            address = next
        }
        protocol = option.id
        error = null
        fetchedModels = emptyList()
        fetchNote = null
    }

    val endpointPreview = remember(address, protocol) {
        val trimmed = address.trim().trimEnd('/')
        if (trimmed.isBlank()) null
        else runCatching { "POST " + CustomModelConfig(trimmed, "", "m", protocol = protocol).chatUrl() }.getOrNull()
    }

    Box(Modifier.fillMaxSize().background(scheme.surface)) {
        Column(Modifier.fillMaxSize().imePadding()) {
            RefinedScreenBar(
                providerName.trim().ifBlank { if (saved == null) "添加第三方服务" else "编辑模型服务" },
                onBack = onBack,
                titleSize = 16.5f,
            ) { RefinedBarAction("保存", { save(activate = false) }, accent = true) }

            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
                ModelSectionHeader("连接设置")
                Column(
                    Modifier.fillMaxWidth().background(scheme.surface).padding(horizontal = 16.dp, vertical = 14.dp),
                    verticalArrangement = Arrangement.spacedBy(14.dp),
                ) {
                    FieldBlock("服务商名称") {
                        ModelFieldBox(providerName, { providerName = it; error = null }, "例如 DeepSeek、OpenRouter")
                    }

                    FieldBlock("接口协议") {
                        Row(
                            Modifier
                                .fillMaxWidth()
                                .height(46.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(extra.fill)
                                .clickable(role = Role.Button) { showProtoSheet = true }
                                .padding(horizontal = 14.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.SpaceBetween,
                        ) {
                            Text(activeProto.name, Modifier.weight(1f), style = XType.Body.copy(fontSize = 14.sp),
                                fontWeight = FontWeight(550), color = scheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            XIcon(XIcons.ChevronRight, size = 16.dp, tint = extra.ink3)
                        }
                    }

                    FieldBlock("API 地址") {
                        ModelFieldBox(
                            value = address,
                            onValueChange = { address = it; error = null; fetchedModels = emptyList(); fetchNote = null },
                            placeholder = presetBase(activeProto),
                            keyboardType = KeyboardType.Uri,
                            monospace = true,
                        )
                        if (endpointPreview != null) {
                            Text(
                                endpointPreview,
                                fontSize = 11.5.sp,
                                lineHeight = 16.sp,
                                fontFamily = FontFamily.Monospace,
                                color = extra.ink3,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        if (address.trim().startsWith("http://", ignoreCase = true)) {
                            Text("HTTP 会明文传输密钥和内容，仅建议用于可信内网。", style = XType.Caption, color = Danger)
                        }
                    }

                    FieldBlock(
                        label = "API 密钥（可选）",
                        action = initial?.consoleUrl?.takeIf { it.isNotBlank() }?.let { url ->
                            {
                                Text(
                                    initial.consoleText.ifBlank { "获取 Key ↗" },
                                    Modifier.clickable { openUrl(url) },
                                    style = XType.Caption,
                                    color = scheme.onSurfaceVariant,
                                    textDecoration = TextDecoration.Underline,
                                )
                            }
                        },
                    ) {
                        ModelFieldBox(
                            value = key,
                            onValueChange = { key = it; error = null; fetchedModels = emptyList(); fetchNote = null },
                            placeholder = "没有密钥可留空",
                            password = !revealKey,
                            monospace = true,
                            trailing = {
                                Box(
                                    Modifier
                                        .size(30.dp)
                                        .clip(CircleShape)
                                        .clickable(role = Role.Button) { revealKey = !revealKey },
                                    contentAlignment = Alignment.Center,
                                ) {
                                    XIcon(if (revealKey) XIcons.EyeOff else XIcons.Eye, size = 17.dp, tint = extra.ink3,
                                        description = if (revealKey) "隐藏密钥" else "显示密钥")
                                }
                            },
                        )
                    }

                    if (protocol == "anthropic") {
                        FieldBlock("密钥认证方式") {
                            Row(
                                Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(extra.fill).padding(3.dp),
                                horizontalArrangement = Arrangement.spacedBy(4.dp),
                            ) {
                                listOf("api-key" to "x-api-key", "bearer" to "Bearer").forEach { (modeId, label) ->
                                    val on = authMode == modeId
                                    Box(
                                        Modifier
                                            .weight(1f)
                                            .height(38.dp)
                                            .clip(RoundedCornerShape(8.dp))
                                            .background(if (on) scheme.surface else Color.Transparent)
                                            .clickable(role = Role.RadioButton) { authMode = modeId; fetchedModels = emptyList(); fetchNote = null },
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        Text(
                                            label,
                                            style = XType.Secondary,
                                            fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                                            color = if (on) scheme.onSurface else scheme.onSurfaceVariant,
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
                ModelPageNote("密钥仅保存在本机应用私有存储中，不会上传到小伴账号。")

                // 模型列表区
                Row(
                    Modifier.fillMaxWidth().padding(start = 16.dp, end = 12.dp, top = 16.dp, bottom = 7.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    Text("模型 · ${models.size} 个（点选设为默认）", Modifier.weight(1f), style = XType.Caption.copy(fontSize = 12.5.sp), color = extra.ink3)
                    Row(
                        Modifier
                            .clip(RoundedCornerShape(6.dp))
                            .alpha(if (!fetching && address.isNotBlank()) 1f else 0.45f)
                            .clickable(enabled = !fetching && address.isNotBlank(), role = Role.Button) {
                                val current = CustomModelConfig(address, key, defaultModel, protocol = protocol, authMode = authMode)
                                fetching = true; fetchNote = null
                                scope.launch {
                                    try {
                                        val result = chat.fetchCustomModels(current)
                                        if (address == current.baseUrl && key == current.apiKey && protocol == current.protocol && authMode == current.authMode) {
                                            fetchedModels = result
                                            fetchNote = if (result.isEmpty()) "未发现模型，请手动填写。" else "发现 ${result.size} 个模型，点击添加。"
                                        }
                                    } catch (e: CancellationException) {
                                        throw e
                                    } catch (e: Exception) {
                                        if (address == current.baseUrl && key == current.apiKey && protocol == current.protocol && authMode == current.authMode) {
                                            fetchNote = e.message ?: "连接失败，请手动填写模型。"
                                        }
                                    } finally {
                                        fetching = false
                                    }
                                }
                            }
                            .padding(horizontal = 6.dp, vertical = 3.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        XIcon(XIcons.Refresh, size = 13.dp, tint = scheme.onSurfaceVariant)
                        Text(if (fetching) "拉取中…" else "拉取模型", style = XType.Caption, fontWeight = FontWeight.Medium, color = scheme.onSurfaceVariant)
                    }
                }

                Column(Modifier.fillMaxWidth().background(scheme.surface)) {
                    models.forEachIndexed { idx, model ->
                        if (idx > 0) Hairline(Modifier.padding(start = 16.dp))
                        val isDef = model == defaultModel
                        Row(
                            Modifier.fillMaxWidth().padding(start = 16.dp, end = 12.dp, top = 7.dp, bottom = 7.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Row(
                                Modifier.weight(1f).selectable(isDef, role = Role.RadioButton) { defaultModel = model }.padding(vertical = 2.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(10.dp),
                            ) {
                                Box(
                                    Modifier
                                        .size(18.dp)
                                        .clip(CircleShape)
                                        .border(1.6.dp, if (isDef) scheme.onSurfaceVariant else extra.line, CircleShape),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    if (isDef) Box(Modifier.size(9.dp).background(scheme.onSurfaceVariant, CircleShape))
                                }
                                Text(model, Modifier.weight(1f, fill = false), style = XType.Body.copy(fontSize = 14.sp), maxLines = 1, overflow = TextOverflow.Ellipsis)
                                if (isDef) ModelNeutralTag("默认")
                            }
                            IconTap(
                                XIcons.Trash,
                                "移除模型 $model",
                                {
                                    models = models - model
                                    if (defaultModel == model) defaultModel = models.firstOrNull().orEmpty()
                                },
                                size = 17.dp,
                                tint = extra.ink3,
                            )
                        }
                    }

                    if (models.isNotEmpty()) Hairline(Modifier.padding(start = 16.dp))
                    Row(
                        Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        ModelFieldBox(
                            value = modelDraft,
                            onValueChange = { modelDraft = it; error = null },
                            placeholder = "输入模型名称",
                            modifier = Modifier.weight(1f),
                        )
                        val canAdd = modelDraft.isNotBlank()
                        Box(
                            Modifier
                                .height(46.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(if (canAdd) Brand else extra.fill)
                                .clickable(enabled = canAdd, role = Role.Button) { addModel(modelDraft) }
                                .padding(horizontal = 16.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("添加", style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = if (canAdd) Color.White else extra.ink3)
                        }
                    }

                    fetchNote?.let {
                        Text(it, Modifier.padding(start = 16.dp, end = 16.dp, bottom = 10.dp), style = XType.Caption, color = extra.ink3)
                    }
                    if (fetchedModels.isNotEmpty()) {
                        Row(
                            Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, bottom = 12.dp),
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                        ) {
                            fetchedModels.forEach { item ->
                                val added = item in models
                                Text(
                                    item,
                                    Modifier
                                        .clip(CircleShape)
                                        .background(extra.fill)
                                        .clickable(enabled = !added) { addModel(item) }
                                        .padding(horizontal = 11.dp, vertical = 6.dp),
                                    style = XType.Caption,
                                    fontWeight = if (added) FontWeight.Medium else FontWeight.Normal,
                                    color = if (added) extra.ink3 else scheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }

                // 高级设置区（含思考等级多选）
                Spacer(Modifier.height(8.dp))
                Hairline()
                Column(Modifier.fillMaxWidth().background(scheme.surface)) {
                    Row(
                        Modifier.fillMaxWidth().height(48.dp).clickable { advanced = !advanced }.padding(horizontal = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("高级设置", Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.SemiBold)
                        XIcon(if (advanced) XIcons.ChevronUp else XIcons.ChevronDown, size = 16.dp, tint = extra.ink3)
                    }
                    if (advanced) {
                        Hairline()
                        Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                FieldBlock("上下文窗口", Modifier.weight(1f)) {
                                    ModelFieldBox(context, { context = it; error = null }, "默认自动", keyboardType = KeyboardType.Number)
                                }
                                FieldBlock("单次输出上限", Modifier.weight(1f)) {
                                    ModelFieldBox(maxOutput, { maxOutput = it; error = null }, "默认自动", keyboardType = KeyboardType.Number)
                                }
                            }
                            CalmSwitchRow("支持工具调用", supportsTools) { supportsTools = it }
                            CalmSwitchRow("支持图片输入", supportsVision) { supportsVision = it }
                            Hairline()
                            Row(
                                Modifier
                                    .fillMaxWidth()
                                    .heightIn(min = 38.dp)
                                    .clickable(role = Role.Button) { showEffortSheet = true }
                                    .padding(vertical = 4.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(10.dp),
                            ) {
                                Text("支持的思考等级", style = XType.Body.copy(fontSize = 14.sp), color = scheme.onSurface)
                                Text(
                                    summarizeReasoningLevels(reasoningEfforts),
                                    Modifier.weight(1f),
                                    style = XType.Caption.copy(fontSize = 12.5.sp),
                                    color = scheme.onSurfaceVariant,
                                    textAlign = TextAlign.End,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                XIcon(XIcons.ChevronRight, size = 15.dp, tint = extra.ink3)
                            }
                        }
                    }
                }

                if (saved != null) {
                    Spacer(Modifier.height(14.dp))
                    Column(Modifier.fillMaxWidth().background(scheme.surface).padding(horizontal = 16.dp, vertical = 6.dp)) {
                        if (confirmingDelete) {
                            Row(Modifier.fillMaxWidth().padding(vertical = 6.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text("删除此模型服务？", Modifier.weight(1f), style = XType.Secondary, color = scheme.onSurfaceVariant)
                                Text(
                                    "取消",
                                    Modifier.clickable { confirmingDelete = false }.padding(horizontal = 10.dp, vertical = 6.dp),
                                    style = XType.Secondary,
                                    color = scheme.onSurfaceVariant,
                                )
                                Text(
                                    "确认删除",
                                    Modifier.clickable {
                                        try { chat.deleteCustomProvider(saved.id); onBack() }
                                        catch (e: Exception) { error = e.message ?: "删除失败" }
                                    }.padding(horizontal = 10.dp, vertical = 6.dp),
                                    style = XType.Secondary,
                                    fontWeight = FontWeight.SemiBold,
                                    color = Danger,
                                )
                            }
                        } else {
                            Box(
                                Modifier.fillMaxWidth().height(44.dp).clickable { confirmingDelete = true },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("删除此模型服务", style = XType.Body, fontWeight = FontWeight.Medium, color = Danger)
                            }
                        }
                    }
                }
            }

            Column(Modifier.fillMaxWidth().background(scheme.surface).navigationBarsPadding()) {
                Hairline()
                error?.let { Text(it, Modifier.padding(start = 16.dp, end = 16.dp, top = 8.dp), color = Danger, style = XType.Secondary) }
                Row(
                    Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 10.dp, bottom = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Box(
                        Modifier
                            .height(44.dp)
                            .clip(CircleShape)
                            .background(extra.fill)
                            .clickable(role = Role.Button, onClick = onBack)
                            .padding(horizontal = 16.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text("取消", style = XType.Body.copy(fontSize = 14.sp), fontWeight = FontWeight.SemiBold, color = scheme.onSurfaceVariant)
                    }
                    ModelPrimaryButton(
                        label = "保存并使用",
                        onClick = { save(activate = true) },
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }

        // 协议选择 BottomSheet（按协议联动对应预置地址）
        if (showProtoSheet) {
            BottomSheet(
                onDismiss = { showProtoSheet = false },
                flush = true,
                topPadding = 0.dp,
                scrim = ModelSheetScrim,
                showHandle = true,
                cancelLabel = "取消",
            ) {
                Column(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 10.dp)) {
                    Text("接口协议", style = XType.Section, fontWeight = FontWeight.SemiBold)
                    Text("切换协议时自动同步对应的预置 API 地址", Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
                }
                Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState()).selectableGroup()) {
                  ProtocolOptions.forEachIndexed { idx, opt ->
                    if (idx > 0) Hairline(Modifier.padding(horizontal = 16.dp))
                    val on = protocol == opt.id
                    val suggestedBase = presetBase(opt)
                    val endpoint = CustomModelConfig(suggestedBase, "", "model", protocol = opt.id).chatUrl()
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .selectable(on, role = Role.RadioButton) { selectProtocol(opt) }
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Text(opt.name, style = XType.Body, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Medium)
                            Text(opt.desc, style = XType.Caption, color = extra.ink3)
                            Text(
                                "POST $endpoint",
                                fontSize = 11.5.sp,
                                lineHeight = 15.sp,
                                fontFamily = FontFamily.Monospace,
                                color = extra.ink3,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        ModelCheck(on)
                    }
                  }
                }
            }
        }

        // 自定义思考等级多选 BottomSheet
        if (showEffortSheet) {
            CustomReasoningMultiSheet(
                selected = reasoningEfforts,
                onChange = { reasoningEfforts = it },
                onDismiss = { showEffortSheet = false },
            )
        }
    }
}

@Composable
private fun ManagedModelSheet(
    chat: CloudChatController,
    providerId: String,
    providerName: String,
    models: List<String>,
    selectedTier: String,
    sending: Boolean,
    onDismiss: () -> Unit,
) {
    var confirming by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme

    BottomSheet(
        onDismiss = onDismiss,
        flush = true,
        topPadding = 0.dp,
        scrim = ModelSheetScrim,
        showHandle = true,
        cancelLabel = "取消",
    ) {
        Column(Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 10.dp)) {
            Text(providerName, style = XType.Section, fontWeight = FontWeight.SemiBold)
            Text("服务地址和模型由服务端统一维护", Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
        }
        Column(Modifier.fillMaxWidth().weight(1f, fill = false).heightIn(max = 360.dp).verticalScroll(rememberScrollState())) {
            models.forEachIndexed { idx, model ->
                if (idx > 0) Hairline(Modifier.padding(start = 16.dp))
                val tierId = customModelTier(providerId, model)
                val selected = selectedTier == tierId
                Row(
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = 50.dp)
                        .clickable(enabled = !sending, role = Role.RadioButton) {
                            chat.selectTier(tierId)
                            onDismiss()
                        }
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        model,
                        Modifier.weight(1f),
                        style = XType.Body,
                        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Box(Modifier.size(18.dp), contentAlignment = Alignment.Center) {
                        if (selected) XIcon(XIcons.Check, size = 17.dp, tint = scheme.onSurface)
                    }
                }
            }
            error?.let { Text(it, Modifier.padding(horizontal = 16.dp, vertical = 8.dp), color = Danger, style = XType.Secondary) }
        }
        Hairline()
        if (confirming) {
            Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("移除此服务？", Modifier.weight(1f), style = XType.Secondary, color = scheme.onSurfaceVariant)
                Text("取消", Modifier.clickable { confirming = false }.padding(horizontal = 10.dp, vertical = 6.dp), style = XType.Secondary, color = scheme.onSurfaceVariant)
                Text(
                    "确认移除",
                    Modifier.clickable {
                        try { chat.deleteCustomProvider(providerId); onDismiss() }
                        catch (e: Exception) { error = e.message ?: "移除失败" }
                    }.padding(horizontal = 10.dp, vertical = 6.dp),
                    style = XType.Secondary,
                    fontWeight = FontWeight.SemiBold,
                    color = Danger,
                )
            }
        } else {
            Box(
                Modifier.fillMaxWidth().height(48.dp).clickable { confirming = true },
                contentAlignment = Alignment.Center,
            ) {
                Text("移除此服务", style = XType.Body, fontWeight = FontWeight.Medium, color = Danger)
            }
        }
    }
}

// ───────────── 共享轻量化控件 ─────────────

@Composable
private fun ModelSectionHeader(title: String, trailing: String? = null, separated: Boolean = false) {
    val ink = LocalExtraColors.current.ink3
    if (separated) {
        Spacer(Modifier.height(8.dp))
        Hairline()
    }
    Row(
        Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 18.dp, bottom = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(title, style = XType.Caption.copy(fontSize = 12.5.sp), fontWeight = FontWeight(550), color = ink)
        if (!trailing.isNullOrBlank()) {
            Text(trailing, Modifier.weight(1f), style = XType.Caption, color = ink,
                textAlign = TextAlign.End, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun ModelPageNote(message: String) {
    Spacer(Modifier.height(4.dp))
    Hairline()
    Text(message, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp),
        style = XType.Caption.copy(lineHeight = 18.sp), color = LocalExtraColors.current.ink3)
}

@Composable
private fun ModelProviderRow(
    title: String,
    description: String,
    active: Boolean,
    onOpen: () -> Unit,
    action: (@Composable () -> Unit)? = null,
    icon: @Composable () -> Unit,
) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().clickable(role = Role.Button, onClick = onOpen)
            .padding(start = 16.dp, end = 10.dp, top = 14.dp, bottom = 14.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        icon()
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(7.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(title, Modifier.weight(1f, fill = false), style = XType.Body.copy(fontSize = 15.5.sp),
                    fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (active) ActiveGreenBadge()
            }
            Text(description, style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 17.sp),
                color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        action?.invoke()
        Box(Modifier.size(28.dp, 32.dp), contentAlignment = Alignment.Center) {
            XIcon(XIcons.ChevronRight, size = 15.dp, tint = extra.ink3.copy(alpha = .68f))
        }
    }
}

@Composable
private fun ProviderInitial(name: String, compact: Boolean = false) {
    Box(
        Modifier.size(if (compact) 38.dp else 40.dp).background(LocalExtraColors.current.fill, RoundedCornerShape(if (compact) 11.dp else 12.dp)),
        contentAlignment = Alignment.Center,
    ) {
        Text(name.trim().take(1).uppercase().ifBlank { "M" }, style = XType.Body.copy(fontSize = 14.sp),
            fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun OfficialBrandTile() {
    Box(Modifier.size(40.dp).background(Brand, RoundedCornerShape(12.dp)), contentAlignment = Alignment.Center) {
        XIcon(XIcons.SparkleSmall, tint = Color.White, size = 20.dp)
    }
}

@Composable
private fun ModelUsePill(label: String, enabled: Boolean, onClick: () -> Unit) {
    Box(
        Modifier.heightIn(min = 30.dp).clip(CircleShape).background(LocalExtraColors.current.fill)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 12.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(label, style = XType.Secondary.copy(fontSize = 12.5.sp), fontWeight = FontWeight(550),
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = if (enabled) 1f else .44f), maxLines = 1)
    }
}

@Composable
private fun ActiveGreenBadge() {
    Row(
        Modifier.padding(horizontal = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(5.dp),
    ) {
        Box(Modifier.size(6.dp).background(ModelSuccess, CircleShape))
        Text("使用中", style = XType.Caption.copy(lineHeight = 18.sp), fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
    }
}

@Composable
internal fun ModelNeutralTag(label: String, selected: Boolean = false) {
    val scheme = MaterialTheme.colorScheme
    Text(label, Modifier.background(if (selected) scheme.surface else LocalExtraColors.current.fill, RoundedCornerShape(6.dp))
        .padding(horizontal = 6.dp, vertical = 1.dp), style = XType.Caption.copy(fontSize = 11.sp, lineHeight = 15.sp),
        fontWeight = FontWeight.Medium, color = scheme.onSurfaceVariant, maxLines = 1)
}

@Composable
internal fun ModelCheck(selected: Boolean) {
    Box(Modifier.size(18.dp), contentAlignment = Alignment.Center) {
        if (selected) XIcon(XIcons.Check, size = 17.dp, tint = MaterialTheme.colorScheme.onSurface)
    }
}

@Composable
private fun FieldBlock(
    label: String,
    modifier: Modifier = Modifier,
    action: (@Composable () -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(modifier, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            action?.invoke()
        }
        content()
    }
}

/** 严格统一 46dp 高度的单行输入框。 */
@Composable
private fun ModelFieldBox(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    keyboardType: KeyboardType = KeyboardType.Text,
    password: Boolean = false,
    monospace: Boolean = false,
    trailing: (@Composable () -> Unit)? = null,
) {
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = modifier
            .fillMaxWidth()
            .height(46.dp)
            .clip(RoundedCornerShape(10.dp))
            .background(extra.fill)
            .padding(start = 14.dp, end = if (trailing != null) 8.dp else 14.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
            if (value.isEmpty()) {
                Text(
                    placeholder,
                    fontSize = if (monospace) 13.5.sp else 14.5.sp,
                    color = extra.ink3,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            BasicTextField(
                value = value,
                onValueChange = onValueChange,
                singleLine = true,
                textStyle = XType.Body.copy(
                    fontSize = if (monospace) 13.5.sp else 14.5.sp,
                    fontFamily = if (monospace) FontFamily.Monospace else FontFamily.Default,
                    color = scheme.onSurface,
                ),
                cursorBrush = SolidColor(scheme.onSurface),
                keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
                visualTransformation = if (password) PasswordVisualTransformation() else VisualTransformation.None,
                modifier = Modifier.fillMaxWidth(),
            )
        }
        trailing?.invoke()
    }
}

@Composable
private fun CalmSwitchRow(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().padding(vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(label, style = XType.Body, color = MaterialTheme.colorScheme.onSurface)
        Box(
            Modifier
                .size(44.dp, 26.dp)
                .clip(CircleShape)
                .background(if (checked) Success else extra.line)
                .clickable(role = Role.Switch) { onCheckedChange(!checked) }
                .padding(2.5.dp),
            contentAlignment = if (checked) Alignment.CenterEnd else Alignment.CenterStart,
        ) {
            Box(Modifier.size(21.dp).clip(CircleShape).background(Color.White))
        }
    }
}
