package androidx.compose.ui.text
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.TextUnit
class TextStyle(val color: Color = Color.Black, val fontSize: TextUnit = TextUnit(0f), val lineHeight: TextUnit = TextUnit(0f))
class SpanStyle(val color: Color = Color.Black, val fontSize: TextUnit = TextUnit(0f), val fontWeight: androidx.compose.ui.text.font.FontWeight? = null, val fontStyle: androidx.compose.ui.text.font.FontStyle? = null, val fontFamily: androidx.compose.ui.text.font.FontFamily? = null, val background: Color = Color.Black, val textDecoration: androidx.compose.ui.text.style.TextDecoration? = null)
class TextLinkStyles(val style: SpanStyle? = null)
fun interface LinkInteractionListener { fun onClick(link: LinkAnnotation) }
sealed class LinkAnnotation {
    class Clickable(val tag: String, val styles: TextLinkStyles? = null, val linkInteractionListener: LinkInteractionListener? = null) : LinkAnnotation()
}
class AnnotatedString(val text: String) {
    class Builder {
        fun append(text: String) {}
    }
}
fun buildAnnotatedString(builder: AnnotatedString.Builder.() -> Unit): AnnotatedString = AnnotatedString("")
fun <R> AnnotatedString.Builder.withStyle(style: SpanStyle, block: AnnotatedString.Builder.() -> R): R = block()
fun <R> AnnotatedString.Builder.withLink(link: LinkAnnotation, block: AnnotatedString.Builder.() -> R): R = block()
