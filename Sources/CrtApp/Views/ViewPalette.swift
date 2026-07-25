import SwiftUI

/// Floating palette over the preview: display-only controls that affect how
/// the preview renders, not the exported pixels — compare split, integer
/// scale, animate, zoom + reset. ContentView fades it out after the pointer
/// has been idle over the preview for a couple of seconds.
struct ViewPalette: View {
    @Environment(AppState.self) private var state

    var body: some View {
        @Bindable var state = state
        HStack(spacing: 8) {
            toggle($state.compareEnabled, icon: "square.split.2x1",
                   help: "Compare: full pipeline on the left of the line, untouched original on the right. Drag the line in the preview to move the split.")
            toggle($state.integerScale, icon: "square.grid.3x3",
                   help: "Integer scale: render at a whole-number multiple of the chain input (letterboxed), like RetroArch. Scanline and beam-shape parameters read much more clearly.")
            toggle($state.animatePreview, icon: "sparkles",
                   help: "Animate: advance the shader frame counter continuously (60 fps) so interlacing, tape noise, and animated artifacts actually move.")
                .disabled(state.exportInProgress)

            Divider().frame(height: 16)

            Image(systemName: "plus.magnifyingglass")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
            Slider(value: $state.zoom, in: 1.0...12.0)
                .controlSize(.small)
                .frame(width: 100)
                .help("Zoom the preview (also ⌥-scroll). Hold Space to pan when zoomed.")
            Text("\(Int((state.zoom * 100).rounded()))%")
                .font(.system(.caption2, design: .monospaced))
                .foregroundStyle(.secondary)
                .frame(width: 40, alignment: .trailing)

            Button {
                state.resetView()
            } label: {
                Image(systemName: "arrow.counterclockwise")
                    .help("Reset zoom and pan")
            }
            .buttonStyle(.borderless)
            .disabled(state.zoom == 1.0 && state.panX == 0 && state.panY == 0)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 10))
        .overlay(RoundedRectangle(cornerRadius: 10).strokeBorder(.white.opacity(0.08)))
    }

    /// Explicit monochrome on/off pill — the system button-toggle tint is
    /// indistinguishable from the off state on dark material with some
    /// accent colors (e.g. graphite).
    private func toggle(_ isOn: Binding<Bool>, icon: String, help: String) -> some View {
        Button {
            isOn.wrappedValue.toggle()
        } label: {
            Image(systemName: icon)
                .font(.system(size: 12, weight: .medium))
                .frame(width: 16, height: 14)
                .padding(.horizontal, 7)
                .padding(.vertical, 5)
                .foregroundStyle(isOn.wrappedValue ? Color.black.opacity(0.8) : .secondary)
                .background(
                    isOn.wrappedValue ? Color.white.opacity(0.85) : Color.white.opacity(0.07),
                    in: RoundedRectangle(cornerRadius: 6))
                .contentShape(RoundedRectangle(cornerRadius: 6))
                // .help on the label, not the plain-styled Button — tooltips
                // don't reliably fire when attached outside .buttonStyle(.plain).
                .help(help)
        }
        .buttonStyle(.plain)
    }
}
