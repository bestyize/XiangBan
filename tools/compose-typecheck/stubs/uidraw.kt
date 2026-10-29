package androidx.compose.ui.draw
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Dp
fun Modifier.clip(shape: Shape): Modifier = this
fun Modifier.rotate(degrees: Float): Modifier = this
fun Modifier.shadow(elevation: Dp, shape: Shape? = null, clip: Boolean = true): Modifier = this
fun Modifier.alpha(alpha: Float): Modifier = this
