package androidx.compose.runtime
import kotlin.reflect.KProperty
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow
@Target(AnnotationTarget.FUNCTION, AnnotationTarget.TYPE, AnnotationTarget.PROPERTY_GETTER, AnnotationTarget.TYPE_PARAMETER) annotation class Composable
annotation class Immutable
interface State<out T> { val value: T }
interface MutableState<T> : State<T> { override var value: T }
fun <T> mutableStateOf(value: T): MutableState<T> = TODO()
fun <T> derivedStateOf(calculation: () -> T): State<T> = TODO()
operator fun <T> State<T>.getValue(thisObj: Any?, property: KProperty<*>): T = value
operator fun <T> MutableState<T>.setValue(thisObj: Any?, property: KProperty<*>, value: T) { this.value = value }
@Composable fun <T> remember(calculation: () -> T): T = TODO()
@Composable fun <T> remember(key1: Any?, calculation: () -> T): T = TODO()
@Composable fun <T> remember(key1: Any?, key2: Any?, calculation: () -> T): T = TODO()
@Composable fun <T> remember(vararg keys: Any?, calculation: () -> T): T = TODO()
@Composable fun rememberCoroutineScope(): CoroutineScope = TODO()
@Composable fun <T> StateFlow<T>.collectAsState(): State<T> = TODO()
@Composable fun LaunchedEffect(key1: Any?, block: suspend CoroutineScope.() -> Unit) {}
@Composable fun LaunchedEffect(key1: Any?, key2: Any?, block: suspend CoroutineScope.() -> Unit) {}
@Composable fun LaunchedEffect(key1: Any?, key2: Any?, key3: Any?, block: suspend CoroutineScope.() -> Unit) {}
@Composable fun LaunchedEffect(vararg keys: Any?, block: suspend CoroutineScope.() -> Unit) {}
abstract class ProvidableCompositionLocal<T> { val current: T @Composable get() = TODO(); infix fun provides(v: T): Any = TODO() }
fun <T> compositionLocalOf(defaultFactory: () -> T): ProvidableCompositionLocal<T> = TODO()
@Composable fun CompositionLocalProvider(vararg values: Any, content: @Composable () -> Unit) {}
