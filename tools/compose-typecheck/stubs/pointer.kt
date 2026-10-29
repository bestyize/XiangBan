package androidx.compose.ui.input.pointer
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.unit.Dp
class PointerId(val value: Long)
class PointerInputChange(val id: PointerId, val position: Offset, val pressed: Boolean) { fun consume() {} }
class PointerEvent(val changes: List<PointerInputChange>)
class IntSizeStub(val width: Int, val height: Int)
interface PointerInputScope { val size: IntSizeStub; fun Dp.toPx(): Float }
interface AwaitPointerEventScope { suspend fun awaitPointerEvent(): PointerEvent }
fun Modifier.pointerInput(key1: Any?, block: suspend PointerInputScope.() -> Unit): Modifier = this
fun Modifier.pointerInput(key1: Any?, key2: Any?, block: suspend PointerInputScope.() -> Unit): Modifier = this
