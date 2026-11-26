package xyz.xiangban.xiaoban.common.ui.controls

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.ui.theme.Brand
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon
import xyz.xiangban.xiaoban.common.ui.theme.XIcons
import xyz.xiangban.xiaoban.common.ui.theme.XType

@Composable
fun TextBox(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    label: String? = null,
    placeholder: String = "",
    password: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Text,
    enabled: Boolean = true,
    leadingIcon: ImageVector? = null,
    filled: Boolean = false,
    fontSize: Int = 16,
) {
    Column(modifier) {
        if (label != null) Text(label, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(bottom = 6.dp))
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            enabled = enabled,
            textStyle = TextStyle(fontSize = fontSize.sp, color = MaterialTheme.colorScheme.onSurface),
            keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
            singleLine = true,
            visualTransformation = if (password) PasswordVisualTransformation() else VisualTransformation.None,
            cursorBrush = SolidColor(Brand),
            decorationBox = { inner ->
                Row(
                    Modifier.fillMaxWidth().height(XDimens.InputHeight).clip(RoundedCornerShape(XDimens.RadiusInner))
                        .background(if (filled) LocalExtraColors.current.fill else MaterialTheme.colorScheme.surface)
                        .padding(horizontal = 14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    if (leadingIcon != null) {
                        XIcon(leadingIcon, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = XDimens.IconMedium)
                        Spacer(Modifier.width(10.dp))
                    }
                    Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                        if (value.isEmpty()) Text(placeholder, color = LocalExtraColors.current.ink3, fontSize = fontSize.sp, maxLines = 1)
                        inner()
                    }
                }
            },
        )
    }
}

/** 多行输入框：和 [TextBox] 同样的无描边样式，高度随内容增长，至少 [minHeight]。 */
@Composable
fun AreaBox(value: String, onValueChange: (String) -> Unit, modifier: Modifier = Modifier, placeholder: String = "", minHeight: Dp = 120.dp, enabled: Boolean = true, filled: Boolean = false) {
    BasicTextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier.fillMaxWidth(),
        enabled = enabled,
        textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface),
        cursorBrush = SolidColor(Brand),
        decorationBox = { inner ->
            Box(
                Modifier.fillMaxWidth().heightIn(min = minHeight).clip(RoundedCornerShape(XDimens.RadiusInner))
                    .background(if (filled) LocalExtraColors.current.fill else MaterialTheme.colorScheme.surface)
                    .padding(horizontal = 14.dp, vertical = 12.dp),
            ) {
                if (value.isEmpty()) Text(placeholder, color = LocalExtraColors.current.ink3, style = XType.Body)
                inner()
            }
        },
    )
}

/** 开关：48×28，开启时主题色。 */
@Composable
fun Toggle(checked: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) {
    Box(
        modifier.size(width = 48.dp, height = 28.dp).clip(CircleShape).background(if (checked) Brand else LocalExtraColors.current.line).clickable { onChange(!checked) }.padding(2.dp),
        contentAlignment = if (checked) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Box(Modifier.size(24.dp).clip(CircleShape).background(Color.White))
    }
}

/** 勾选框：22×22 圆角方块。 */
@Composable
fun CheckBox(checked: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) {
    val shape = RoundedCornerShape(XDimens.RadiusInner)
    Box(
        modifier.size(22.dp).clip(shape).background(if (checked) Brand else MaterialTheme.colorScheme.surface)
            .border(2.dp, if (checked) Brand else LocalExtraColors.current.line, shape).clickable { onChange(!checked) },
        contentAlignment = Alignment.Center,
    ) {
        if (checked) XIcon(XIcons.Check, tint = Color.White, size = XDimens.IconSmall)
    }
}

/** 单选行：圆圈、标题和说明（模型选择、思考强度）。 */
@Composable
fun RadioRow(selected: Boolean, title: String, desc: String?, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Row(modifier.fillMaxWidth().alpha(if (enabled) 1f else 0.5f).clickable(enabled = enabled, onClick = onClick).padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.size(20.dp).clip(CircleShape).border(2.dp, if (selected) Brand else LocalExtraColors.current.line, CircleShape), contentAlignment = Alignment.Center) {
            if (selected) Box(Modifier.size(10.dp).clip(CircleShape).background(Brand))
        }
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Body, fontWeight = FontWeight.SemiBold)
            if (desc != null) Text(desc, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
        }
    }
}
