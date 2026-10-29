package androidx.compose.foundation.gestures
import androidx.compose.foundation.lazy.LazyListState
// 真正的 scrollBy 是 ScrollableState 的扩展函数，要单独 import；这里照着做，漏了 import 类型检查就会报错。
suspend fun LazyListState.scrollBy(value: Float): Float = 0f

// 按住说话用到的指针输入（真实签名：androidx.compose.ui.input.pointer / foundation.gestures）
suspend fun androidx.compose.ui.input.pointer.PointerInputScope.awaitEachGesture(block: suspend androidx.compose.ui.input.pointer.AwaitPointerEventScope.() -> Unit) {}
suspend fun androidx.compose.ui.input.pointer.AwaitPointerEventScope.awaitFirstDown(requireUnconsumed: Boolean = true): androidx.compose.ui.input.pointer.PointerInputChange = TODO()
