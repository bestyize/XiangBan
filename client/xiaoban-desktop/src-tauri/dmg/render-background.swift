// Build-only DMG Finder background. Requires macOS AppKit (provided by Xcode).
// The image deliberately contains no fake app/folder icons: Finder places the real
// XiangBan .app and /Applications alias over the empty left/right spaces.
//
// Usage: swift src-tauri/dmg/render-background.swift src-tauri/dmg/background.png
// Tauri's macOS beforeBundleCommand runs this automatically before packaging.
import AppKit
import Foundation

private let width = 760
private let height = 440
private let canvasHeight = CGFloat(height)

private func color(_ hex: String) -> NSColor {
    guard hex.hasPrefix("#"), let number = Int(hex.dropFirst(), radix: 16) else {
        fatalError("Invalid color: \(hex)")
    }
    return NSColor(srgbRed: CGFloat((number >> 16) & 0xFF) / 255,
                   green: CGFloat((number >> 8) & 0xFF) / 255,
                   blue: CGFloat(number & 0xFF) / 255,
                   alpha: 1)
}

private func rect(_ x: CGFloat, _ y: CGFloat, _ w: CGFloat, _ h: CGFloat) -> NSRect {
    NSRect(x: x, y: canvasHeight - y - h, width: w, height: h)
}

private func rounded(_ x: CGFloat, _ y: CGFloat, _ w: CGFloat, _ h: CGFloat,
                     radius: CGFloat, fill: String, stroke: String? = nil) {
    let shape = NSBezierPath(roundedRect: rect(x, y, w, h), xRadius: radius, yRadius: radius)
    color(fill).setFill()
    shape.fill()
    if let stroke {
        color(stroke).setStroke()
        shape.lineWidth = 1
        shape.stroke()
    }
}

private func text(_ value: String, x: CGFloat, top: CGFloat, w: CGFloat,
                  size: CGFloat, weight: NSFont.Weight = .regular, ink: String = "#34313A") {
    let style = NSMutableParagraphStyle()
    style.alignment = .left
    style.lineBreakMode = .byClipping
    let attributes: [NSAttributedString.Key: Any] = [
        .font: NSFont.systemFont(ofSize: size, weight: weight),
        .foregroundColor: color(ink),
        .paragraphStyle: style
    ]
    NSString(string: value).draw(
        in: rect(x, top, w, size * 1.8),
        withAttributes: attributes
    )
}

private func line(_ points: [CGPoint], ink: String, lineWidth: CGFloat, dashed: Bool = false) {
    guard let first = points.first else { return }
    let path = NSBezierPath()
    path.move(to: CGPoint(x: first.x, y: canvasHeight - first.y))
    for point in points.dropFirst() {
        path.line(to: CGPoint(x: point.x, y: canvasHeight - point.y))
    }
    path.lineWidth = lineWidth
    path.lineCapStyle = .round
    path.lineJoinStyle = .round
    if dashed { path.setLineDash([4, 7], count: 2, phase: 0) }
    color(ink).setStroke()
    path.stroke()
}

guard CommandLine.arguments.count == 2 else {
    fputs("Usage: swift render-background.swift <output.png>\n", stderr)
    exit(2)
}
let destination = URL(fileURLWithPath: CommandLine.arguments[1])
try FileManager.default.createDirectory(at: destination.deletingLastPathComponent(),
                                         withIntermediateDirectories: true)

guard let bitmap = NSBitmapImageRep(
    bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
), let graphics = NSGraphicsContext(bitmapImageRep: bitmap) else {
    fatalError("Unable to initialize DMG background renderer")
}

NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = graphics
graphics.imageInterpolation = .high

// Warm porcelain base, without gradients. Non-translucent to stay legible in Finder.
color("#F8F8F9").setFill()
NSRect(x: 0, y: 0, width: width, height: height).fill()

rounded(31, 24, 26, 26, radius: 9, fill: "#F26BAA")
text("✦", x: 37, top: 26, w: 20, size: 15, weight: .bold, ink: "#FFFFFF")
text("小伴桌面", x: 67, top: 26, w: 160, size: 15, weight: .semibold)
text("MACOS 安装指引", x: 610, top: 31, w: 160, size: 10, weight: .medium, ink: "#A29AA3")

text("把小伴带回桌面", x: 31, top: 67, w: 410, size: 25, weight: .semibold)
text("将左侧的小伴拖到右侧「应用程序」文件夹，即可完成安装。", x: 32, top: 106,
     w: 665, size: 12, ink: "#716B75")

// Finder puts the *actual* application icon and the Applications alias here:
// App: x=177 y=220; folder: x=583 y=220. Do not paint imitations.
rounded(31, 147, 698, 162, radius: 17, fill: "#FFFFFF", stroke: "#EEEAF0")
rounded(307, 171, 144, 27, radius: 13, fill: "#FFF1F7")
text("拖动以安装", x: 338, top: 175, w: 112, size: 10, weight: .medium, ink: "#B83378")

// Light dash and a clear direction indicator; allow 100+ px around each Finder icon.
line([CGPoint(x: 311, y: 227), CGPoint(x: 438, y: 227)],
     ink: "#E7A0C7", lineWidth: 3, dashed: true)
line([CGPoint(x: 431, y: 219), CGPoint(x: 443, y: 227), CGPoint(x: 431, y: 235)],
     ink: "#D64D93", lineWidth: 3)

// First-launch help. This is only information: it does NOT disable Gatekeeper.
rounded(31, 323, 698, 90, radius: 15, fill: "#FFF2F8", stroke: "#F6E6EF")
rounded(47, 344, 25, 25, radius: 12, fill: "#FBE0EE")
text("i", x: 57, top: 344, w: 18, size: 14, weight: .semibold, ink: "#C53C83")
text("首次打开提示", x: 87, top: 338, w: 200, size: 13, weight: .semibold, ink: "#A73E78")
text("如果 macOS 阻止打开小伴，先尝试运行应用，然后打开「系统设置 → 隐私与安全性」，",
     x: 87, top: 362, w: 610, size: 11, ink: "#5F5864")
text("找到小伴并点击「仍要打开」。仅在确认下载来源可信时继续。",
     x: 87, top: 381, w: 610, size: 11, ink: "#5F5864")

graphics.flushGraphics()
NSGraphicsContext.restoreGraphicsState()

guard let data = bitmap.representation(using: .png, properties: [:]) else {
    fatalError("Unable to encode PNG background")
}
try data.write(to: destination, options: .atomic)
print("Generated \(destination.path) (\(width)x\(height), \(data.count) bytes)")
