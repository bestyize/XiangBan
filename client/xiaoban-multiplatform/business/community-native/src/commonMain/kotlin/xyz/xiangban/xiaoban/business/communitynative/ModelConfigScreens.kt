package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.AiBalance
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CUSTOM_MODEL_TIER
import xyz.xiangban.xiaoban.common.community.CustomModelConfig
import xyz.xiangban.xiaoban.common.community.DEFAULT_CUSTOM_REASONING_EFFORTS
import xyz.xiangban.xiaoban.common.community.bucketKindLabel
import xyz.xiangban.xiaoban.common.community.customModelTier
import xyz.xiangban.xiaoban.common.community.formatAiCredits
import xyz.xiangban.xiaoban.common.community.isCustomModelTier
import xyz.xiangban.xiaoban.common.community.newCustomProviderId
import xyz.xiangban.xiaoban.common.community.remoteProviderId

private val ModelSuccessSoft: Color get() = Color(if (ThemeState.dark) 0xFF183024L else 0xFFEAF6F0L)
private val ModelWarn: Color get() = Color(if (ThemeState.dark) 0xFFE8AD52L else 0xFFA66208L)
private val ModelWarnSoft: Color get() = Color(if (ThemeState.dark) 0xFF332614L else 0xFFFDF4E5L)

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

/** 取额度里最早到期的一项，只显示日期；没有到期时间返回 null。 */
private fun expiryText(balance: AiBalance?): String? =
    balance?.buckets?.filter { it.remaining > 0 }?.mapNotNull { it.expiresAt }?.minOrNull()?.take(10)

// ───────────── 01 · 模型配置主页（严格控色 · 通栏精修） ─────────────

