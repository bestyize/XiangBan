package androidx.compose.material3
import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.*
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.*
import androidx.compose.ui.unit.*
import androidx.compose.foundation.layout.*
@RequiresOptIn annotation class ExperimentalMaterial3Api
val LocalContentColor = compositionLocalOf { Color(0) }
class ColorScheme(val primary: Color, val onPrimary: Color, val background: Color, val onBackground: Color, val surface: Color, val onSurface: Color, val surfaceVariant: Color, val onSurfaceVariant: Color, val outline: Color, val outlineVariant: Color) { val errorContainer: Color = primary; val primaryContainer: Color = primary }
fun lightColorScheme(primary: Color = Color(0), onPrimary: Color = Color(0), background: Color = Color(0), onBackground: Color = Color(0), surface: Color = Color(0), onSurface: Color = Color(0), surfaceVariant: Color = Color(0), onSurfaceVariant: Color = Color(0), outline: Color = Color(0), outlineVariant: Color = Color(0)) = ColorScheme(primary, onPrimary, background, onBackground, surface, onSurface, surfaceVariant, onSurfaceVariant, outline, outlineVariant)
fun darkColorScheme(primary: Color = Color(0), onPrimary: Color = Color(0), background: Color = Color(0), onBackground: Color = Color(0), surface: Color = Color(0), onSurface: Color = Color(0), surfaceVariant: Color = Color(0), onSurfaceVariant: Color = Color(0), outline: Color = Color(0), outlineVariant: Color = Color(0)) = lightColorScheme(primary, onPrimary, background, onBackground, surface, onSurface, surfaceVariant, onSurfaceVariant, outline, outlineVariant)
object MaterialTheme { val colorScheme: ColorScheme @Composable get() = TODO() }
@Composable fun MaterialTheme(colorScheme: ColorScheme, content: @Composable () -> Unit) {}
@Composable fun Text(text: String, modifier: Modifier = Modifier, color: Color = Color(0), fontSize: TextUnit = TextUnit(0f), fontWeight: FontWeight? = null, textAlign: TextAlign? = null, lineHeight: TextUnit = TextUnit(0f), overflow: TextOverflow = TextOverflow.Clip, maxLines: Int = Int.MAX_VALUE) {}
@Composable fun Icon(imageVector: ImageVector, contentDescription: String?, modifier: Modifier = Modifier, tint: Color = Color(0)) {}
@Composable fun HorizontalDivider(modifier: Modifier = Modifier, thickness: Dp = Dp(1f), color: Color = Color(0)) {}
@Composable fun CircularProgressIndicator(modifier: Modifier = Modifier, color: Color = Color(0), strokeWidth: Dp = Dp(1f)) {}
@Composable fun Button(onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, shape: Shape? = null, content: @Composable RowScope.() -> Unit) {}
@Composable fun OutlinedButton(onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, shape: Shape? = null, content: @Composable RowScope.() -> Unit) {}
@Composable fun TextButton(onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, content: @Composable RowScope.() -> Unit) {}
