package xyz.xiangban.xiaoban.common.share

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathBuilder
import androidx.compose.ui.graphics.vector.path
import androidx.compose.ui.unit.dp

/** 与详情页分享面板设计稿一致的 24dp 1.8px 线宽矢量图标集。 */
internal object ShareIcons {
    fun of(key: ShareIconKey): ImageVector = when (key) {
        ShareIconKey.SystemShare -> ShareNodes
        ShareIconKey.CopyLink -> Link
        ShareIconKey.Check -> Check
        ShareIconKey.ShareCard -> Card
        ShareIconKey.Sparkles -> Sparkles
        ShareIconKey.CopyText -> CopyText
        ShareIconKey.Browser -> Globe
        ShareIconKey.Delete -> Trash
        ShareIconKey.Desktop -> Monitor
        ShareIconKey.Code -> Code
        ShareIconKey.Download -> Download
        ShareIconKey.Send -> Send
        ShareIconKey.Back -> Back
        ShareIconKey.Close -> Close
        ShareIconKey.Alert -> Alert
        ShareIconKey.FileText -> FileText
        ShareIconKey.Newspaper -> Newspaper
    }

    private fun strokeIcon(name: String, strokeWidth: Float = 1.8f, block: PathBuilder.() -> Unit): ImageVector =
        ImageVector.Builder(name = name, defaultWidth = 24.dp, defaultHeight = 24.dp, viewportWidth = 24f, viewportHeight = 24f).apply {
            path(
                fill = null,
                stroke = SolidColor(Color.Black),
                strokeLineWidth = strokeWidth,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
                pathFillType = PathFillType.NonZero,
                pathBuilder = block,
            )
        }.build()

    private fun PathBuilder.circle(cx: Float, cy: Float, r: Float) {
        moveTo(cx + r, cy)
        arcTo(r, r, 0f, false, true, cx - r, cy)
        arcTo(r, r, 0f, false, true, cx + r, cy)
        close()
    }

    private fun PathBuilder.roundRect(x: Float, y: Float, w: Float, h: Float, r: Float) {
        moveTo(x + r, y)
        lineTo(x + w - r, y)
        arcTo(r, r, 0f, false, true, x + w, y + r)
        lineTo(x + w, y + h - r)
        arcTo(r, r, 0f, false, true, x + w - r, y + h)
        lineTo(x + r, y + h)
        arcTo(r, r, 0f, false, true, x, y + h - r)
        lineTo(x, y + r)
        arcTo(r, r, 0f, false, true, x + r, y)
        close()
    }

    val ShareNodes: ImageVector by lazy {
        strokeIcon("ShareNodes") {
            circle(18f, 5f, 3f)
            circle(6f, 12f, 3f)
            circle(18f, 19f, 3f)
            moveTo(8.59f, 13.51f); lineTo(15.42f, 17.49f)
            moveTo(15.41f, 6.51f); lineTo(8.59f, 10.49f)
        }
    }

    val Link: ImageVector by lazy {
        strokeIcon("Link") {
            moveTo(10f, 13f)
            arcTo(5f, 5f, 0f, false, false, 17.07f, 13f)
            lineTo(19.19f, 10.88f)
            arcTo(5f, 5f, 0f, false, false, 12.12f, 3.81f)
            lineTo(10.9f, 5f)
            moveTo(14f, 11f)
            arcTo(5f, 5f, 0f, false, false, 6.93f, 11f)
            lineTo(4.8f, 13.12f)
            arcTo(5f, 5f, 0f, false, false, 11.87f, 20.19f)
            lineTo(13.1f, 19f)
        }
    }

    val Check: ImageVector by lazy {
        strokeIcon("Check") {
            moveTo(20f, 6f); lineTo(9f, 17f); lineTo(4f, 12f)
        }
    }

    val Card: ImageVector by lazy {
        strokeIcon("Card") {
            roundRect(4f, 3.5f, 16f, 17f, 2.5f)
            moveTo(8f, 8.5f); lineTo(16f, 8.5f)
            moveTo(8f, 12f); lineTo(13f, 12f)
            circle(15f, 15.5f, 1.4f)
        }
    }

    val Sparkles: ImageVector by lazy {
        strokeIcon("Sparkles") {
            moveTo(12f, 3f)
            lineTo(13.7f, 8.1f)
            lineTo(18.8f, 9.8f)
            lineTo(13.7f, 11.5f)
            lineTo(12f, 16.6f)
            lineTo(10.3f, 11.5f)
            lineTo(5.2f, 9.8f)
            lineTo(10.3f, 8.1f)
            close()
        }
    }

    val CopyText: ImageVector by lazy {
        strokeIcon("CopyText") {
            roundRect(9f, 9f, 11f, 11f, 2f)
            moveTo(5f, 15f)
            lineTo(4f, 15f)
            arcTo(2f, 2f, 0f, false, true, 2f, 13f)
            lineTo(2f, 4f)
            arcTo(2f, 2f, 0f, false, true, 4f, 2f)
            lineTo(13f, 2f)
            arcTo(2f, 2f, 0f, false, true, 15f, 4f)
            lineTo(15f, 5f)
        }
    }

