import AppKit
import AVFoundation
import ExtensionFoundation
import Foundation
import IOSurface
import OSLog
import Security

private let logger = Logger(subsystem: "xyz.xiaban.desktop-wallpaper", category: "LockScreenPrototype")

@main
final class XiaobanLockScreenPrototype: NSObject, AppExtension {
    override required init() {
        super.init()
        let framework = "/System/Library/PrivateFrameworks/WallpaperExtensionKit.framework/WallpaperExtensionKit"
        if dlopen(framework, RTLD_LAZY | RTLD_LOCAL) == nil {
            logger.error("WallpaperExtensionKit is unavailable")
        }
    }

    var configuration: some AppExtensionConfiguration { WallpaperConfiguration() }
}

private struct WallpaperConfiguration: AppExtensionConfiguration {
    func accept(connection: NSXPCConnection) -> Bool {
        guard isAppleHost(connection) else {
            logger.error("Rejected unverified wallpaper host")
            return false
        }
        let required = ["WallpaperIDXPC", "WallpaperCreationRequestXPC", "WallpaperRemoteContextXPC",
                        "WallpaperSettingsViewModelsXPC", "WallpaperSnapshotXPC"]
        guard required.allSatisfy({ objc_getClass($0) != nil }) else {
            logger.error("Unsupported wallpaper interface on this system")
            return false
        }
        let optional = ["WallpaperUpdateRequestXPC", "WallpaperContentTypeSetXPC",
                        "WallpaperChoiceIDXPC", "WallpaperChoiceIDsXPC", "WallpaperExtensionChoiceRequestXPC",
                        "WallpaperChoiceRequestAdditionResultXPC", "WallpaperDebugRequestXPC",
                        "WallpaperDebugResponseXPC", "WallpaperMigrationVersionXPC", "AuditTokenXPC"]
        let classes = NSMutableSet(array: [NSString.self, NSNumber.self, NSData.self, NSArray.self,
                                           NSDictionary.self, NSURL.self, NSError.self, NSUUID.self])
        for name in required + optional {
            if let cls = objc_getClass(name) { classes.add(cls) }
        }
        guard let allowed = classes as? Set<AnyHashable> else { return false }

        let interface = NSXPCInterface(with: (any WallpaperExtensionXPCProtocol).self)
        let methods: [(Selector, Int, Bool)] = [
            (#selector(Handler.acquire(withId:request:reply:)), 2, true),
            (#selector(Handler.update(withId:request:reply:)), 2, false),
            (#selector(Handler.invalidate(withId:reply:)), 1, false),
            (#selector(Handler.snapshot(withId:reply:)), 1, true),
            (#selector(Handler.provideSettingsViewModels(withContentTypes:reply:)), 1, true),
            (#selector(Handler.addChoiceRequest(withChoiceRequest:onBehalfOfProcess:reply:)), 2, true),
            (#selector(Handler.removeChoiceRequest(withChoiceRequest:reply:)), 1, false),
            (#selector(Handler.selectedChoicesDidChange(for:reply:)), 1, false),
            (#selector(Handler.invokeContextMenuAction(withMenuItemID:groupItemID:reply:)), 2, false),
            (#selector(Handler.isChoiceDownloaded(with:reply:)), 1, false),
            (#selector(Handler.download(withChoiceID:reply:)), 1, false),
            (#selector(Handler.pauseDownload(for:reply:)), 1, false),
            (#selector(Handler.cancelDownload(for:reply:)), 1, false),
            (#selector(Handler.resumeDownload(for:reply:)), 1, false),
            (#selector(Handler.removeDownload(for:reply:)), 1, false),
            (#selector(Handler.migrateSelectedChoice(for:reply:)), 1, true),
            (#selector(Handler.migrate(from:to:reply:)), 2, false),
            (#selector(Handler.skipShuffledContent(withId:reply:)), 1, false),
            (#selector(Handler.canSkipShuffledContent(withId:reply:)), 1, false),
            (#selector(Handler.handleDebugRequest(for:reply:)), 1, true),
            (#selector(Handler.handleNotification(withNamed:reply:)), 1, false),
        ]
        for (method, arguments, objectReply) in methods {
            for index in 0..<arguments {
                interface.setClasses(allowed, for: method, argumentIndex: index, ofReply: false)
            }
            if objectReply {
                interface.setClasses(allowed, for: method, argumentIndex: 0, ofReply: true)
            }
        }
        connection.exportedInterface = interface
        connection.exportedObject = Handler.shared
        connection.resume()
        return true
    }
}

private func isAppleHost(_ connection: NSXPCConnection) -> Bool {
    guard connection.responds(to: NSSelectorFromString("auditToken")) else { return false }
    var token = connection.auditToken
    let audit = withUnsafeBytes(of: &token) { Data($0) }
    var peer: SecCode?
    var requirement: SecRequirement?
    let flags = SecCSFlags(rawValue: 0)
    let attributes = [kSecGuestAttributeAudit: audit] as CFDictionary
    guard SecCodeCopyGuestWithAttributes(nil, attributes, flags, &peer) == errSecSuccess,
          SecRequirementCreateWithString("anchor apple" as CFString, flags, &requirement) == errSecSuccess,
          let peer, let requirement else { return false }
    return SecCodeCheckValidity(peer, flags, requirement) == errSecSuccess
}

private func unsupported(_ message: String) -> NSError {
    NSError(domain: "XiaobanLockScreenPrototype", code: 1, userInfo: [NSLocalizedDescriptionKey: message])
}

private func member(_ name: String, of value: Any, depth: Int = 0) -> Any? {
    guard depth < 6 else { return nil }
    for child in Mirror(reflecting: value).children {
        if child.label == name { return child.value }
        if let result = member(name, of: child.value, depth: depth + 1) { return result }
    }
    return nil
}

private func wallpaperID(_ value: Any?, depth: Int = 0) -> UUID? {
    guard let value, depth < 6 else { return nil }
    if let id = value as? UUID { return id }
    return Mirror(reflecting: value).children.lazy.compactMap { wallpaperID($0.value, depth: depth + 1) }.first
}

// WallpaperExtensionKit has no public initializer for these two replies. Only
// accept the expected named field and layout in a fresh reply object.
private func makeReply<Value>(_ name: String, field: String, value: Value) -> AnyObject? {
    guard let cls = objc_getClass(name) as? AnyClass,
          let ivar = class_getInstanceVariable(cls, field) else { return nil }
    let offset = ivar_getOffset(ivar)
    guard offset >= MemoryLayout<UnsafeRawPointer>.size,
          offset % MemoryLayout<Value>.alignment == 0,
          offset + MemoryLayout<Value>.size <= class_getInstanceSize(cls),
          let object = class_createInstance(cls, 0) as AnyObject? else { return nil }
    Unmanaged.passUnretained(object).toOpaque().advanced(by: offset).storeBytes(of: value, as: Value.self)
    return object
}

private final class Surface {
    let context: CAContext
    let root = CALayer()
    private let player: AVQueuePlayer
    private let loop: AVPlayerLooper
    private let drops = CAEmitterLayer()

    init?(size: CGSize, display: UInt32?) {
        guard size.width.isFinite, size.height.isFinite,
              (32...8192).contains(size.width), (32...8192).contains(size.height),
              let clip = Bundle.main.url(forResource: "scene", withExtension: "mp4") else { return nil }
        let rawContext: Any?
        if let display, CAContext.responds(to: NSSelectorFromString("remoteContextWithOptions:")) {
            rawContext = CAContext.perform(NSSelectorFromString("remoteContextWithOptions:"),
                                           with: ["displayId": display])?.takeUnretainedValue()
        } else {
            rawContext = CAContext.remoteContext()
        }
        guard let context = rawContext as? CAContext, context.contextId != 0 else { return nil }
        self.context = context
        player = AVQueuePlayer()
        player.isMuted = true
        loop = AVPlayerLooper(player: player, templateItem: AVPlayerItem(url: clip))

        root.frame = CGRect(origin: .zero, size: size)
        root.backgroundColor = NSColor(srgbRed: 0.03, green: 0.09, blue: 0.14, alpha: 1).cgColor
        root.masksToBounds = true
        let video = AVPlayerLayer(player: player)
        video.frame = root.bounds
        video.videoGravity = .resizeAspectFill
        root.addSublayer(video)

        let cell = CAEmitterCell()
        let stroke = CGContext(data: nil, width: 3, height: 22, bitsPerComponent: 8, bytesPerRow: 12,
                               space: CGColorSpaceCreateDeviceRGB(),
                               bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)
        stroke?.setFillColor(red: 0.80, green: 0.91, blue: 1.00, alpha: 0.55)
        stroke?.fill(CGRect(x: 1, y: 0, width: 1, height: 22))
        cell.contents = stroke?.makeImage()
        cell.birthRate = 80
        cell.lifetime = 3
        cell.velocity = 460
        cell.velocityRange = 130
        cell.emissionLongitude = -.pi / 2
        cell.emissionRange = .pi / 20
        cell.alphaSpeed = -0.16
        drops.frame = root.bounds
        drops.emitterPosition = CGPoint(x: size.width / 2, y: size.height + 24)
        drops.emitterSize = CGSize(width: size.width, height: 1)
        drops.emitterShape = .line
        drops.emitterCells = [cell]
        root.addSublayer(drops)

        let weather = CATextLayer()
        weather.string = "天气示例  18°C"
        weather.fontSize = 16
        weather.contentsScale = 2
        weather.alignmentMode = .center
        weather.frame = CGRect(x: max(0, size.width - 184), y: max(0, size.height - 116), width: 152, height: 24)
        weather.foregroundColor = NSColor.white.cgColor
        weather.backgroundColor = NSColor.black.withAlphaComponent(0.42).cgColor
        weather.cornerRadius = 8
        root.addSublayer(weather)

        context.layer = root
        setPlaying(true)
    }

    func setPlaying(_ playing: Bool) {
        drops.birthRate = playing ? 1 : 0
        if playing { player.play() } else { player.pause() }
    }

    deinit {
        player.pause()
        context.layer = nil
    }
}

private final class Handler: NSObject, WallpaperExtensionXPCProtocol, @unchecked Sendable {
    static let shared = Handler()
    private var surfaces = [UUID: Surface]()
    private var asleep = false
    private var observers: [NSObjectProtocol] = []

    override init() {
        super.init()
        let center = NSWorkspace.shared.notificationCenter
        observers = [
            center.addObserver(forName: NSWorkspace.screensDidSleepNotification, object: nil, queue: .main) { [weak self] _ in self?.setAsleep(true) },
            center.addObserver(forName: NSWorkspace.screensDidWakeNotification, object: nil, queue: .main) { [weak self] _ in self?.setAsleep(false) },
        ]
    }

    private func setAsleep(_ value: Bool) {
        asleep = value
        surfaces.values.forEach { $0.setPlaying(!value) }
    }

    func acquire(withId id: Any?, request: Any?, reply: @escaping @Sendable (Any?, (any Error)?) -> Void) {
        guard let key = wallpaperID(id), let request,
              let size = member("size", of: request) as? CGSize else {
            reply(nil, unsupported("不支持的壁纸请求"))
            return
        }
        let display = member("directDisplayID", of: request) as? UInt32
        let preview = member("isPreview", of: request) as? Bool ?? true
        DispatchQueue.main.async {
            guard self.surfaces[key] != nil || self.surfaces.count < 16,
                  let surface = Surface(size: size, display: display),
                  let result = makeReply("WallpaperRemoteContextXPC", field: "box", value: surface.context.contextId) else {
                logger.error("Could not create wallpaper surface")
                reply(nil, unsupported("未能创建系统壁纸渲染层"))
                return
            }
            surface.setPlaying(!self.asleep)
            self.surfaces[key] = surface
            CATransaction.flush()
            logger.notice("Wallpaper surface delivered, preview: \(preview)")
            reply(result, nil)
        }
    }

    func update(withId _: Any?, request _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func invalidate(withId id: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) {
        let key = wallpaperID(id)
        DispatchQueue.main.async {
            if let key { self.surfaces.removeValue(forKey: key) }
            reply(nil)
        }
    }

    func snapshot(withId _: Any?, reply: @escaping @Sendable (Any?, (any Error)?) -> Void) {
        let size = 640
        let format: UInt32 = 0x42475241
        let values: [IOSurfacePropertyKey: any Sendable] = [
            .width: size, .height: 360, .bytesPerElement: 4, .bytesPerRow: size * 4, .pixelFormat: format,
        ]
        guard let surface = IOSurface(properties: values) else { reply(nil, unsupported("无法生成预览")); return }
        surface.lock(options: [], seed: nil)
        if let bitmap = CGContext(data: surface.baseAddress, width: size, height: 360, bitsPerComponent: 8,
                                  bytesPerRow: surface.bytesPerRow, space: CGColorSpaceCreateDeviceRGB(),
                                  bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue | CGBitmapInfo.byteOrder32Little.rawValue) {
            bitmap.setFillColor(red: 0.03, green: 0.09, blue: 0.14, alpha: 1)
            bitmap.fill(CGRect(x: 0, y: 0, width: size, height: 360))
        }
        surface.unlock(options: [], seed: nil)
        let reference = Unmanaged.passRetained(surface)
        guard let snapshot = makeReply("WallpaperSnapshotXPC", field: "rawValue", value: reference.toOpaque()) else {
            reference.release()
            reply(nil, unsupported("不支持的壁纸预览接口"))
            return
        }
        reply(snapshot, nil)
    }

    func provideSettingsViewModels(withContentTypes _: Any?, reply: @escaping @Sendable (Any?, (any Error)?) -> Void) {
        guard let panel = makeSettingsViewModels() else { reply(nil, unsupported("不支持的壁纸选择界面")); return }
        reply(panel, nil)
    }

    func addChoiceRequest(withChoiceRequest _: Any?, onBehalfOfProcess _: Any?, reply: @escaping @Sendable (Any?, (any Error)?) -> Void) { reply(nil, nil) }
    func removeChoiceRequest(withChoiceRequest _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func selectedChoicesDidChange(for _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func invokeContextMenuAction(withMenuItemID _: Any?, groupItemID _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func isChoiceDownloaded(with _: Any?, reply: @escaping @Sendable (Bool, (any Error)?) -> Void) { reply(true, nil) }
    func download(withChoiceID _: Any?, reply: ((any Error)?) -> Void) -> Any? { reply(nil); return nil }
    func pauseDownload(for _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func cancelDownload(for _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func resumeDownload(for _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func removeDownload(for _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func migrateSelectedChoice(for _: Any?, reply: @escaping @Sendable (Any?, (any Error)?) -> Void) { reply(nil, nil) }
    func migrate(from _: Any?, to _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func skipShuffledContent(withId _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
    func canSkipShuffledContent(withId _: Any?, reply: @escaping @Sendable (Bool, (any Error)?) -> Void) { reply(false, nil) }
    func handleDebugRequest(for _: Any?, reply: @escaping @Sendable (Any?, (any Error)?) -> Void) { reply(nil, nil) }
    func handleNotification(withNamed _: Any?, reply: @escaping @Sendable ((any Error)?) -> Void) { reply(nil) }
}

private func makeSettingsViewModels() -> Any? {
    guard let provider = Bundle.main.bundleIdentifier,
          let preview = Bundle.main.url(forResource: "preview", withExtension: "png"),
          let realClass = objc_getClass("WallpaperSettingsViewModelsXPC") as? AnyClass else { return nil }
    let name = "小伴锁屏原型"
    let choice = "xiaoban-lock-screen-prototype"
    let providerID = ChoiceProviderID(rawValue: provider)
    let id = ChoiceID(id: choice, descriptor: ChoiceIDDescriptor(provider: providerID, identifier: choice,
                                                                files: [], configuration: Data(choice.utf8)))
    let thumbnail = Thumbnail.image(url: preview)
    let descriptor = ChoiceDescriptor(id: id, provider: providerID, identifier: choice, name: name,
                                      localizedDescription: "视频、雨雪和只读组件预览", thumbnail: thumbnail,
                                      isDownloaded: true, options: [])
    let item = SettingsItem(id: id, localizedName: name, thumbnail: thumbnail, choice: descriptor,
                            contentBadge: .video, showInTopLevel: true, sortOrder: 0, disposability: .none)
    let group = SettingsGroup(id: .init(id: choice), items: [item], localizedName: name, disposability: .none,
                              sortOrder: -100, sortID: .init(id: provider), allChoiceID: nil,
                              shouldHideItemLabels: false, contextMenu: nil, thumbnail: nil)
    let models = SettingsViewModels(desktop: .init(groups: [group], refreshPolicy: .default,
                                                   isModificationDisabled: false), screenSaver: nil)
    guard let bytes = try? NSKeyedArchiver.archivedData(withRootObject: ShimViewModelsXPC(value: models),
                                                      requiringSecureCoding: false),
          let unarchiver = try? NSKeyedUnarchiver(forReadingFrom: bytes) else { return nil }
    // Both encode and decode are local to this function. WallpaperAgent does not supply the archive.
    unarchiver.requiresSecureCoding = false
    unarchiver.decodingFailurePolicy = .setErrorAndReturn
    unarchiver.setClass(realClass, forClassName: "ShimViewModelsXPC")
    let result = unarchiver.decodeObject(forKey: NSKeyedArchiveRootObjectKey)
    unarchiver.finishDecoding()
    return result
}
