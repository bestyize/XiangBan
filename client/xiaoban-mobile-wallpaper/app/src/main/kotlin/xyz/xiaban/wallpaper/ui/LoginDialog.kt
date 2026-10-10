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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.data.AccountRules

/**
 * The classic sign-in dialog: a card centred over a dimmed page, a close button in the corner, an email field,
 * a code field with a "get code" button, and one red button. An email with no account yet gets one on first sign-in.
 */
@Composable
fun LoginDialog(app: AppViewModel, onDismiss: () -> Unit) {
    val scope = rememberCoroutineScope()
    var email by rememberSaveable { mutableStateOf("") }
    var code by rememberSaveable { mutableStateOf("") }
    var cooldown by remember { mutableIntStateOf(0) }
    var sending by remember { mutableStateOf(false) }
    var submitting by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var info by remember { mutableStateOf<String?>(null) }
    val codeFocus = remember { FocusRequester() }

    LaunchedEffect(cooldown > 0) {
        while (cooldown > 0) {
            delay(1000)
            cooldown--
        }
    }

    val emailOk = AccountRules.isEmail(email)
    val canLogin = emailOk && code.length == AccountRules.CODE_LENGTH && !submitting
    val canSend = emailOk && cooldown == 0 && !sending && !submitting

    fun sendCode() {
        if (!canSend) return
        error = null
        info = null
        sending = true
        scope.launch {
            try {
                cooldown = app.sendLoginCode(email.trim())
                info = "验证码已发送，请查收邮件"
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

    fun login() {
        if (!canLogin) return
        error = null
        submitting = true
        scope.launch {
            try {
                app.login(email.trim(), code)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = e.message ?: "登录失败"
            } finally {
                submitting = false
            }
        }
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
            Column(Modifier.fillMaxWidth().padding(start = 24.dp, end = 24.dp, top = 28.dp, bottom = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                PetFace(Expression.HAPPY, Modifier.widthIn(max = 84.dp).fillMaxWidth())
                Spacer(Modifier.height(8.dp))
                Text("登录 / 注册", color = Palette.Text, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                Spacer(Modifier.height(6.dp))
                Text("使用邮箱验证码登录，没有账号会自动创建", color = Palette.Sub, fontSize = 13.sp, textAlign = TextAlign.Center)
                Spacer(Modifier.height(22.dp))

                LoginField(
                    value = email,
                    onValueChange = { email = it.trim(); error = null },
                    placeholder = "邮箱地址",
                    keyboardType = KeyboardType.Email,
                    imeAction = ImeAction.Next,
                    onImeAction = { codeFocus.requestFocus() },
                    enabled = !submitting,
                )
                Spacer(Modifier.height(12.dp))
                LoginField(
                    value = code,
                    onValueChange = { code = it.filter(Char::isDigit).take(AccountRules.CODE_LENGTH); error = null },
                    placeholder = "验证码",
                    keyboardType = KeyboardType.NumberPassword,
                    imeAction = ImeAction.Done,
                    onImeAction = ::login,
                    enabled = !submitting,
                    modifier = Modifier.focusRequester(codeFocus),
                    trailing = {
                        val label = when {
                            sending -> "发送中…"
                            cooldown > 0 -> "重新发送（${cooldown}s）"
                            else -> "获取验证码"
                        }
                        Text(
                            label,
                            Modifier.clickable(enabled = canSend, interactionSource = remember { MutableInteractionSource() }, indication = null, onClick = ::sendCode)
                                .padding(horizontal = 14.dp, vertical = 12.dp),
                            color = if (canSend) Palette.Brand else Palette.Hint,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Medium,
                            maxLines = 1,
                        )
                    },
                )

                Box(Modifier.fillMaxWidth().height(34.dp).padding(top = 8.dp), contentAlignment = Alignment.CenterStart) {
                    val shown = error ?: info
                    if (shown != null) Text(shown, color = if (error != null) Palette.Brand else Palette.Sub, fontSize = 12.sp, maxLines = 2)
                }

                Box(
                    Modifier.fillMaxWidth().height(48.dp).clip(CircleShape)
                        .background(if (canLogin) Palette.Brand else Palette.Brand.copy(alpha = 0.4f))
                        .clickable(enabled = canLogin, onClick = ::login),
                    contentAlignment = Alignment.Center,
                ) {
                    if (submitting) {
                        CircularProgressIndicator(Modifier.size(22.dp), color = Color.White, strokeWidth = 2.5.dp)
                    } else {
                        Text("登录", color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.Bold)
                    }
                }
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
