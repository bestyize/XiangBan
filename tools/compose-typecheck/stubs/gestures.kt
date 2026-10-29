package androidx.compose.foundation.gestures
import androidx.compose.foundation.lazy.LazyListState
// 真正的 scrollBy 是 ScrollableState 的扩展函数，要单独 import；这里照着做，漏了 import 类型检查就会报错。
suspend fun LazyListState.scrollBy(value: Float): Float = 0f