@Composable
fun ModelsScreen(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) {
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

    LaunchedEffect(cloud.officialAiEnabled) { if (!cloud.officialAiEnabled) officialPage = false }
    LaunchedEffect(session.loggedIn, session.accountId) { if (session.loggedIn) chat.loadMeta() }

    val officialActive = cloud.officialAiEnabled && session.loggedIn && !isCustomModelTier(cloud.selectedTier) &&
        (cloud.selectedTier.isBlank() || cloud.tiers.any { it.tier == cloud.selectedTier })
    val quota = quotaText(cloud.balance)
    val exhausted = cloud.balance != null && quota == "点数已用完"
    val activeModelLabel = cloud.modelName()

    fun isProviderActive(providerId: String): Boolean =
        cloud.selectedTier.startsWith("$CUSTOM_MODEL_TIER:$providerId:") ||
            (cloud.selectedTier == CUSTOM_MODEL_TIER && providerId == "legacy")

    Box(Modifier.fillMaxSize().background(scheme.background)) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar(
                "模型配置",
                onBack = back,
                trailing = {
                    IconTap(XIcons.Plus, "添加第三方服务", { presetError = null; presetSheet = true }, size = 20.dp)
                },
            )
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 28.dp)) {
                if (cloud.officialAiEnabled) {
                    ModelSectionHeader(
                        title = "官方服务",
                        trailing = activeModelLabel.takeIf(String::isNotBlank)?.let { "当前：$it" },
                    )
                    Column(Modifier.fillMaxWidth().background(scheme.surface)) {
                        if (!session.loggedIn) {
                            Row(
                                Modifier.fillMaxWidth().clickable { open(Route.Login) }
                                    .padding(horizontal = 16.dp, vertical = 14.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                OfficialBrandTile()
                                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                                    Text("小伴 AI", style = XType.Body, fontWeight = FontWeight.SemiBold)
                                    Text("登录后查看额度与官方模型", style = XType.Caption, color = extra.ink3)
                                }
                                Text(
                                    "登录",
                                    Modifier.clip(CircleShape).background(extra.fill).padding(horizontal = 12.dp, vertical = 5.dp),
                                    style = XType.Caption,
                                    fontWeight = FontWeight.Medium,
                                    color = scheme.onSurfaceVariant,
                                )
                                XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 16.dp)
                            }
                        } else {
                            Row(
                                Modifier.fillMaxWidth().clickable { officialPage = true; chat.clearRedeemFeedback() }
                                    .padding(horizontal = 16.dp, vertical = 14.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                OfficialBrandTile()
                                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                        Text("小伴 AI", style = XType.Body, fontWeight = FontWeight.SemiBold)
                                        if (officialActive) ActiveGreenBadge()
                                    }
                                    val sub = if (exhausted) "点数已用完，点击兑换或获取额度" else "随账号同步 · 手机与电脑通用"
                                    Text(sub, style = XType.Caption, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                                if (exhausted) {
                                    Text(
                                        "0 点 · 兑换",
                                        Modifier.clip(CircleShape).background(ModelWarnSoft).padding(horizontal = 8.dp, vertical = 3.dp),
                                        style = XType.Caption,
                                        fontWeight = FontWeight.Medium,
                                        color = ModelWarn,
                                    )
                                } else {
                                    val balanceShort = cloud.balance?.let { formatAiCredits(it.total, it.quotaPerUnit) }
                                    if (!balanceShort.isNullOrBlank()) {
                                        Text(balanceShort, style = XType.Secondary, fontWeight = FontWeight.Medium, color = scheme.onSurfaceVariant)
                                    }
                                }
                                XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 16.dp)
                            }
                        }
                    }
                }

                ModelSectionHeader(title = "第三方模型服务", trailing = "仅本机保存")

                cloud.customConfigError?.let {
                    Text(it, Modifier.padding(horizontal = 16.dp, vertical = 8.dp), color = Danger, style = XType.Secondary)
                }

                Column(Modifier.fillMaxWidth().background(scheme.surface)) {
                    cloud.customProviders.forEachIndexed { idx, provider ->
                        val active = isProviderActive(provider.id)
                        val openConfig = {
                            if (provider.remoteType in listOf("free", "paid")) managedProvider = provider.id
                            else { editingProvider = provider.id; addingPreset = null; editCustom = true }
                        }
                        if (idx > 0) Hairline(Modifier.padding(start = 68.dp))
                        Row(
                            Modifier.fillMaxWidth().clickable(onClick = openConfig)
                                .padding(horizontal = 16.dp, vertical = 14.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            Box(
                                Modifier.size(40.dp).clip(RoundedCornerShape(12.dp)).background(extra.fill),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    provider.name.trim().take(1).uppercase().ifBlank { "M" },
                                    style = XType.Body,
                                    fontWeight = FontWeight.SemiBold,
                                    color = scheme.onSurface,
                                )
                            }
                            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                    Text(
                                        provider.name,
                                        Modifier.weight(1f, fill = false),
                                        style = XType.Body,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    when {
                                        active -> ActiveGreenBadge()
                                        !provider.available -> Text(
                                            "暂不可用",
                                            Modifier.clip(CircleShape).background(extra.fill).padding(horizontal = 7.dp, vertical = 2.dp),
                                            style = XType.Caption,
                                            color = extra.ink3,
                                        )
                                    }
                                }
                                Text(
                                    "默认 ${provider.defaultModel} · 共 ${provider.models.size} 个模型",
                                    style = XType.Caption,
                                    color = extra.ink3,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                            XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 16.dp)
                        }
                    }

                    if (cloud.customProviders.isNotEmpty()) Hairline(Modifier.padding(start = 16.dp))
                    Row(
                        Modifier.fillMaxWidth().heightIn(min = 50.dp)
                            .alpha(if (cloud.customConfigError == null) 1f else 0.45f)
                            .clickable(enabled = cloud.customConfigError == null) { presetError = null; presetSheet = true }
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        XIcon(XIcons.Plus, tint = scheme.onSurfaceVariant, size = 18.dp)
                        Text("添加第三方服务", style = XType.Body, fontWeight = FontWeight.Medium, color = scheme.onSurfaceVariant)
                    }
                }

                Text(
                    "第三方密钥仅保存在本机，不会上传到小伴账号；对话时也可随时切换不同服务商的模型。",
                    Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp),
                    style = XType.Caption,
                    color = extra.ink3,
                )
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
                error = presetError ?: cloud.catalogueError,
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
private fun OfficialAiConfigScreen(
    env: CommunityEnv,
    chat: CloudChatController,
    onBack: () -> Unit,
) {
    val cloud by chat.state.collectAsState()
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    var code by remember { mutableStateOf("") }
    var effortTier by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(cloud.redeemMessage) { if (cloud.redeemMessage != null) code = "" }
    BackHandler(onBack = onBack)

    Box(Modifier.fillMaxSize().background(scheme.background)) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar(
                "小伴 AI",
                onBack = onBack,
                trailing = {
                    Text(
                        "获取额度",
                        Modifier
                            .clip(CircleShape)
                            .clickable(role = Role.Button) { env.platform?.openInBrowser(CONTACT_URL) }
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        style = XType.Secondary,
                        fontWeight = FontWeight.Medium,
                        color = scheme.onSurfaceVariant,
                    )
                },
            )
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 28.dp)) {
                ModelSectionHeader("额度与兑换", "随账号同步")
                Column(Modifier.fillMaxWidth().background(scheme.surface).padding(16.dp)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.SpaceBetween) {
                        Column(Modifier.weight(1f)) {
                            Text(
                                text = cloud.balance?.let { formatAiCredits(it.total, it.quotaPerUnit) } ?: "—",
                                fontSize = 26.sp,
                                lineHeight = 32.sp,
                                fontWeight = FontWeight.Bold,
                                color = scheme.onSurface,
                            )
                            cloud.balance?.let { balance ->
                                Text(
                                    "其中付费点数 ${formatAiCredits(balance.paid, balance.quotaPerUnit)}",
                                    Modifier.padding(top = 4.dp),
                                    style = XType.Caption,
                                    color = extra.ink3,
                                )
                            }
                        }
                        Text(
                            text = expiryText(cloud.balance)?.let { "$it 到期" } ?: "到期 —",
                            style = XType.Caption,
                            color = extra.ink3,
                        )
                    }
                    cloud.balance?.let { balance ->
                        if (balance.trial.available) {
                            Text(
                                "首次使用自动发放 ${formatAiCredits(balance.trial.quota, balance.quotaPerUnit)} 试用点数，${balance.trial.days} 天内有效。",
                                Modifier.padding(top = 8.dp),
                                style = XType.Caption,
                                color = extra.ink3,
                            )
                        }
                        balance.buckets.forEach { bucket ->
                            Text(
                                "${bucketKindLabel(bucket.kind)} · ${formatAiCredits(bucket.remaining, balance.quotaPerUnit)} / ${formatAiCredits(bucket.total, balance.quotaPerUnit)}" +
                                    (bucket.expiresAt?.let { " · ${it.take(10)} 到期" } ?: " · 长期有效"),
                                Modifier.padding(top = 4.dp),
                                style = XType.Caption,
                                color = extra.ink3,
                            )
                        }
                    }

                    Spacer(Modifier.height(14.dp))
                    Hairline()
                    Spacer(Modifier.height(14.dp))

                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        ModelFieldBox(
                            value = code,
                            onValueChange = { code = it },
                            placeholder = "输入兑换码补充点数",
                            modifier = Modifier.weight(1f),
                        )
                        val ready = !cloud.redeeming && code.isNotBlank()
                        Box(
                            Modifier
                                .height(46.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(if (ready) extra.brandSoft else extra.fill)
                                .then(if (ready) Modifier.border(1.dp, Brand.copy(alpha = if (ThemeState.dark) 0.36f else 0.24f), RoundedCornerShape(10.dp)) else Modifier)
                                .clickable(enabled = ready, role = Role.Button) { chat.redeem(code) }
                                .padding(horizontal = 16.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                text = if (cloud.redeeming) "兑换中…" else "兑换",
                                style = XType.Secondary,
                                fontWeight = FontWeight.SemiBold,
                                color = if (ready) extra.brandInk else extra.ink3,
                            )
                        }
                    }
                    cloud.redeemMessage?.let {
                        Text(it, Modifier.padding(top = 8.dp), style = XType.Secondary, fontWeight = FontWeight.Medium, color = Success)
                    }
                    cloud.redeemError?.let {
                        Text(it, Modifier.padding(top = 8.dp), style = XType.Secondary, color = Danger)
                    }
                }

                ModelSectionHeader("默认对话模型", "点选即刻生效")
                Column(Modifier.fillMaxWidth().background(scheme.surface)) {
                    if (cloud.tiers.isEmpty()) {
                        Text(
                            if (cloud.loadingMeta) "读取中…" else (cloud.metaError ?: "暂时没有可选的官方模型"),
                            Modifier.padding(16.dp),
                            style = XType.Secondary,
                            color = extra.ink3,
                        )
                    }
                    cloud.tiers.forEachIndexed { idx, tier ->
                        if (idx > 0) Hairline(Modifier.padding(start = 16.dp))
                        val selected = cloud.selectedTier == tier.tier
                        val desc = (if (!tier.available && tier.requiresPaid) "需要充值后使用　" else "") + tier.description
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .heightIn(min = 54.dp)
                                .alpha(if (tier.available) 1f else 0.45f)
                                .padding(horizontal = 16.dp, vertical = 11.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Column(
                                modifier = Modifier
                                    .weight(1f)
                                    .clickable(enabled = tier.available && !cloud.sending, role = Role.RadioButton) {
                                        chat.selectTier(tier.tier)
                                    }
                                    .padding(vertical = 2.dp),
                            ) {
                                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                    Text(
                                        tier.displayName,
                                        Modifier.weight(1f, fill = false),
                                        style = XType.Body,
                                        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    if (tier.isDefault) {
                                        Text(
                                            "推荐",
                                            Modifier.clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 6.dp, vertical = 1.dp),
                                            style = XType.Caption,
                                            color = scheme.onSurfaceVariant,
                                        )
                                    }
                                }
                                if (desc.isNotBlank()) {
                                    Text(desc, Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
                                }
                            }
                            if (tier.available && tier.reasoningEfforts.isNotEmpty()) {
                                ReasoningEffortChip(
                                    modelName = tier.displayName,
                                    current = cloud.reasoningByTier[tier.tier],
                                    enabled = !cloud.sending,
                                    onClick = { effortTier = tier.tier },
                                )
                            }
                            Box(Modifier.size(18.dp), contentAlignment = Alignment.Center) {
                                if (selected) XIcon(XIcons.Check, size = 17.dp, tint = extra.brandInk)
                            }
                        }
                    }
                }
                Text(
                    "点选模型或右侧思考档位立即生效，无需额外确认；对话页仍可随时切换。",
                    Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp),
                    style = XType.Caption,
                    color = extra.ink3,
                )
            }
        }

        effortTier?.let { tid ->
            val tier = cloud.tiers.firstOrNull { it.tier == tid }
            if (tier != null) {
                ReasoningEffortBottomSheet(
                    modelName = tier.displayName,
                    levels = tier.reasoningEfforts,
                    selected = cloud.reasoningByTier[tier.tier],
                    onDismiss = { effortTier = null },
                    onSelect = { effort ->
                        chat.selectReasoning(tier.tier, effort)
                        effortTier = null
                    },
                )
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

    BottomSheet(onDismiss = onDismiss, panelModifier = panelModifier, flush = true, cornerRadius = 18.dp) {
        SheetDragHandle()
        Row(
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, bottom = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("添加第三方服务", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
            SheetCloseCircle(onDismiss)
        }
        Column(Modifier.fillMaxWidth().heightIn(max = 420.dp).verticalScroll(rememberScrollState())) {
            presets.forEachIndexed { idx, preset ->
                if (idx > 0) Hairline(Modifier.padding(start = 66.dp))
                Row(
                    Modifier.fillMaxWidth().clickable { onSelectPreset(preset) }
                        .padding(horizontal = 16.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Box(
                        Modifier.size(38.dp).clip(RoundedCornerShape(11.dp)).background(extra.fill),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            preset.name.trim().take(1).uppercase().ifBlank { "M" },
                            style = XType.Body,
                            fontWeight = FontWeight.SemiBold,
                            color = scheme.onSurface,
                        )
                    }
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text(preset.name, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            if (preset.added) {
                                Text(
                                    "已添加",
                                    Modifier.clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 6.dp, vertical = 1.dp),
                                    style = XType.Caption,
                                    color = extra.ink3,
                                )
                            }
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
            if (error != null) {
                Row(
                    Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 8.dp)
                        .clip(RoundedCornerShape(10.dp))
                        .background(ModelWarnSoft)
                        .padding(horizontal = 12.dp, vertical = 9.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    XIcon(XIcons.Info, size = 15.dp, tint = ModelWarn)
                    Text(error, Modifier.weight(1f), style = XType.Caption, color = ModelWarn)
                    if (canRetryCatalogue) {
                        Text(
                            "重试",
                            Modifier
                                .clip(RoundedCornerShape(6.dp))
                                .clickable(enabled = !loading, role = Role.Button, onClick = onRetryCatalogue)
                                .padding(horizontal = 6.dp, vertical = 2.dp),
                            style = XType.Caption,
                            fontWeight = FontWeight.SemiBold,
                            color = ModelWarn,
                        )
                    }
                }
            }
        }
        Spacer(Modifier.height(4.dp))
        Box(Modifier.fillMaxWidth().height(6.dp).background(scheme.background))
        Row(
            Modifier.fillMaxWidth().clickable(onClick = onSelectCustom)
                .padding(horizontal = 16.dp, vertical = 14.dp),
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
    var advanced by remember {
        mutableStateOf(
            saved?.contextTokens != null || saved?.maxOutputTokens != null ||
                saved?.supportsTools == false || saved?.supportsVision == false ||
                (saved?.reasoningEfforts != null && saved.reasoningEfforts != DEFAULT_CUSTOM_REASONING_EFFORTS),
        )
    }
    val scope = rememberCoroutineScope()
    var fetching by remember { mutableStateOf(false) }
    var fetchedModels by remember { mutableStateOf<List<String>>(emptyList()) }
    var fetchNote by remember { mutableStateOf<String?>(null) }
    var error by remember { mutableStateOf<String?>(null) }
    var revealKey by remember { mutableStateOf(false) }
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

    fun save() {
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
            chat.saveCustomProvider(config)
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
    val endpointPreview = remember(address, protocol) {
        val trimmed = address.trim().trimEnd('/')
        if (trimmed.isBlank()) null
        else runCatching { "POST " + CustomModelConfig(trimmed, "", "m", protocol = protocol).chatUrl() }.getOrNull()
    }

    Box(Modifier.fillMaxSize().background(scheme.background)) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(
                providerName.trim().ifBlank { if (saved == null) "添加第三方服务" else "编辑模型服务" },
                onBack = onBack,
                trailing = {
                    Text(
                        "保存",
                        Modifier
                            .clip(CircleShape)
                            .clickable(role = Role.Button, onClick = ::save)
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        style = XType.Secondary,
                        fontWeight = FontWeight.SemiBold,
                        color = extra.brandInk,
                    )
                },
            )

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
                            Text(activeProto.name, style = XType.Body, fontWeight = FontWeight.Medium, color = scheme.onSurface)
                            XIcon(XIcons.ChevronRight, size = 16.dp, tint = extra.ink3)
                        }
                    }

                    FieldBlock("API 地址") {
                        ModelFieldBox(
                            value = address,
                            onValueChange = { address = it; error = null; fetchedModels = emptyList(); fetchNote = null },
                            placeholder = activeProto.fallbackBase,
                            keyboardType = KeyboardType.Uri,
                            monospace = true,
                        )
                        val isHttp = address.trim().startsWith("http://", ignoreCase = true)
                        if (isHttp) {
                            Text("HTTP 不加密，密钥和内容会明文传输，仅建议用于可信内网。", style = XType.Caption, color = Danger)
                        } else if (endpointPreview != null) {
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
                                    XIcon(XIcons.Eye, size = 18.dp, tint = if (revealKey) scheme.onSurface else extra.ink3)
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
                Text(
                    "密钥仅保存在本机应用私有存储中，不会上传到小伴账号。",
                    Modifier.padding(start = 16.dp, end = 16.dp, top = 8.dp),
                    style = XType.Caption,
                    color = extra.ink3,
                )

                // 模型列表区
                Row(
                    Modifier.fillMaxWidth().padding(start = 16.dp, end = 12.dp, top = 16.dp, bottom = 7.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    Text("模型 · ${models.size} 个（点选设为默认）", style = XType.Caption, color = extra.ink3)
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
                            Modifier.fillMaxWidth().padding(start = 16.dp, end = 8.dp, top = 10.dp, bottom = 10.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Row(
                                Modifier.weight(1f).clickable(role = Role.RadioButton) { defaultModel = model }.padding(vertical = 2.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(10.dp),
                            ) {
                                Box(
                                    Modifier
                                        .size(18.dp)
                                        .clip(CircleShape)
                                        .border(1.6.dp, if (isDef) Brand else extra.line, CircleShape),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    if (isDef) Box(Modifier.size(9.dp).clip(CircleShape).background(Brand))
                                }
                                Text(model, Modifier.weight(1f, fill = false), style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                if (isDef) {
                                    Text(
                                        "默认",
                                        Modifier.clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 6.dp, vertical = 1.dp),
                                        style = XType.Caption,
                                        color = scheme.onSurfaceVariant,
                                    )
                                }
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
                                .background(if (canAdd) extra.brandSoft else extra.fill)
                                .then(if (canAdd) Modifier.border(1.dp, Brand.copy(alpha = if (ThemeState.dark) 0.36f else 0.24f), RoundedCornerShape(10.dp)) else Modifier)
                                .clickable(enabled = canAdd, role = Role.Button) { addModel(modelDraft) }
                                .padding(horizontal = 16.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("添加", style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = if (canAdd) extra.brandInk else extra.ink3)
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
                                        .background(if (added) ModelSuccessSoft else extra.fill)
                                        .clickable { addModel(item) }
                                        .padding(horizontal = 11.dp, vertical = 6.dp),
                                    style = XType.Caption,
                                    fontWeight = if (added) FontWeight.Medium else FontWeight.Normal,
                                    color = if (added) Success else scheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }

                // 高级设置区（含思考等级多选）
                Spacer(Modifier.height(12.dp))
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
                                Text("支持的思考等级", style = XType.Body, color = scheme.onSurface)
                                Spacer(Modifier.weight(1f))
                                Text(
                                    summarizeReasoningLevels(reasoningEfforts),
                                    style = XType.Caption,
                                    color = scheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                XIcon(XIcons.ChevronRight, size = 15.dp, tint = extra.ink3)
                            }
                        }
                    }
                }

                error?.let {
                    Text(it, Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), color = Danger, style = XType.Secondary)
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

            // 底部柔和主操作栏（去实心大红按钮）
            Column(Modifier.fillMaxWidth().background(scheme.surface).navigationBarsPadding()) {
                Hairline()
                Row(
                    Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Box(
                        Modifier
                            .height(44.dp)
                            .clip(CircleShape)
                            .background(extra.fill)
                            .clickable(role = Role.Button, onClick = onBack)
                            .padding(horizontal = 20.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text("取消", style = XType.Body, fontWeight = FontWeight.Medium, color = scheme.onSurfaceVariant)
                    }
                    SoftAccentPillButton(
                        label = if (saved == null) "保存并使用" else "保存修改",
                        onClick = ::save,
                        modifier = Modifier.weight(1f),
                    )
                }
            }
        }

        // 协议选择 BottomSheet（按协议联动对应预置地址）
        if (showProtoSheet) {
            BottomSheet(onDismiss = { showProtoSheet = false }, flush = true, cornerRadius = 18.dp) {
                SheetDragHandle()
                Row(
                    Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, bottom = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Column(Modifier.weight(1f)) {
                        Text("接口协议", style = XType.Section, fontWeight = FontWeight.SemiBold)
                        Text("切换协议时自动同步对应的预置 API 地址", Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
                    }
                    SheetCloseCircle { showProtoSheet = false }
                }
                ProtocolOptions.forEachIndexed { idx, opt ->
                    if (idx > 0) Hairline(Modifier.padding(start = 16.dp))
                    val on = protocol == opt.id
                    val presetEndpoint = initial?.endpoints?.get(opt.id)?.takeIf(String::isNotBlank) ?: opt.fallbackBase
                    Row(
                        Modifier
                            .fillMaxWidth()
                            .clickable(role = Role.RadioButton) {
                                val oldPreset = initial?.endpoints?.get(protocol)
                                val matchesOldPreset = address.isBlank() || address == oldPreset || address == initial?.baseUrl ||
                                    address == protocolOption(protocol).fallbackBase
                                if (matchesOldPreset) {
                                    initial?.endpoints?.get(opt.id)?.takeIf(String::isNotBlank)?.let { address = it }
                                        ?: if (initial == null) { address = opt.fallbackBase } else Unit
                                }
                                protocol = opt.id
                                error = null
                                fetchedModels = emptyList()
                                fetchNote = null
                                showProtoSheet = false
                            }
                            .padding(horizontal = 16.dp, vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Text(opt.name, style = XType.Body, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Medium)
                            Text(opt.desc, style = XType.Caption, color = extra.ink3)
                            Text(
                                presetEndpoint,
                                fontSize = 11.5.sp,
                                lineHeight = 15.sp,
                                fontFamily = FontFamily.Monospace,
                                color = extra.ink3,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                        Box(Modifier.size(18.dp), contentAlignment = Alignment.Center) {
                            if (on) XIcon(XIcons.Check, size = 17.dp, tint = extra.brandInk)
                        }
                    }
                }
                Spacer(Modifier.height(8.dp))
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

    BottomSheet(onDismiss = onDismiss, flush = true, cornerRadius = 18.dp) {
        SheetDragHandle()
        Row(
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, bottom = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Column(Modifier.weight(1f)) {
                Text(providerName, style = XType.Section, fontWeight = FontWeight.SemiBold)
                Text("服务地址和模型由服务端统一维护", Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
            }
            SheetCloseCircle(onDismiss)
        }
        Column(Modifier.fillMaxWidth().heightIn(max = 360.dp).verticalScroll(rememberScrollState())) {
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
                        if (selected) XIcon(XIcons.Check, size = 17.dp, tint = extra.brandInk)
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
private fun ModelSectionHeader(title: String, trailing: String? = null) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(title, style = XType.Caption, fontWeight = FontWeight.Medium, color = extra.ink3)
        if (!trailing.isNullOrBlank()) {
            Text(trailing, style = XType.Caption, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun OfficialBrandTile() {
    val extra = LocalExtraColors.current
    val shape = RoundedCornerShape(12.dp)
    Box(
        Modifier
            .size(40.dp)
            .clip(shape)
            .background(extra.brandSoft)
            .border(1.dp, Brand.copy(alpha = if (ThemeState.dark) 0.36f else 0.24f), shape),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(XIcons.SparkleSmall, tint = extra.brandInk, size = 20.dp)
    }
}

@Composable
private fun ActiveGreenBadge() {
    Row(
        Modifier.clip(CircleShape).background(ModelSuccessSoft).padding(horizontal = 7.dp, vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Box(Modifier.size(5.dp).clip(CircleShape).background(Success))
        Text("使用中", fontSize = 11.sp, lineHeight = 15.sp, fontWeight = FontWeight.SemiBold, color = Success)
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

/** 克制的冷静绿色开关（不再使用大块亮红开关）。 */
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
