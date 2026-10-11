package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.data.AccountException
import xyz.xiaban.wallpaper.data.AccountRules
import xyz.xiaban.wallpaper.data.CodeLoginResult

private enum class LoginMode { PASSWORD, CODE, RESET }

private enum class LoginStep { IDENTIFY, VERIFY, SETUP }

/** What the live username check last told us. */
private sealed interface Availability {
    data object Idle : Availability
    data object Checking : Availability
    data object Ok : Availability
    data class Problem(val message: String) : Availability
}

/**
 * Sign in or register. An existing account signs in with a username, email or phone number plus a password. A new
 * person verifies an email with a code and then picks a unique, unchangeable username and a password. A forgotten
 * password is reset with a code.
 */
@Composable
fun LoginDialog(app: AppViewModel, onDismiss: () -> Unit) {
    val scope = rememberCoroutineScope()
    var mode by rememberSaveable { mutableStateOf(LoginMode.PASSWORD) }
    var step by rememberSaveable { mutableStateOf(LoginStep.IDENTIFY) }
    var identifier by rememberSaveable { mutableStateOf("") }
    var password by rememberSaveable { mutableStateOf("") }
    var code by rememberSaveable { mutableStateOf("") }
    var username by rememberSaveable { mutableStateOf("") }
    var registrationToken by rememberSaveable { mutableStateOf<String?>(null) }
    var showPassword by remember { mutableStateOf(false) }
    var cooldown by remember { mutableIntStateOf(0) }
    var sending by remember { mutableStateOf(false) }
    var submitting by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var info by remember { mutableStateOf<String?>(null) }
    var availability by remember { mutableStateOf<Availability>(Availability.Idle) }
    val codeFocus = remember { FocusRequester() }

    LaunchedEffect(cooldown > 0) {
        while (cooldown > 0) {
            delay(1000)
            cooldown--
        }
    }

    // While choosing a username, check it as the user types. It is only a hint: the server enforces uniqueness.
    LaunchedEffect(username, step) {
        if (step != LoginStep.SETUP) return@LaunchedEffect
        val name = username.trim()
        val local = AccountRules.usernameError(name)
        availability = when {
            name.isEmpty() -> Availability.Idle
            local != null -> Availability.Problem(local)
            else -> Availability.Checking
        }
        if (name.isEmpty() || local != null) return@LaunchedEffect
        delay(350)
        availability = try {
            val check = app.checkUsername(name)
            if (check.available) Availability.Ok else Availability.Problem(check.message ?: "这个用户名已被占用")
        } catch (e: CancellationException) {
            throw e
        } catch (_: Exception) {
            Availability.Idle
        }
    }

    fun switchMode(next: LoginMode) {
        mode = next
        step = LoginStep.IDENTIFY
        code = ""
        password = ""
        error = null
        info = null
        registrationToken = null
        cooldown = 0
    }

    val identified = identifier.trim().isNotEmpty()
    val codeSent = step == LoginStep.VERIFY
    val passwordProblem = AccountRules.passwordError(password, if (mode == LoginMode.RESET) "" else username)
    val canSend = identified && cooldown == 0 && !sending && !submitting
    val canSubmit = !submitting && when {
        mode == LoginMode.PASSWORD -> identified && password.isNotEmpty()
        mode == LoginMode.RESET -> codeSent && code.length == AccountRules.CODE_LENGTH && password.isNotEmpty() && passwordProblem == null
        step == LoginStep.SETUP -> username.isNotBlank() && availability is Availability.Ok && password.isNotEmpty() && passwordProblem == null
        else -> codeSent && code.length == AccountRules.CODE_LENGTH
    }

    fun sendCode() {
        if (!canSend) return
        error = null
        info = null
        sending = true
        scope.launch {
            try {
                cooldown = app.sendLoginCode(identifier.trim(), resetPassword = mode == LoginMode.RESET)
                step = LoginStep.VERIFY
                info = "验证码已发送，请查收"
                codeFocus.requestFocus()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = e.message ?: "验证码发送失败"
            } finally {
                sending = false
            }
        }
    }

    fun submit() {
        if (!canSubmit) return
        error = null
        submitting = true
        scope.launch {
            try {
                when {
                    mode == LoginMode.PASSWORD -> app.loginWithPassword(identifier.trim(), password)
                    mode == LoginMode.RESET -> {
                        app.resetPassword(identifier.trim(), code, password)
                        switchMode(LoginMode.PASSWORD)
                        info = "密码已重置，请用新密码登录"
                    }
                    step == LoginStep.SETUP -> {
                        val token = registrationToken ?: throw AccountException("验证已过期，请重新获取验证码", code = "registration_expired")
                        app.register(token, username.trim(), password)
                    }
                    else -> when (val result = app.loginWithCode(identifier.trim(), code)) {
                        is CodeLoginResult.RegistrationRequired -> {
                            registrationToken = result.token
                            password = ""
                            step = LoginStep.SETUP
                        }
                        is CodeLoginResult.SignedIn -> Unit
                    }
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                val coded = (e as? AccountException)?.code
                if (coded == "username_taken") availability = Availability.Problem(e.message ?: "这个用户名已被占用")
                if (coded == "registration_expired" || coded == "identity_exists") {
                    registrationToken = null
                    step = LoginStep.IDENTIFY
                    code = ""
                }
                error = e.message ?: "登录失败"
            } finally {
                submitting = false
            }
        }
    }

    val submitLabel = when {
        mode == LoginMode.PASSWORD -> "登录"
        mode == LoginMode.RESET -> "重置密码"
        step == LoginStep.SETUP -> "完成注册"
        else -> "验证并继续"
    }

    Dialog(
        onDismissRequest = { if (!submitting) onDismiss() },
        properties = DialogProperties(dismissOnBackPress = !submitting, dismissOnClickOutside = !submitting, usePlatformDefaultWidth = false),
    ) {
        Box(
            Modifier.padding(horizontal = 28.dp).widthIn(max = 380.dp).fillMaxWidth()
                .clip(RoundedCornerShape(24.dp)).background(Palette.Card),
        ) {
            Box(
                Modifier.align(Alignment.TopEnd).padding(8.dp).size(40.dp).clip(CircleShape)
                    .clickable(enabled = !submitting, onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) {
                Icon(Icons.Default.Close, "关闭", Modifier.size(20.dp), tint = Palette.Sub)
            }
            Column(
                Modifier.fillMaxWidth().padding(start = 24.dp, end = 24.dp, top = 28.dp, bottom = 24.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                PetFace(Expression.HAPPY, Modifier.widthIn(max = 84.dp).fillMaxWidth())
                Spacer(Modifier.height(8.dp))
                Text("登录 / 注册", color = Palette.Text, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                Spacer(Modifier.height(6.dp))
                Text(
                    if (step == LoginStep.SETUP) "验证通过，设置你的账号。用户名是你的唯一账号，设置后不能修改。"
                    else "已有账号用密码登录；新用户用邮箱验证码注册，再设置用户名和密码",
                    color = Palette.Sub, fontSize = 13.sp, textAlign = TextAlign.Center,
                )
                Spacer(Modifier.height(18.dp))

                if (step != LoginStep.SETUP) {
                    ModeTabs(mode != LoginMode.CODE, enabled = !submitting, onPassword = { switchMode(LoginMode.PASSWORD) }, onCode = { switchMode(LoginMode.CODE) })
                    Spacer(Modifier.height(14.dp))
                }

                if (step == LoginStep.SETUP) {
                    LoginField(
                        value = username,
                        onValueChange = { username = it.trim().take(AccountRules.USERNAME_MAX); error = null },
                        placeholder = "用户名（字母开头，4-20 位字母、数字或下划线）",
                        keyboardType = KeyboardType.Ascii,
                        imeAction = ImeAction.Next,
                        onImeAction = {},
                        enabled = !submitting,
                    )
                    val hint = when (val a = availability) {
                        Availability.Checking -> "正在检查…" to false
                        Availability.Ok -> "可以使用" to false
                        is Availability.Problem -> a.message to true
                        Availability.Idle -> null
                    }
                    if (hint != null) {
                        Text(hint.first, Modifier.fillMaxWidth().padding(top = 4.dp, start = 4.dp), color = if (hint.second) Palette.Brand else Palette.Sub, fontSize = 12.sp)
                    }
                } else {
                    LoginField(
                        value = identifier,
                        onValueChange = { identifier = it.trim(); error = null },
                        placeholder = if (mode == LoginMode.PASSWORD) "用户名、邮箱或手机号" else "邮箱或手机号",
                        keyboardType = KeyboardType.Email,
                        imeAction = if (mode == LoginMode.PASSWORD) ImeAction.Next else ImeAction.Done,
                        onImeAction = { if (mode != LoginMode.PASSWORD) sendCode() },
                        enabled = !submitting && !codeSent,
                    )
                }

                if (mode == LoginMode.PASSWORD || step == LoginStep.SETUP || (mode == LoginMode.RESET && codeSent)) {
                    Spacer(Modifier.height(12.dp))
                    LoginField(
                        value = password,
                        onValueChange = { password = it.take(AccountRules.PASSWORD_MAX); error = null },
                        placeholder = if (mode == LoginMode.RESET) "新密码（至少 8 位）" else if (step == LoginStep.SETUP) "设置密码（至少 8 位）" else "密码",
                        keyboardType = KeyboardType.Password,
                        imeAction = ImeAction.Done,
                        onImeAction = ::submit,
                        enabled = !submitting,
                        visualTransformation = if (showPassword) VisualTransformation.None else PasswordVisualTransformation(),
                        trailing = {
                            Text(
                                if (showPassword) "隐藏" else "显示",
                                Modifier.clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { showPassword = !showPassword }
                                    .padding(horizontal = 14.dp, vertical = 12.dp),
                                color = Palette.Brand, fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 1,
                            )
                        },
                    )
                    if (mode != LoginMode.PASSWORD && passwordProblem != null) {
                        Text(passwordProblem, Modifier.fillMaxWidth().padding(top = 4.dp, start = 4.dp), color = Palette.Brand, fontSize = 12.sp)
                    }
                }

                if (mode != LoginMode.PASSWORD && codeSent) {
                    Spacer(Modifier.height(12.dp))
                    LoginField(
                        value = code,
                        onValueChange = { code = it.filter(Char::isDigit).take(AccountRules.CODE_LENGTH); error = null },
                        placeholder = "验证码",
                        keyboardType = KeyboardType.NumberPassword,
                        imeAction = ImeAction.Done,
                        onImeAction = ::submit,
                        enabled = !submitting,
                        modifier = Modifier.focusRequester(codeFocus),
                    )
                }

                if (mode != LoginMode.PASSWORD && step != LoginStep.SETUP) {
                    Spacer(Modifier.height(8.dp))
                    val label = when {
                        sending -> "发送中…"
                        cooldown > 0 -> "重新发送（${cooldown}s）"
                        codeSent -> "重新发送"
                        else -> "获取验证码"
                    }
                    Text(
                        label,
                        Modifier.fillMaxWidth().clickable(enabled = canSend, interactionSource = remember { MutableInteractionSource() }, indication = null, onClick = ::sendCode)
                            .padding(vertical = 8.dp),
                        color = if (canSend) Palette.Brand else Palette.Hint,
                        fontSize = 14.sp, fontWeight = FontWeight.Medium, textAlign = TextAlign.Center,
                    )
                }

                Box(Modifier.fillMaxWidth().height(34.dp).padding(top = 8.dp), contentAlignment = Alignment.CenterStart) {
                    val shown = error ?: info
                    if (shown != null) Text(shown, color = if (error != null) Palette.Brand else Palette.Sub, fontSize = 12.sp, maxLines = 2)
                }

                if (mode == LoginMode.PASSWORD || codeSent || step == LoginStep.SETUP) {
                    Box(
                        Modifier.fillMaxWidth().height(48.dp).clip(CircleShape)
                            .background(if (canSubmit) Palette.Brand else Palette.Brand.copy(alpha = 0.4f))
                            .clickable(enabled = canSubmit, onClick = ::submit),
                        contentAlignment = Alignment.Center,
                    ) {
                        if (submitting) {
                            CircularProgressIndicator(Modifier.size(22.dp), color = Color.White, strokeWidth = 2.5.dp)
                        } else {
                            Text(submitLabel, color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                }

                if (mode == LoginMode.PASSWORD) {
                    Text(
                        "忘记密码？",
                        Modifier.padding(top = 10.dp).clickable(enabled = !submitting, interactionSource = remember { MutableInteractionSource() }, indication = null) { switchMode(LoginMode.RESET) },
                        color = Palette.Sub, fontSize = 13.sp,
                    )
                }
                if (mode == LoginMode.RESET) {
                    Text(
                        "返回密码登录",
                        Modifier.padding(top = 10.dp).clickable(enabled = !submitting, interactionSource = remember { MutableInteractionSource() }, indication = null) { switchMode(LoginMode.PASSWORD) },
                        color = Palette.Sub, fontSize = 13.sp,
                    )
                }
            }
        }
    }
}

/** Two pills: sign in with a password, or use an emailed code (which is also how a new person registers). */
@Composable
private fun ModeTabs(passwordSelected: Boolean, enabled: Boolean, onPassword: () -> Unit, onCode: () -> Unit) {
    Row(Modifier.fillMaxWidth().clip(CircleShape).background(Palette.Field).padding(4.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        listOf("密码登录" to passwordSelected, "验证码登录 / 注册" to !passwordSelected).forEachIndexed { index, (label, selected) ->
            Box(
                Modifier.weight(1f).height(36.dp).clip(CircleShape)
                    .background(if (selected) Palette.Brand else Color.Transparent)
                    .clickable(enabled = enabled && !selected) { if (index == 0) onPassword() else onCode() },
                contentAlignment = Alignment.Center,
            ) {
                Text(label, color = if (selected) Color.White else Palette.Sub, fontSize = 13.sp, fontWeight = FontWeight.Medium, maxLines = 1)
            }
        }
    }
}

/** A rounded grey input with a placeholder and an optional button at its right end. */
@Composable
private fun LoginField(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    keyboardType: KeyboardType,
    imeAction: ImeAction,
    onImeAction: () -> Unit,
    enabled: Boolean,
    modifier: Modifier = Modifier,
    visualTransformation: VisualTransformation = VisualTransformation.None,
    trailing: (@Composable () -> Unit)? = null,
) {
    Row(
        Modifier.fillMaxWidth().height(48.dp).clip(RoundedCornerShape(14.dp)).background(Palette.Field).padding(start = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = modifier.weight(1f),
            enabled = enabled,
            singleLine = true,
            textStyle = TextStyle(color = Palette.Text, fontSize = 15.sp),
            cursorBrush = SolidColor(Palette.Brand),
            visualTransformation = visualTransformation,
            keyboardOptions = KeyboardOptions(keyboardType = keyboardType, imeAction = imeAction),
            keyboardActions = KeyboardActions(onNext = { onImeAction() }, onDone = { onImeAction() }),
            decorationBox = { inner ->
                Box(contentAlignment = Alignment.CenterStart) {
                    if (value.isEmpty()) Text(placeholder, color = Palette.Hint, fontSize = 15.sp)
                    inner()
                }
            },
        )
        trailing?.invoke()
    }
}
