import SwiftUI

/// Full-width video transport docked under the preview: play/pause, a frame
/// scrubber spanning the pane (maximum scrubbing precision), and a
/// frame/time/fps readout. Rendered only when a video source is loaded.
struct TransportBar: View {
    @Environment(AppState.self) private var state

    var body: some View {
        if let vs = state.videoSource {
            let total = vs.totalFrames
            let idx = Binding<Double>(
                get: { Double(state.currentFrameIndex) },
                set: { state.currentFrameIndex = Int($0.rounded()) }
            )
            HStack(spacing: 10) {
                Button {
                    state.togglePlayback()
                } label: {
                    Image(systemName: state.videoPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 13))
                        .frame(width: 18)
                }
                .buttonStyle(.borderless)
                .disabled(state.exportInProgress)
                .help(state.videoPlaying ? "Pause" : "Play the video in the preview (with all effects applied)")

                Slider(value: idx, in: 0...Double(max(1, total - 1)), step: 1)

                Text("\(state.currentFrameIndex + 1)/\(total)  ·  \(String(format: "%.2fs", Double(state.currentFrameIndex) / Double(max(1, vs.frameRate))))  ·  \(String(format: "%.0f", vs.frameRate)) fps")
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .fixedSize()
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(.bar)
        }
    }
}
