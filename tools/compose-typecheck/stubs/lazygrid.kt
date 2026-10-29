package androidx.compose.foundation.lazy.grid
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.foundation.layout.*
class GridCells { class Fixed(n: Int) }
class GridItemSpan(n: Int)
interface LazyGridItemSpanScope { val maxLineSpan: Int }
interface LazyGridScope { fun item(key: Any? = null, span: (LazyGridItemSpanScope.() -> GridItemSpan)? = null, content: @Composable () -> Unit); fun <T> items(items: List<T>, key: ((T) -> Any)? = null, span: (LazyGridItemSpanScope.(T) -> GridItemSpan)? = null, itemContent: @Composable (T) -> Unit) }

@Composable fun LazyVerticalGrid(columns: GridCells.Fixed, modifier: Modifier = Modifier, contentPadding: PaddingValues = PaddingValues(), horizontalArrangement: Arrangement.Horizontal = Arrangement.Start, verticalArrangement: Arrangement.Vertical = Arrangement.Center, content: LazyGridScope.() -> Unit) {}

fun <T> LazyGridScope.items(items: List<T>, key: ((T) -> Any)? = null, span: (LazyGridItemSpanScope.(T) -> GridItemSpan)? = null, itemContent: @Composable (T) -> Unit) = this.items(items, key, span, itemContent)
