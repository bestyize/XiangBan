package androidx.compose.foundation.shape
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Dp
class RoundedCornerShape(topStart: Dp = Dp(0f), topEnd: Dp = Dp(0f), bottomEnd: Dp = Dp(0f), bottomStart: Dp = Dp(0f)) : Shape { constructor(all: Dp) : this(all, all, all, all) }
val CircleShape: Shape = TODO()
