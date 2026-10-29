package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.unit.dp

/** 设计稿里用到的线条图标（24×24，描边 1.8）。路径数据直接取自设计稿的 SVG，由脚本生成。 */
object XIcons {
    val ArrowRight: ImageVector by lazy { strokeIcon("ArrowRight", "M5 12h13M13 6l6 6-6 6") }
    val ArrowUp: ImageVector by lazy { strokeIcon("ArrowUp", "M12 19V5M6 11l6-6 6 6") }
    val Bell: ImageVector by lazy { strokeIcon("Bell", "M6 17V11a6 6 0 0112 0v6l1.5 2h-15zM10 21h4") }
    val Bolt: ImageVector by lazy { strokeIcon("Bolt", "M13 3L5 14h6l-1 7 8-11h-6z") }
    val Book: ImageVector by lazy { strokeIcon("Book", "M5 4h10a3 3 0 013 3v13H8a3 3 0 01-3-3zM5 17a3 3 0 013-3h10") }
    val Bookmark: ImageVector by lazy { strokeIcon("Bookmark", "M7 4h10a1 1 0 011 1v15l-6-4-6 4V5a1 1 0 011-1z") }
    val Camera: ImageVector by lazy { strokeIcon("Camera", "M4 8h3l1.5-2h7L17 8h3a1 1 0 011 1v9a1 1 0 01-1 1H4a1 1 0 01-1-1V9a1 1 0 011-1zM8.5 13a3.5 3.5 0 1 0 7 0a3.5 3.5 0 1 0 -7 0z") }
    val Check: ImageVector by lazy { strokeIcon("Check", "M5 12l5 5 9-10") }
    val CheckBold: ImageVector by lazy { strokeIcon("CheckBold", "M5 12.5l4.5 4.5L19 7") }
    val ChevronDown: ImageVector by lazy { strokeIcon("ChevronDown", "M6 9l6 6 6-6") }
    val ChevronLeft: ImageVector by lazy { strokeIcon("ChevronLeft", "M15 5l-7 7 7 7") }
    val ChevronRight: ImageVector by lazy { strokeIcon("ChevronRight", "M9 5l7 7-7 7") }
    val Clock: ImageVector by lazy { strokeIcon("Clock", "M3.5 12a8.5 8.5 0 1 0 17 0a8.5 8.5 0 1 0 -17 0zM12 7.5V12l3 2") }
    val Copy: ImageVector by lazy { strokeIcon("Copy", "M11 9h7a2 2 0 012 2v7a2 2 0 01-2 2h-7a2 2 0 01-2-2v-7a2 2 0 012-2zM5 15V6a2 2 0 012-2h9") }
    val Close: ImageVector by lazy { strokeIcon("Close", "M6 6l12 12M18 6L6 18") }
    val File: ImageVector by lazy { strokeIcon("File", "M7 3h7l5 5v13H7zM14 3v5h5") }
    val Flag: ImageVector by lazy { strokeIcon("Flag", "M5 21V4M5 4h11l-2 4 2 4H5") }
    val Folder: ImageVector by lazy { strokeIcon("Folder", "M4 7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H6a2 2 0 01-2-2z") }
    val Gamepad: ImageVector by lazy { strokeIcon("Gamepad", "M8 8h8a5 5 0 0 1 5 5v0a5 5 0 0 1 -5 5h-8a5 5 0 0 1 -5 -5v-0a5 5 0 0 1 5 -5zM8 11v4M6 13h4M15.2 12a0.8 0.8 0 1 0 1.6 0a0.8 0.8 0 1 0 -1.6 0zM17.2 14a0.8 0.8 0 1 0 1.6 0a0.8 0.8 0 1 0 -1.6 0z") }
    val Globe: ImageVector by lazy { strokeIcon("Globe", "M3 12a9 9 0 1 0 18 0a9 9 0 1 0 -18 0zM3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18") }
    val Grid: ImageVector by lazy { strokeIcon("Grid", "M6 4h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2zM15 4h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2zM6 13h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2zM15 13h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2z") }
    val Hash: ImageVector by lazy { strokeIcon("Hash", "M5 9h14M5 15h14M10 4L8 20M16 4l-2 16") }
    val Heart: ImageVector by lazy { strokeIcon("Heart", "M12 20s-7-4.4-7-10a4 4 0 017-2.6A4 4 0 0119 10c0 5.6-7 10-7 10z") }
    val Image: ImageVector by lazy { strokeIcon("Image", "M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2zM7.4 10a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM4 18l5-5 4 4 3-3 4 4") }
    val Link: ImageVector by lazy { strokeIcon("Link", "M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1M14 10a4 4 0 00-5.7 0l-3 3a4 4 0 005.7 5.7l1-1") }
    val Lock: ImageVector by lazy { strokeIcon("Lock", "M7 11h10a2 2 0 0 1 2 2v5a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-5a2 2 0 0 1 2 -2zM8 11V8a4 4 0 018 0v3") }
    val Menu: ImageVector by lazy { strokeIcon("Menu", "M4 7h16M4 12h16M4 17h10") }
    val Message: ImageVector by lazy { strokeIcon("Message", "M5 6h14a1 1 0 011 1v9a1 1 0 01-1 1h-7l-4 3v-3H5a1 1 0 01-1-1V7a1 1 0 011-1z") }
    val Mic: ImageVector by lazy { strokeIcon("Mic", "M12 3h0a3 3 0 0 1 3 3v5a3 3 0 0 1 -3 3h-0a3 3 0 0 1 -3 -3v-5a3 3 0 0 1 3 -3zM5 11a7 7 0 0014 0M12 18v3") }
    val Monitor: ImageVector by lazy { strokeIcon("Monitor", "M5 5h14a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2zM8 21h8M12 17v4") }
    val More: ImageVector by lazy { strokeIcon("More", "M3.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM10.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM17.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0z") }
    val MoreVertical: ImageVector by lazy { strokeIcon("MoreVertical", "M10.4 5a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM10.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM10.4 19a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0z") }
    val NewChat: ImageVector by lazy { strokeIcon("NewChat", "M12 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-6M17.5 3.5a2 2 0 013 3L12 15l-4 1 1-4z") }
    val Pen: ImageVector by lazy { strokeIcon("Pen", "M4 20l4-1 11-11a2 2 0 00-3-3L5 16l-1 4z") }
    val Pause: ImageVector by lazy { strokeIcon("Pause", "M9 6v12M15 6v12") }
    val Plan: ImageVector by lazy { strokeIcon("Plan", "M5 4v16M5 5h12l-2.5 4 2.5 4H5") }
    val Plus: ImageVector by lazy { strokeIcon("Plus", "M12 5v14M5 12h14") }
    val Question: ImageVector by lazy { strokeIcon("Question", "M3.5 12a8.5 8.5 0 1 0 17 0a8.5 8.5 0 1 0 -17 0zM9.5 9.5a2.5 2.5 0 114 2c-.9.6-1.5 1-1.5 2M12 17v.5") }
    val Refresh: ImageVector by lazy { strokeIcon("Refresh", "M20 12a8 8 0 10-2.3 5.6M20 5v5h-5") }
    val Search: ImageVector by lazy { strokeIcon("Search", "M4.5 11a6.5 6.5 0 1 0 13 0a6.5 6.5 0 1 0 -13 0zM16 16l4.5 4.5") }
    val Share: ImageVector by lazy { strokeIcon("Share", "M12 15V4M8 8l4-4 4 4M5 12v6a2 2 0 002 2h10a2 2 0 002-2v-6") }
    val Shield: ImageVector by lazy { strokeIcon("Shield", "M12 3l8 3v6c0 5-3.5 8-8 9-4.5-1-8-4-8-9V6z") }
    val Sliders: ImageVector by lazy { strokeIcon("Sliders", "M4 7h10M18 7h2M4 17h2M10 17h10M14 7a2 2 0 1 0 4 0a2 2 0 1 0 -4 0zM6 17a2 2 0 1 0 4 0a2 2 0 1 0 -4 0z") }
    val Sparkle: ImageVector by lazy { strokeIcon("Sparkle", "M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8zM19 17l.7 1.8 1.8.7-1.8.7L19 22l-.7-1.8-1.8-.7 1.8-.7z") }
    val SparkleSmall: ImageVector by lazy { strokeIcon("SparkleSmall", "M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z") }
    val Star: ImageVector by lazy { strokeIcon("Star", "M12 3.5l2.6 5.4 5.9.8-4.3 4.1 1 5.9-5.2-2.8-5.2 2.8 1-5.9-4.3-4.1 5.9-.8z") }
    val Stop: ImageVector by lazy { strokeIcon("Stop", "M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2z") }
    val Swap: ImageVector by lazy { strokeIcon("Swap", "M7 10l5-5 5 5M7 14l5 5 5-5") }
    val Target: ImageVector by lazy { strokeIcon("Target", "M3.5 12a8.5 8.5 0 1 0 17 0a8.5 8.5 0 1 0 -17 0zM9 12a3 3 0 1 0 6 0a3 3 0 1 0 -6 0zM12 3.5v5M20.5 12h-5M12 20.5v-5M3.5 12h5") }
    val Tool: ImageVector by lazy { strokeIcon("Tool", "M14.5 6.5a4 4 0 00-5 5L4 17l3 3 5.5-5.5a4 4 0 005-5l-2.5 2.5-2.5-.5-.5-2.5z") }
    val Upload: ImageVector by lazy { strokeIcon("Upload", "M12 16V5M8 9l4-4 4 4M5 15v3a2 2 0 002 2h10a2 2 0 002-2v-3") }
    val User: ImageVector by lazy { strokeIcon("User", "M8.5 8.5a3.5 3.5 0 1 0 7 0a3.5 3.5 0 1 0 -7 0zM5 20c.8-3.6 3.5-5.5 7-5.5s6.2 1.9 7 5.5") }
    val Volume: ImageVector by lazy { strokeIcon("Volume", "M4 10v4h4l5 4V6L8 10zM16 9a4 4 0 010 6") }
    val Warning: ImageVector by lazy { strokeIcon("Warning", "M12 4l9 16H3zM12 10v4M12 17v.5") }
    val Wave: ImageVector by lazy { strokeIcon("Wave", "M4 11v2M8 8v8M12 5v14M16 8v8M20 11v2") }
    val Wechat: ImageVector by lazy { strokeIcon("Wechat", "M9.5 4C5.9 4 3 6.3 3 9.2c0 1.6.9 3 2.3 4L4.8 15l2.2-1.2c.8.2 1.7.4 2.5.4M15 10c-3 0-5.5 2-5.5 4.5S12 19 15 19c.7 0 1.4-.1 2-.3l2 1-.5-1.8c1.2-.8 2-2 2-3.4C20.5 12 18 10 15 10z") }
}

private fun strokeIcon(name: String, path: String): ImageVector =
    ImageVector.Builder(name = name, defaultWidth = 24.dp, defaultHeight = 24.dp, viewportWidth = 24f, viewportHeight = 24f)
        .addPath(
            pathData = addPathNodes(path),
            stroke = SolidColor(Color.Black),
            strokeLineWidth = 1.8f,
            strokeLineCap = StrokeCap.Round,
            strokeLineJoin = StrokeJoin.Round,
        )
        .build()
