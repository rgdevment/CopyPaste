import Cocoa
import FlutterMacOS
import AVFoundation
import ApplicationServices

public class ListenerPlugin: NSObject, FlutterPlugin {

  private var eventSink: FlutterEventSink?
  private var pollingTimer: Timer?
  private var lastChangeCount: Int = 0
  private var lastContentHash: String = ""
  private var lastChangeTick: UInt64 = 0

  private static let debounceMs: UInt64 = 500
  private static let pollingIntervalSec: TimeInterval = 0.5

  public static func register(with registrar: FlutterPluginRegistrar) {
    let instance = ListenerPlugin()

    let eventChannel = FlutterEventChannel(
      name: "copypaste/clipboard",
      binaryMessenger: registrar.messenger
    )
    eventChannel.setStreamHandler(instance)

    let methodChannel = FlutterMethodChannel(
      name: "copypaste/clipboard_writer",
      binaryMessenger: registrar.messenger
    )
    methodChannel.setMethodCallHandler(instance.handleMethodCall)
  }

  // MARK: - Method Channel Handler

  private func handleMethodCall(call: FlutterMethodCall, result: @escaping FlutterResult) {
    switch call.method {
    case "setClipboardContent":
      handleSetClipboard(call: call, result: result)
    case "getMediaInfo":
      handleGetMediaInfo(call: call, result: result)
    case "captureFrontmostApp":
      result(NSWorkspace.shared.frontmostApplication?.bundleIdentifier)
    case "activateAndPaste":
      handleActivateAndPaste(call: call, result: result)
    case "getCursorAndScreenInfo":
      handleCursorAndScreenInfo(result: result)
    case "checkAccessibility":
      result(AXIsProcessTrusted())
    case "requestAccessibility":
      let options = [
        kAXTrustedCheckOptionPrompt.takeUnretainedValue(): true,
      ] as CFDictionary
      result(AXIsProcessTrustedWithOptions(options))
    case "openAccessibilitySettings":
      if #available(macOS 13.0, *) {
        NSWorkspace.shared.open(
          URL(string: "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility")!
        )
      } else {
        NSWorkspace.shared.open(
          URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")!
        )
      }
      result(true)
    default:
      result(FlutterMethodNotImplemented)
    }
  }

  // MARK: - Clipboard Monitoring

  private func startPolling() {
    lastChangeCount = NSPasteboard.general.changeCount
    pollingTimer = Timer.scheduledTimer(
      withTimeInterval: ListenerPlugin.pollingIntervalSec,
      repeats: true
    ) { [weak self] _ in
      self?.checkClipboard()
    }
  }

  private func stopPolling() {
    pollingTimer?.invalidate()
    pollingTimer = nil
  }

  private func checkClipboard() {
    let pb = NSPasteboard.general
    let currentCount = pb.changeCount
    guard currentCount != lastChangeCount else { return }
    lastChangeCount = currentCount
    onClipboardChanged()
  }

  private func onClipboardChanged() {
    let pb = NSPasteboard.general

    let hash = computeClipboardHash(pb)
    if !hash.isEmpty && isDuplicate(hash) { return }

    let source = getClipboardSource()

    var event: [String: Any]?

    if let fileUrls = pb.readObjects(forClasses: [NSURL.self], options: [
      .urlReadingFileURLsOnly: true,
    ]) as? [URL], !fileUrls.isEmpty {
      event = buildFileEvent(fileUrls: fileUrls, source: source, hash: hash)
    } else if let text = pb.string(forType: .string), !text.isEmpty {
      event = buildTextEvent(pb: pb, text: text, source: source, hash: hash)
    } else if let tiffData = pb.data(forType: .tiff), !tiffData.isEmpty {
      event = buildImageEvent(imageData: tiffData, source: source, hash: hash)
    }

    guard let eventMap = event else { return }
    DispatchQueue.main.async { [weak self] in
      self?.eventSink?(eventMap)
    }
  }

  // MARK: - Event Builders

  private func buildTextEvent(
    pb: NSPasteboard,
    text: String,
    source: String,
    hash: String
  ) -> [String: Any] {
    let isUrl = Self.isUrl(text)
    let eventType: Int = isUrl ? 4 : 0

    var event: [String: Any] = [
      "type": eventType,
      "text": text,
      "source": source,
      "contentHash": hash,
    ]

    if let rtfData = pb.data(forType: .rtf), !rtfData.isEmpty {
      event["rtf"] = FlutterStandardTypedData(bytes: rtfData)
    }
    if let htmlData = pb.data(forType: .html), !htmlData.isEmpty {
      event["html"] = FlutterStandardTypedData(bytes: htmlData)
    }

    return event
  }

  private func buildImageEvent(
    imageData: Data,
    source: String,
    hash: String
  ) -> [String: Any] {
    guard let bitmap = NSBitmapImageRep(data: imageData) else { return [:] }
    guard let bmpData = bitmap.representation(using: .bmp, properties: [:]) else { return [:] }

    return [
      "type": 1,
      "bytes": FlutterStandardTypedData(bytes: bmpData),
      "source": source,
      "contentHash": hash,
    ]
  }

  private func buildFileEvent(
    fileUrls: [URL],
    source: String,
    hash: String
  ) -> [String: Any] {
    let paths = fileUrls.map { $0.path }
    var eventType = 2

    if fileUrls.count == 1 {
      eventType = Self.detectFileType(url: fileUrls[0])
    }

    return [
      "type": eventType,
      "files": paths,
      "source": source,
      "contentHash": hash,
    ]
  }

  // MARK: - Deduplication

  private func isDuplicate(_ hash: String) -> Bool {
    let now = DispatchTime.now().uptimeNanoseconds / 1_000_000
    if hash == lastContentHash && (now - lastChangeTick) < ListenerPlugin.debounceMs {
      return true
    }
    lastContentHash = hash
    lastChangeTick = now
    return false
  }

  private func computeClipboardHash(_ pb: NSPasteboard) -> String {
    var signature = ""

    if let text = pb.string(forType: .string), !text.isEmpty {
      let sample = text.count > 100 ? String(text.prefix(100)) : text
      signature += "T:" + sample
    } else if let fileUrls = pb.readObjects(forClasses: [NSURL.self], options: [
      .urlReadingFileURLsOnly: true,
    ]) as? [URL], !fileUrls.isEmpty {
      for url in fileUrls {
        signature += "F:" + url.path + "|"
      }
    } else if let tiffData = pb.data(forType: .tiff), !tiffData.isEmpty {
      let sampleSize = min(tiffData.count, 256)
      let sample = tiffData.prefix(sampleSize)
      signature += "I:\(tiffData.count):" + sample.map { String(format: "%02x", $0) }.joined()
    }

    if signature.isEmpty { return "" }
    return Self.computeFnv1a(signature)
  }

  private func getClipboardSource() -> String {
    guard let frontApp = NSWorkspace.shared.frontmostApplication else { return "" }
    return frontApp.localizedName ?? frontApp.bundleIdentifier ?? ""
  }

  // MARK: - File Type Detection

  static func detectFileType(url: URL) -> Int {
    var isDirectory: ObjCBool = false
    if FileManager.default.fileExists(atPath: url.path, isDirectory: &isDirectory),
       isDirectory.boolValue {
      return 3
    }

    let ext = url.pathExtension.lowercased()

    let audioExts: Set<String> = ["mp3", "wav", "flac", "aac", "ogg", "wma", "m4a"]
    let videoExts: Set<String> = ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm"]
    let imageExts: Set<String> = [
      "png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "tiff", "heic",
    ]

    if audioExts.contains(ext) { return 5 }
    if videoExts.contains(ext) { return 6 }
    if imageExts.contains(ext) { return 1 }
    return 2
  }

  // MARK: - URL Detection

  static func isUrl(_ text: String) -> Bool {
    guard text.count >= 5 else { return false }
    let lower = text.lowercased()
    let prefixes = ["https://", "http://", "ftp://", "file:///", "mailto:"]
    guard prefixes.contains(where: { lower.hasPrefix($0) }) else { return false }
    return !text.contains(" ") && !text.contains("\n")
  }

  // MARK: - FNV-1a Hash

  static func computeFnv1a(_ data: String) -> String {
    var hash: UInt64 = 14695981039346656037
    for byte in data.utf8 {
      hash ^= UInt64(byte)
      hash &*= 1099511628211
    }
    return String(hash, radix: 16)
  }

  // MARK: - Set Clipboard Content

  private func handleSetClipboard(call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let type = args["type"] as? Int else {
      result(FlutterError(
        code: "invalid_args", message: "Expected map with 'type'", details: nil
      ))
      return
    }

    let success: Bool

    switch type {
    case 0, 4:
      let content = args["content"] as? String ?? ""
      let plainText = args["plainText"] as? Bool ?? (type == 4)

      var rtfData: Data?
      var htmlData: Data?
      if !plainText {
        if let rtfTyped = args["rtf"] as? FlutterStandardTypedData {
          rtfData = rtfTyped.data
        }
        if let htmlTyped = args["html"] as? FlutterStandardTypedData {
          htmlData = htmlTyped.data
        }
      }

      success = setTextToClipboard(text: content, rtf: rtfData, html: htmlData)

    case 1:
      let imagePath = args["content"] as? String ?? ""
      success = setImageToClipboard(imagePath: imagePath)

    case 2, 3, 5, 6:
      let content = args["content"] as? String ?? ""
      let paths = content.split(separator: "\n").map(String.init).filter { !$0.isEmpty }
      success = setFilesToClipboard(paths: paths)

    default:
      success = false
    }

    lastChangeCount = NSPasteboard.general.changeCount
    result(success)
  }

  private func setTextToClipboard(text: String, rtf: Data?, html: Data?) -> Bool {
    guard !text.isEmpty else { return false }
    let pb = NSPasteboard.general
    pb.clearContents()

    var types: [NSPasteboard.PasteboardType] = [.string]
    if let rtf, !rtf.isEmpty { types.append(.rtf) }
    if let html, !html.isEmpty { types.append(.html) }

    pb.declareTypes(types, owner: nil)
    pb.setString(text, forType: .string)

    if let rtf, !rtf.isEmpty {
      pb.setData(rtf, forType: .rtf)
    }
    if let html, !html.isEmpty {
      pb.setData(html, forType: .html)
    }

    return true
  }

  private func setImageToClipboard(imagePath: String) -> Bool {
    guard !imagePath.isEmpty else { return false }
    let url = URL(fileURLWithPath: imagePath)
    guard let image = NSImage(contentsOf: url) else { return false }
    guard let tiffData = image.tiffRepresentation else { return false }

    let pb = NSPasteboard.general
    pb.clearContents()
    pb.declareTypes([.tiff], owner: nil)
    pb.setData(tiffData, forType: .tiff)

    return true
  }

  private func setFilesToClipboard(paths: [String]) -> Bool {
    guard !paths.isEmpty else { return false }
    let urls = paths.compactMap { path -> URL? in
      let url = URL(fileURLWithPath: path)
      return FileManager.default.fileExists(atPath: url.path) ? url : nil
    }
    guard !urls.isEmpty else { return false }

    let pb = NSPasteboard.general
    pb.clearContents()
    pb.writeObjects(urls as [NSPasteboardWriting])

    return true
  }

  // MARK: - Activate & Paste (CGEvent)

  private func handleActivateAndPaste(call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let bundleId = args["bundleId"] as? String,
          let delayMs = args["delayMs"] as? Int else {
      result(false)
      return
    }

    if !AXIsProcessTrusted() {
      result(
        FlutterError(
          code: "ACCESSIBILITY_DENIED",
          message: "Accessibility permission not granted",
          details: nil
        )
      )
      return
    }

    guard let app = NSRunningApplication.runningApplications(
      withBundleIdentifier: bundleId
    ).first else {
      result(false)
      return
    }

    app.activate()

    let maxAttempts = max(delayMs / 10, 15)
    waitForFocusThenPaste(bundleId: bundleId, attempt: 0, maxAttempts: maxAttempts, result: result)
  }

  private func waitForFocusThenPaste(bundleId: String, attempt: Int, maxAttempts: Int, result: @escaping FlutterResult) {
    let focused = NSWorkspace.shared.frontmostApplication?.bundleIdentifier == bundleId

    if focused || attempt >= maxAttempts {
      if !focused {
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(30)) {
          self.simulatePaste(result: result)
        }
      } else {
        simulatePaste(result: result)
      }
      return
    }

    DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(10)) {
      self.waitForFocusThenPaste(bundleId: bundleId, attempt: attempt + 1, maxAttempts: maxAttempts, result: result)
    }
  }

  private func simulatePaste(result: @escaping FlutterResult) {
    let src = CGEventSource(stateID: .combinedSessionState)
    let vKey: CGKeyCode = 0x09

    guard let keyDown = CGEvent(keyboardEventSource: src, virtualKey: vKey, keyDown: true),
          let keyUp = CGEvent(keyboardEventSource: src, virtualKey: vKey, keyDown: false) else {
      result(false)
      return
    }

    keyDown.flags = .maskCommand
    keyUp.flags = .maskCommand
    keyDown.post(tap: .cghidEventTap)
    keyUp.post(tap: .cghidEventTap)
    result(true)
  }

  // MARK: - Cursor & Screen Info

  private func handleCursorAndScreenInfo(result: @escaping FlutterResult) {
    let mouseLocation = NSEvent.mouseLocation
    guard let mainScreen = NSScreen.main else {
      result(nil)
      return
    }

    let mainH = mainScreen.frame.height
    let cursorX = mouseLocation.x
    let cursorY = mainH - mouseLocation.y

    var info: [String: Double] = ["cursorX": cursorX, "cursorY": cursorY]

    for screen in NSScreen.screens {
      if screen.frame.contains(mouseLocation) {
        let vf = screen.visibleFrame
        info["waLeft"] = vf.origin.x
        info["waTop"] = mainH - vf.origin.y - vf.height
        info["waRight"] = vf.origin.x + vf.width
        info["waBottom"] = mainH - vf.origin.y
        break
      }
    }

    if info["waLeft"] == nil {
      let vf = mainScreen.visibleFrame
      info["waLeft"] = vf.origin.x
      info["waTop"] = mainH - vf.origin.y - vf.height
      info["waRight"] = vf.origin.x + vf.width
      info["waBottom"] = mainH - vf.origin.y
    }

    result(info)
  }

  // MARK: - Media Info

  private func handleGetMediaInfo(call: FlutterMethodCall, result: @escaping FlutterResult) {
    guard let args = call.arguments as? [String: Any],
          let path = args["path"] as? String else {
      result(nil)
      return
    }

    guard FileManager.default.fileExists(atPath: path) else {
      result(nil)
      return
    }

    let url = URL(fileURLWithPath: path)
    var info: [String: Any] = [:]
    let asset = AVURLAsset(url: url)

    let duration = CMTimeGetSeconds(asset.duration)
    if duration.isFinite && duration > 0 {
      info["duration"] = Int(duration)
    }

    if let videoTrack = asset.tracks(withMediaType: .video).first {
      let size = videoTrack.naturalSize
      let transform = videoTrack.preferredTransform
      let transformedSize = size.applying(transform)
      info["video_width"] = Int(abs(transformedSize.width))
      info["video_height"] = Int(abs(transformedSize.height))
    }

    for item in asset.commonMetadata {
      if item.commonKey == .commonKeyArtist,
         let artist = item.stringValue, !artist.isEmpty {
        info["artist"] = artist
      }
      if item.commonKey == .commonKeyTitle,
         let title = item.stringValue, !title.isEmpty {
        info["title"] = title
      }
      if item.commonKey == .commonKeyAlbumName,
         let album = item.stringValue, !album.isEmpty {
        info["album"] = album
      }
    }

    result(info.isEmpty ? nil : info)
  }
}

// MARK: - FlutterStreamHandler

extension ListenerPlugin: FlutterStreamHandler {
  public func onListen(
    withArguments arguments: Any?,
    eventSink events: @escaping FlutterEventSink
  ) -> FlutterError? {
    eventSink = events
    startPolling()
    return nil
  }

  public func onCancel(withArguments arguments: Any?) -> FlutterError? {
    stopPolling()
    eventSink = nil
    return nil
  }
}
