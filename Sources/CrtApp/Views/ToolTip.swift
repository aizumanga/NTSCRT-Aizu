import SwiftUI
import AppKit

/// AppKit-backed tooltip.
///
/// SwiftUI's `.help()` doesn't reliably display on custom `.buttonStyle(.plain)`
/// controls (our palette pills and timeline buttons), so the tooltip text is
/// attached to a real NSView instead: `toolTip` installs a geometry-based
/// tracking rect, which fires independently of hit-testing — so the overlay
/// can refuse hits (`hitTest` → nil) and let clicks reach the control beneath.
private struct ToolTipOverlay: NSViewRepresentable {
    let text: String

    final class PassThroughView: NSView {
        override func hitTest(_ point: NSPoint) -> NSView? { nil }
    }

    func makeNSView(context: Context) -> NSView {
        let v = PassThroughView()
        v.toolTip = text
        return v
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        nsView.toolTip = text
    }
}

extension View {
    /// Tooltip that works on custom controls (see ToolTipOverlay).
    func tooltip(_ text: String) -> some View {
        overlay(ToolTipOverlay(text: text))
    }
}
