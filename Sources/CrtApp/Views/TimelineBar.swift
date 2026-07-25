import SwiftUI

/// Keyframe timeline docked under the preview (image sources, timeline mode
/// on): scrub the playhead, snapshot master keyframes with the stopwatch,
/// drag diamonds to retime, right-click a diamond for easing / delete.
/// Keyframe times are normalized, so the duration field rescales the whole
/// animation proportionally.
struct TimelineBar: View {
    @Environment(AppState.self) private var state

    var body: some View {
        @Bindable var state = state
        HStack(spacing: 10) {
            Button {
                state.toggleTimelinePreview()
            } label: {
                Image(systemName: state.timelinePlaying ? "pause.fill" : "play.fill")
                    .font(.system(size: 13))
                    .frame(width: 18)
                    .help(state.timelinePlaying ? "Pause"
                          : "Preview the keyframe animation in the preview (loops)")
            }
            .buttonStyle(.borderless)
            .disabled(state.exportInProgress || state.timelineKeys.isEmpty)

            Button {
                state.setKeyframeAtPlayhead()
            } label: {
                Image(systemName: "stopwatch")
                    .font(.system(size: 13))
                    .frame(width: 18)
                    .help("Set a keyframe: snapshot every VHS + shader parameter at the playhead (updates the keyframe under the playhead). Nothing is keyed until you press this.")
            }
            .buttonStyle(.borderless)

            ruler

            Text(String(format: "%.2fs", state.playheadT * state.timelineDuration))
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.secondary)
                .frame(width: 52, alignment: .trailing)

            HStack(spacing: 4) {
                NumericField(value: $state.timelineDuration, range: 0.5...600, width: 44)
                Text("s").font(.caption).foregroundStyle(.secondary)
            }
            .help("Video duration. Keyframes are proportional — changing the duration stretches the whole animation.")

            Picker("", selection: $state.timelineFPS) {
                Text("24 fps").tag(24)
                Text("30 fps").tag(30)
                Text("60 fps").tag(60)
            }
            .labelsHidden()
            .pickerStyle(.menu)
            .fixedSize()
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.bar)
    }

    private var ruler: some View {
        GeometryReader { geo in
            let w = max(1, geo.size.width)
            ZStack(alignment: .leading) {
                // track
                RoundedRectangle(cornerRadius: 2)
                    .fill(Color.primary.opacity(0.15))
                    .frame(height: 4)
                    .frame(maxHeight: .infinity, alignment: .center)

                if state.timelineKeys.isEmpty {
                    Text("Dial in a look, then press the stopwatch to keyframe it")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, alignment: .center)
                        .allowsHitTesting(false)
                }

                ForEach(state.timelineKeys) { key in
                    diamond(for: key, width: w)
                }

                // playhead
                RoundedRectangle(cornerRadius: 1)
                    .fill(Color.red)
                    .frame(width: 2)
                    .offset(x: CGFloat(state.playheadT) * w - 1)
                    .allowsHitTesting(false)
            }
            .contentShape(Rectangle())
            .coordinateSpace(name: "ruler")
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { v in
                        state.scrubTimeline(to: Double(v.location.x / w))
                    }
            )
        }
        .frame(height: 22)
        .frame(maxWidth: .infinity)
    }

    private func diamond(for key: Keyframe, width: CGFloat) -> some View {
        Image(systemName: "diamond.fill")
            .font(.system(size: 11))
            .foregroundStyle(Color.accentColor)
            .frame(width: 18, height: 22)          // fat hit target
            .contentShape(Rectangle())
            .offset(x: CGFloat(key.t) * width - 9)
            .gesture(
                DragGesture(minimumDistance: 2, coordinateSpace: .named("ruler"))
                    .onChanged { v in
                        state.moveKeyframe(id: key.id, to: Double(v.location.x / width))
                    }
            )
            .onTapGesture {
                state.scrubTimeline(to: key.t)
            }
            .contextMenu {
                Picker("Interpolation", selection: Binding(
                    get: { key.easing },
                    set: { state.setKeyframeEasing(id: key.id, $0) }
                )) {
                    ForEach(KeyEasing.allCases, id: \.self) { e in
                        Text(e.rawValue).tag(e)
                    }
                }
                Divider()
                Button("Delete Keyframe", role: .destructive) {
                    state.deleteKeyframe(id: key.id)
                }
            }
            .help("Keyframe at \(String(format: "%.2fs", key.t * state.timelineDuration)) — drag to move, right-click for interpolation")
    }
}
