package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiaban.wallpaper.companion.Expression

/** Empty, loading and error screens: 小伴 says what is going on. */
@Composable
fun Center(expression: Expression, message: String, action: String? = null, onAction: () -> Unit = {}) {
    Column(
        Modifier.fillMaxSize().padding(horizontal = 32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        PetFace(expression, Modifier.width(150.dp))
        Spacer(Modifier.height(12.dp))
        Text(message, color = Palette.Sub, fontSize = 14.sp, textAlign = TextAlign.Center, lineHeight = 20.sp)
        if (action != null) {
            Spacer(Modifier.height(16.dp))
            Text(
                action,
                Modifier
                    .clip(RoundedCornerShape(18.dp))
                    .border(1.dp, Palette.Red, RoundedCornerShape(18.dp))
                    .clickable(onClick = onAction)
                    .padding(horizontal = 24.dp, vertical = 8.dp),
                color = Palette.Red, fontSize = 14.sp,
            )
        }
    }
}
