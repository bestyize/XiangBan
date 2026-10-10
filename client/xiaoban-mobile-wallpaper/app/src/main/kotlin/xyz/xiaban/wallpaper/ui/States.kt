package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.background
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiaban.wallpaper.companion.Expression

/** Empty, loading and error screens: 小伴 says what is going on. */
@Composable
fun Center(expression: Expression, message: String, action: String? = null, onAction: () -> Unit = {}) {
    Column(
        Modifier.fillMaxSize().padding(start = 32.dp, end = 32.dp, bottom = tabBarClearance()),
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
                    .clip(RoundedCornerShape(20.dp))
                    .background(Palette.Brand)
                    .clickable(onClick = onAction)
                    .padding(horizontal = 28.dp, vertical = 10.dp),
                color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Medium,
            )
        }
    }
}
