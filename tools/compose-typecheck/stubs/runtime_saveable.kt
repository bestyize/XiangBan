package androidx.compose.runtime.saveable
import androidx.compose.runtime.*
@Composable fun <T : Any> rememberSaveable(vararg inputs: Any?, init: () -> MutableState<T>): MutableState<T> = TODO()
@Composable fun <T> rememberSaveable(vararg inputs: Any?, init: () -> T): T = TODO()
