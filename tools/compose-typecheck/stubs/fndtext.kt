package androidx.compose.foundation.text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.*
class KeyboardOptions(val keyboardType: KeyboardType = KeyboardType.Text)
@Composable fun BasicTextField(value: String, onValueChange: (String) -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, readOnly: Boolean = false, textStyle: TextStyle = TextStyle(), keyboardOptions: KeyboardOptions = KeyboardOptions(), singleLine: Boolean = false, maxLines: Int = Int.MAX_VALUE, minLines: Int = 1, visualTransformation: VisualTransformation = VisualTransformation.None, cursorBrush: Brush = Brush(), decorationBox: @Composable (innerTextField: @Composable () -> Unit) -> Unit = { it() }) {}
