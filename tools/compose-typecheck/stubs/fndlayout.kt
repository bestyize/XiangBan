package androidx.compose.foundation.layout
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
interface ColumnScope { fun Modifier.weight(weight: Float, fill: Boolean = true): Modifier = this; fun Modifier.align(alignment: Alignment.Horizontal): Modifier = this }
interface RowScope { fun Modifier.weight(weight: Float, fill: Boolean = true): Modifier = this }
interface BoxScope { fun Modifier.align(alignment: Alignment): Modifier = this }
object Arrangement { interface Vertical; interface Horizontal; interface HorizontalOrVertical : Vertical, Horizontal
  val Center: HorizontalOrVertical = TODO(); val Bottom: Vertical = TODO(); val SpaceBetween: HorizontalOrVertical = TODO(); val End: Horizontal = TODO(); val Start: Horizontal = TODO()
  fun spacedBy(space: Dp): HorizontalOrVertical = TODO()
  fun spacedBy(space: Dp, alignment: Alignment.Horizontal): Horizontal = TODO()
  fun spacedBy(space: Dp, alignment: Alignment.Vertical): Vertical = TODO() }
@Composable fun Column(modifier: Modifier = Modifier, verticalArrangement: Arrangement.Vertical = Arrangement.Center, horizontalAlignment: Alignment.Horizontal = Alignment.Start, content: @Composable ColumnScope.() -> Unit) {}
@Composable fun Row(modifier: Modifier = Modifier, horizontalArrangement: Arrangement.Horizontal = Arrangement.Start, verticalAlignment: Alignment.Vertical = Alignment.Top, content: @Composable RowScope.() -> Unit) {}
@Composable fun Box(modifier: Modifier = Modifier, contentAlignment: Alignment = Alignment.TopStart, content: @Composable BoxScope.() -> Unit = {}) {}
@Composable fun Spacer(modifier: Modifier) {}
class PaddingValues
fun PaddingValues(horizontal: Dp = Dp(0f), vertical: Dp = Dp(0f)): PaddingValues = PaddingValues()
fun PaddingValues(all: Dp): PaddingValues = PaddingValues()
fun Modifier.padding(all: Dp): Modifier = this
fun Modifier.padding(horizontal: Dp = Dp(0f), vertical: Dp = Dp(0f)): Modifier = this
fun Modifier.padding(start: Dp = Dp(0f), top: Dp = Dp(0f), end: Dp = Dp(0f), bottom: Dp = Dp(0f)): Modifier = this
fun Modifier.size(size: Dp): Modifier = this
fun Modifier.size(width: Dp, height: Dp): Modifier = this
fun Modifier.width(w: Dp): Modifier = this
fun Modifier.wrapContentWidth(align: Alignment.Horizontal = Alignment.CenterHorizontally, unbounded: Boolean = false): Modifier = this
fun Modifier.height(h: Dp): Modifier = this
fun Modifier.widthIn(min: Dp = Dp(0f), max: Dp = Dp(0f)): Modifier = this
fun Modifier.fillMaxSize(fraction: Float = 1f): Modifier = this
fun Modifier.fillMaxWidth(fraction: Float = 1f): Modifier = this
fun Modifier.fillMaxHeight(fraction: Float = 1f): Modifier = this
fun Modifier.aspectRatio(ratio: Float): Modifier = this
fun Modifier.statusBarsPadding(): Modifier = this
fun Modifier.navigationBarsPadding(): Modifier = this
fun Modifier.imePadding(): Modifier = this

fun Modifier.heightIn(min: Dp = Dp(0f), max: Dp = Dp(0f)): Modifier = this