    val Globe: ImageVector by lazy {
        strokeIcon("Globe") {
            circle(12f, 12f, 9f)
            moveTo(3.6f, 9f); lineTo(20.4f, 9f)
            moveTo(3.6f, 15f); lineTo(20.4f, 15f)
            moveTo(12f, 3f)
            arcTo(13.5f, 13.5f, 0f, false, true, 15.4f, 12f)
            arcTo(13.5f, 13.5f, 0f, false, true, 12f, 21f)
            arcTo(13.5f, 13.5f, 0f, false, true, 8.6f, 12f)
            arcTo(13.5f, 13.5f, 0f, false, true, 12f, 3f)
        }
    }

    val Trash: ImageVector by lazy {
        strokeIcon("Trash") {
            moveTo(4f, 7f); lineTo(20f, 7f)
            moveTo(18f, 7f); lineTo(17f, 20f)
            arcTo(2f, 2f, 0f, false, true, 15f, 22f)
            lineTo(9f, 22f)
            arcTo(2f, 2f, 0f, false, true, 7f, 20f)
            lineTo(6f, 7f)
            moveTo(9f, 7f); lineTo(9f, 4.5f)
            arcTo(1.5f, 1.5f, 0f, false, true, 10.5f, 3f)
            lineTo(13.5f, 3f)
            arcTo(1.5f, 1.5f, 0f, false, true, 15f, 4.5f)
            lineTo(15f, 7f)
            moveTo(10f, 11.5f); lineTo(10f, 17f)
            moveTo(14f, 11.5f); lineTo(14f, 17f)
        }
    }

    val Monitor: ImageVector by lazy {
        strokeIcon("Monitor") {
            roundRect(3f, 4f, 18f, 12f, 2f)
            moveTo(8f, 20f); lineTo(16f, 20f)
            moveTo(12f, 16f); lineTo(12f, 20f)
        }
    }

    val Code: ImageVector by lazy {
        strokeIcon("Code") {
            moveTo(8f, 7f); lineTo(3f, 12f); lineTo(8f, 17f)
            moveTo(16f, 7f); lineTo(21f, 12f); lineTo(16f, 17f)
            moveTo(14f, 5f); lineTo(10f, 19f)
        }
    }

    val Download: ImageVector by lazy {
        strokeIcon("Download") {
            moveTo(12f, 4f); lineTo(12f, 15f)
            moveTo(7.5f, 11.5f); lineTo(12f, 16f); lineTo(16.5f, 11.5f)
            moveTo(4.5f, 19.5f); lineTo(19.5f, 19.5f)
        }
    }

    val Send: ImageVector by lazy {
        strokeIcon("Send") {
            moveTo(21f, 3f); lineTo(10f, 14f)
            moveTo(21f, 3f); lineTo(14.5f, 21f); lineTo(10.7f, 13.3f); lineTo(3f, 9.5f); close()
        }
    }

    val Back: ImageVector by lazy {
        strokeIcon("Back") {
            moveTo(15f, 6f); lineTo(9f, 12f); lineTo(15f, 18f)
        }
    }

    val Close: ImageVector by lazy {
        strokeIcon("Close") {
            moveTo(18f, 6f); lineTo(6f, 18f)
            moveTo(6f, 6f); lineTo(18f, 18f)
        }
    }

    val Alert: ImageVector by lazy {
        strokeIcon("Alert") {
            circle(12f, 12f, 9f)
            moveTo(12f, 8f); lineTo(12f, 12.5f)
            circle(12f, 16.2f, 0.45f)
        }
    }

    val FileText: ImageVector by lazy {
        strokeIcon("FileText") {
            moveTo(14f, 3f); lineTo(7f, 3f)
            arcTo(2f, 2f, 0f, false, false, 5f, 5f)
            lineTo(5f, 19f)
            arcTo(2f, 2f, 0f, false, false, 7f, 21f)
            lineTo(17f, 21f)
            arcTo(2f, 2f, 0f, false, false, 19f, 19f)
            lineTo(19f, 8f); close()
            moveTo(14f, 3f); lineTo(14f, 8f); lineTo(19f, 8f)
            moveTo(9f, 13f); lineTo(15f, 13f)
            moveTo(9f, 17f); lineTo(13f, 17f)
        }
    }

    val Newspaper: ImageVector by lazy {
        strokeIcon("Newspaper") {
            roundRect(4f, 4f, 16f, 16f, 2.5f)
            moveTo(8f, 9f); lineTo(16f, 9f)
            moveTo(8f, 13f); lineTo(13f, 13f)
            moveTo(8f, 16.5f); lineTo(16f, 16.5f)
        }
    }
}
