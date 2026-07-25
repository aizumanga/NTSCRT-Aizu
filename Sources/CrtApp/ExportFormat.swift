import Foundation
import CrtCore

/// What the Export popover writes. GIF sits alongside the video codecs in
/// the picker but takes a different path (GifExporter) and its own size and
/// frame rate, so it can't just be another `Mp4Exporter.Codec` case.
enum ExportFormat: String, CaseIterable, Hashable {
    case h264 = "H.264"
    case hevc = "HEVC"
    case prores422 = "ProRes 422"
    case prores422HQ = "ProRes 422 HQ"
    case gif = "GIF"

    var isGIF: Bool { self == .gif }

    /// nil for GIF — the AVFoundation path doesn't apply.
    var codec: Mp4Exporter.Codec? {
        switch self {
        case .h264: return .h264
        case .hevc: return .hevc
        case .prores422: return .prores422
        case .prores422HQ: return .prores422HQ
        case .gif: return nil
        }
    }

    var isProRes: Bool { codec?.isProRes ?? false }

    var fileExtension: String { isGIF ? "gif" : (codec?.fileExtension ?? "mp4") }

    /// Short name for the export button.
    var buttonName: String {
        switch self {
        case .gif: return "GIF"
        case .prores422, .prores422HQ: return "MOV"
        default: return "MP4"
        }
    }
}
