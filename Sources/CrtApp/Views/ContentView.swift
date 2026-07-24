import SwiftUI
import AppKit
import UniformTypeIdentifiers
import CrtCore

struct ContentView: View {
    @Environment(AppState.self) private var state
    // CRT_SHOW_EXPORT=1 / CRT_PALETTE_FADE=<seconds>: dev hooks for
    // screenshot verification (see DEVELOPMENT.md).
    @State private var showExport =
        ProcessInfo.processInfo.environment["CRT_SHOW_EXPORT"] == "1"
    private let paletteFadeSeconds =
        ProcessInfo.processInfo.environment["CRT_PALETTE_FADE"].flatMap(Double.init) ?? 2.0
    @State private var paletteVisible = true
    @State private var palettePinned = false   // pointer is over the palette itself
    /// Fade bookkeeping lives in a plain class so per-mouse-move updates
    /// don't invalidate the view body.
    @State private var fade = FadeTimer()

    private final class FadeTimer {
        var task: Task<Void, Never>?
        /// The pointer can start outside the window, which delivers an
        /// immediate `.ended` — don't hide until the user has hovered once,
        /// so the palette is discoverable at launch.
        var hasHovered = false
    }

    var body: some View {
        NavigationSplitView {
            Sidebar()
                .navigationSplitViewColumnWidth(min: 280, ideal: 320, max: 400)
        } detail: {
            VStack(spacing: 0) {
                // Preserve source aspect ratio: PreviewView gets a frame
                // matching the source's aspect, centred in the available
                // space. The view palette floats over the letterbox area and
                // fades out when the pointer goes idle.
                ZStack {
                    Color(white: 0.04)
                    PreviewView()
                        .aspectRatio(state.sourceAspect, contentMode: .fit)
                        .padding(8)
                }
                .overlay(alignment: .bottom) {
                    ViewPalette()
                        // The palette floats over the near-black canvas in
                        // both system modes; its colors assume dark.
                        .environment(\.colorScheme, .dark)
                        .padding(.bottom, 12)
                        .opacity(paletteVisible || palettePinned ? 1 : 0)
                        .animation(.easeInOut(duration: 0.25),
                                   value: paletteVisible || palettePinned)
                        .onHover { palettePinned = $0 }
                }
                .onContinuousHover { phase in
                    switch phase {
                    case .active:
                        fade.hasHovered = true
                        paletteVisible = true
                        fade.task?.cancel()
                        fade.task = Task { @MainActor in
                            try? await Task.sleep(for: .seconds(paletteFadeSeconds))
                            if !Task.isCancelled { paletteVisible = false }
                        }
                    case .ended:
                        fade.task?.cancel()
                        if fade.hasHovered { paletteVisible = false }
                    }
                }

                TransportBar()
            }
            .frame(minWidth: 480, minHeight: 360)
        }
        .frame(minWidth: 1000, minHeight: 640)
        .toolbar { toolbarContent }
    }

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItem(placement: .navigation) {
            Button {
                openMedia()
            } label: {
                Label("Open…", systemImage: "folder")
            }
            .keyboardShortcut("o")
            .help("Open an image (PNG/JPEG/HEIC) or video (MP4/MOV) — or drop one on the Source panel")
        }

        ToolbarItemGroup(placement: .primaryAction) {
            Menu {
                Button("Save Preset…") { savePreset() }
                Button("Load Preset…") { loadPreset() }
            } label: {
                Label("Preset", systemImage: "doc.badge.gearshape")
                    .labelStyle(.titleAndIcon)
            }
            .help("Save or load the whole configuration (downscale + VHS + shader + view) as a JSON file")

            Button {
                showExport.toggle()
            } label: {
                if state.exportWorking && state.videoSource != nil {
                    Label("\(Int((state.exportProgress * 100).rounded()))%",
                          systemImage: "square.and.arrow.up")
                        .labelStyle(.titleAndIcon)
                } else {
                    Label("Export…", systemImage: "square.and.arrow.up")
                        .labelStyle(.titleAndIcon)
                }
            }
            .keyboardShortcut("e")
            .popover(isPresented: $showExport, arrowEdge: .bottom) {
                ExportPopover()
            }
            .help("Export the current frame as PNG, or the whole video as H.264/HEVC/ProRes")
        }
    }

    // MARK: - toolbar actions

    private func openMedia() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.image, .movie, .mpeg4Movie, .quickTimeMovie, .png, .jpeg, .heic]
        panel.allowsMultipleSelection = false
        if panel.runModal() == .OK, let url = panel.url {
            state.sourceURL = url
        }
    }

    private var presetTimestamp: String {
        let f = DateFormatter()
        f.dateFormat = "dd-MM-yy HH.mm.ss"
        return f.string(from: Date())
    }

    private func savePreset() {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "ntscrt preset \(presetTimestamp).json"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try state.saveLook(to: url)
        } catch {
            presetAlert("Couldn't save the preset.", error)
        }
    }

    private func loadPreset() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.json]
        panel.allowsMultipleSelection = false
        guard panel.runModal() == .OK, let url = panel.url else { return }
        do {
            try state.loadLook(from: url)
        } catch {
            presetAlert("Couldn't load the preset.", error)
        }
    }

    private func presetAlert(_ message: String, _ error: Error) {
        let alert = NSAlert()
        alert.messageText = message
        alert.informativeText = error.localizedDescription
        alert.alertStyle = .warning
        alert.runModal()
    }
}

private struct Sidebar: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                SourcePanel()
                Divider()
                DownscalePanel()
                Divider()
                NtscPanel()
                Divider()
                ShaderPanel()
            }
            .padding(16)
        }
    }
}
