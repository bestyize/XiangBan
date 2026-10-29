package androidx.compose.ui.text.input
class KeyboardType { companion object { val Text = KeyboardType(); val Number = KeyboardType(); val Email = KeyboardType(); val Password = KeyboardType() } }
interface VisualTransformation { companion object { val None: VisualTransformation = TODO() } }
class PasswordVisualTransformation : VisualTransformation
