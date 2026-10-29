package androidx.compose.foundation.lazy
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.foundation.layout.*
class LazyListState { suspend fun scrollToItem(index: Int) {} }
@Composable fun rememberLazyListState(): LazyListState = TODO()
interface LazyListScope { fun item(key: Any? = null, content: @Composable () -> Unit); fun <T> items(items: List<T>, key: ((T) -> Any)? = null, itemContent: @Composable (T) -> Unit) }
fun <T> LazyListScope.items(items: List<T>, key: ((T) -> Any)? = null, itemContent: @Composable (T) -> Unit) = this.items(items, key, itemContent)
@Composable fun LazyColumn(modifier: Modifier = Modifier, state: LazyListState = LazyListState(), contentPadding: PaddingValues = PaddingValues(), verticalArrangement: Arrangement.Vertical = Arrangement.Center, content: LazyListScope.() -> Unit) {}
@Composable fun LazyRow(modifier: Modifier = Modifier, contentPadding: PaddingValues = PaddingValues(), horizontalArrangement: Arrangement.Horizontal = Arrangement.Start, verticalAlignment: Alignment.Vertical = Alignment.Top, content: LazyListScope.() -> Unit) {}
