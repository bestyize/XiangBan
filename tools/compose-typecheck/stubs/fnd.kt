package androidx.compose.foundation
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.*
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.Dp
fun Modifier.background(color: Color, shape: Shape? = null): Modifier = this
fun Modifier.background(brush: Brush, shape: Shape? = null): Modifier = this
fun Modifier.border(width: Dp, color: Color, shape: Shape? = null): Modifier = this
fun Modifier.clickable(enabled: Boolean = true, onClick: () -> Unit): Modifier = this
fun Modifier.clickable(indication: Any?, interactionSource: Any?, enabled: Boolean = true, onClick: () -> Unit): Modifier = this
class ScrollState
@Composable fun rememberScrollState(): ScrollState = TODO()
fun Modifier.verticalScroll(state: ScrollState): Modifier = this
fun Modifier.horizontalScroll(state: ScrollState): Modifier = this
@Composable fun isSystemInDarkTheme(): Boolean = false
