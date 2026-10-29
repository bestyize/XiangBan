package androidx.compose.ui
import androidx.compose.runtime.Composable
interface Modifier { companion object : Modifier; infix fun then(other: Modifier): Modifier = this }
interface Alignment { interface Horizontal : Alignment; interface Vertical : Alignment
  companion object { val Center: Alignment = TODO(); val TopStart: Alignment = TODO(); val BottomCenter: Alignment = TODO(); val CenterStart: Alignment = TODO(); val CenterEnd: Alignment = TODO(); val BottomEnd: Alignment = TODO(); val TopCenter: Alignment = TODO(); val TopEnd: Alignment = TODO()
    val CenterHorizontally: Horizontal = TODO(); val Start: Horizontal = TODO(); val End: Horizontal = TODO(); val CenterVertically: Vertical = TODO(); val Top: Vertical = TODO(); val Bottom: Vertical = TODO() } }
@RequiresOptIn annotation class ExperimentalComposeUiApi
