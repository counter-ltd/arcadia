import AppKit
import Foundation

@main
final class ArcadiaDevelopmentLauncher: NSObject, NSApplicationDelegate, NSMenuDelegate {
    private static let menuRefreshInterval: TimeInterval = 1.5

    private let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    private let repositoryRoot = URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent("Arcadia")
    private let processQueue = DispatchQueue(label: "arcadia.development-launcher.process")

    private var startItem: NSMenuItem!
    private var restartItem: NSMenuItem!
    private var stopItem: NSMenuItem!
    private var quitItem: NSMenuItem!
    private var statusMenu: NSMenu!
    private var launchedProcess: Process?
    private var refreshTimer: Timer?

    static func main() {
        let app = NSApplication.shared
        let delegate = ArcadiaDevelopmentLauncher()
        app.delegate = delegate
        app.setActivationPolicy(.accessory)
        app.run()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        configureStatusItem()
        refreshMenuState()
        scheduleMenuRefreshTimer()
    }

    func applicationWillTerminate(_ notification: Notification) {
        invalidateMenuRefreshTimer()
        let snapshot = launchedProcess
        if let snapshot, snapshot.isRunning {
            snapshot.terminate()
            waitForSnapshotToExit(snapshot, timeout: 4)
        }
        launchedProcess = nil
        processQueue.sync { [weak self] in
            guard let self else { return }
            for pid in self.arcadiaPIDs() {
                Darwin.kill(pid, SIGTERM)
            }
        }
    }

    @objc private func startArcadia() {
        if launchedProcess?.isRunning == true {
            refreshMenuState()
            return
        }

        processQueue.async { [weak self] in
            guard let self else { return }
            guard self.arcadiaPIDs().isEmpty else {
                DispatchQueue.main.async { self.refreshMenuState() }
                return
            }
            DispatchQueue.main.async { [weak self] in
                self?.launchArcadiaProcessIfStillNeeded()
            }
        }
    }

    /// Main thread — worker already confirmed no matching `arcadia` PIDs.
    private func launchArcadiaProcessIfStillNeeded() {
        if launchedProcess?.isRunning == true {
            refreshMenuState()
            return
        }

        let process = makeArcadiaProcess()
        let logHandle = attachLogFile(to: process)
        process.terminationHandler = { [weak self, weak process] _ in
            logHandle?.closeFile()
            DispatchQueue.main.async {
                if self?.launchedProcess === process {
                    self?.launchedProcess = nil
                }
                self?.refreshMenuState()
            }
        }

        do {
            try process.run()
            launchedProcess = process
        } catch {
            logHandle?.closeFile()
            showLaunchError(error)
        }

        refreshMenuState()
    }

    @objc private func restartArcadia() {
        if launchedProcess?.isRunning == true {
            stopArcadia(wait: true)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { [weak self] in
                self?.startArcadia()
            }
            return
        }

        processQueue.async { [weak self] in
            guard let self else { return }
            guard !self.arcadiaPIDs().isEmpty else {
                DispatchQueue.main.async { self.refreshMenuState() }
                return
            }
            DispatchQueue.main.async {
                self.stopArcadia(wait: true)
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    self.startArcadia()
                }
            }
        }
    }

    @objc private func stopArcadiaFromMenu() {
        stopArcadia(wait: false)
    }

    @objc private func quitLauncher() {
        invalidateMenuRefreshTimer()
        let snapshot = launchedProcess
        launchedProcess = nil
        snapshot?.terminate()

        processQueue.async { [weak self] in
            guard let self else {
                DispatchQueue.main.async { NSApp.terminate(nil) }
                return
            }
            for pid in self.arcadiaPIDs() {
                Darwin.kill(pid, SIGTERM)
            }
            DispatchQueue.main.async {
                NSApp.terminate(nil)
            }
        }
    }

    func menuWillOpen(_ menu: NSMenu) {
        invalidateMenuRefreshTimer()
        refreshMenuState()
    }

    func menuDidClose(_ menu: NSMenu) {
        scheduleMenuRefreshTimer()
    }

    private func configureStatusItem() {
        statusItem.length = NSStatusItem.squareLength
        if let button = statusItem.button {
            button.title = ""
            button.image = statusIcon()
            button.imagePosition = .imageOnly
            button.toolTip = "Arcadia Development Launcher"
        }

        let menu = NSMenu()
        menu.delegate = self
        menu.autoenablesItems = false
        startItem = NSMenuItem(title: "Start", action: #selector(startArcadia), keyEquivalent: "")
        restartItem = NSMenuItem(title: "Restart", action: #selector(restartArcadia), keyEquivalent: "")
        stopItem = NSMenuItem(title: "Stop", action: #selector(stopArcadiaFromMenu), keyEquivalent: "")
        quitItem = NSMenuItem(title: "Quit", action: #selector(quitLauncher), keyEquivalent: "q")

        for item in [startItem, restartItem, stopItem, quitItem] {
            item?.target = self
            menu.addItem(item!)
        }

        statusMenu = menu
        statusItem.menu = statusMenu
    }

    private func scheduleMenuRefreshTimer() {
        invalidateMenuRefreshTimer()
        let timer = Timer(timeInterval: Self.menuRefreshInterval, repeats: true) { [weak self] _ in
            self?.refreshMenuState()
        }
        RunLoop.main.add(timer, forMode: .common)
        refreshTimer = timer
    }

    private func invalidateMenuRefreshTimer() {
        refreshTimer?.invalidate()
        refreshTimer = nil
    }

    private func statusIcon() -> NSImage {
        let side: CGFloat = 18
        if let url = Bundle.main.url(forResource: "StatusIcon", withExtension: "png"),
           let source = NSImage(contentsOf: url) {
            let image = NSImage(size: NSSize(width: side, height: side), flipped: false) { bounds in
                source.draw(
                    in: bounds,
                    from: NSRect(origin: .zero, size: source.size),
                    operation: .copy,
                    fraction: 1.0
                )
                return true
            }
            image.isTemplate = false
            return image
        }

        if let symbol = NSImage(systemSymbolName: "hammer.fill", accessibilityDescription: "Arcadia Launcher") {
            let sized = symbol.withSymbolConfiguration(
                NSImage.SymbolConfiguration(pointSize: side - 2, weight: .regular)
            ) ?? symbol
            sized.isTemplate = true
            return sized
        }

        if let appIcon = NSApp.applicationIconImage.copy() as? NSImage {
            appIcon.size = NSSize(width: side, height: side)
            appIcon.isTemplate = false
            return appIcon
        }

        return minimalMenuBarPlaceholderIcon(side: side)
    }

    private func minimalMenuBarPlaceholderIcon(side: CGFloat) -> NSImage {
        NSImage(size: NSSize(width: side, height: side), flipped: false) { rect in
            NSColor.controlAccentColor.setFill()
            NSBezierPath(ovalIn: rect.insetBy(dx: 4, dy: 4)).fill()
            return true
        }
    }

    /// Menu updates only — never runs subprocesses on the main thread.
    private func refreshMenuState() {
        if let process = launchedProcess, process.isRunning {
            applyMenuState(running: true)
            return
        }

        processQueue.async { [weak self] in
            guard let self else { return }
            let running = !self.arcadiaPIDs().isEmpty
            DispatchQueue.main.async {
                self.applyMenuState(running: running)
            }
        }
    }

    private func applyMenuState(running: Bool) {
        startItem?.isEnabled = !running
        restartItem?.isEnabled = running
        stopItem?.isEnabled = running
        quitItem?.isEnabled = true
    }

    private func makeArcadiaProcess() -> Process {
        let process = Process()
        let globalCommand = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".local/bin/arcadia-gui")

        if FileManager.default.isExecutableFile(atPath: globalCommand.path) {
            process.executableURL = globalCommand
            process.arguments = []
        } else {
            process.executableURL = URL(fileURLWithPath: "/bin/bash")
            process.arguments = [
                "-lc",
                """
                export PATH="${HOME}/.cargo/bin:${PATH}"
                cd "\(repositoryRoot.path)"
                cargo build --manifest-path Desktop/Cargo.toml --no-default-features --features gui >/dev/null
                exec "\(repositoryRoot.path)/Builds/workspace/debug/arcadia"
                """
            ]
        }

        process.currentDirectoryURL = repositoryRoot
        process.environment = processEnvironment()
        return process
    }

    private func processEnvironment() -> [String: String] {
        var environment = ProcessInfo.processInfo.environment
        let cargoBin = URL(fileURLWithPath: NSHomeDirectory()).appendingPathComponent(".cargo/bin").path
        let existingPath = environment["PATH"] ?? "/usr/bin:/bin:/usr/sbin:/sbin"
        environment["PATH"] = "\(cargoBin):\(existingPath)"
        return environment
    }

    /// Redirect stdout/stderr to log file. Close in `terminationHandler` (or immediately if `run()` fails); do **not** close right after `run()` — breaks child I/O.
    private func attachLogFile(to process: Process) -> FileHandle? {
        let logsDirectory = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent("Library/Logs/Arcadia")
        let logFile = logsDirectory.appendingPathComponent("DevelopmentLauncher.log")

        do {
            try FileManager.default.createDirectory(
                at: logsDirectory,
                withIntermediateDirectories: true
            )
            if !FileManager.default.fileExists(atPath: logFile.path) {
                FileManager.default.createFile(atPath: logFile.path, contents: nil)
            }
            let handle = try FileHandle(forWritingTo: logFile)
            try handle.seekToEnd()
            process.standardOutput = handle
            process.standardError = handle
            return handle
        } catch {
            process.standardOutput = FileHandle.standardOutput
            process.standardError = FileHandle.standardError
            return nil
        }
    }

    private func stopArcadia(wait: Bool) {
        let snapshot = launchedProcess
        processQueue.async { [weak self] in
            self?.stopArcadiaWorkEntry(snapshotProcess: snapshot, wait: wait, refreshUI: true)
        }
    }

    /// Runs only on `processQueue`. Never call from main without dispatching.
    private func stopArcadiaWorkEntry(snapshotProcess: Process?, wait: Bool, refreshUI: Bool) {
        let pids = arcadiaPIDs()
        if let snapshotProcess, snapshotProcess.isRunning {
            snapshotProcess.terminate()
            if wait {
                waitForSnapshotToExit(snapshotProcess, timeout: 12)
            }
        }

        for pid in pids {
            Darwin.kill(pid, SIGTERM)
        }

        if wait {
            waitForArcadiaToExit(timeout: 5.0)
        }

        if refreshUI {
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                if self.launchedProcess?.isRunning == false {
                    self.launchedProcess = nil
                }
                self.refreshMenuState()
            }
        }
    }

    private func waitForSnapshotToExit(_ process: Process, timeout: TimeInterval) {
        let deadline = Date().addingTimeInterval(timeout)
        while process.isRunning && Date() < deadline {
            Thread.sleep(forTimeInterval: 0.05)
        }
    }

    private func waitForArcadiaToExit(timeout: TimeInterval) {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if arcadiaPIDs().isEmpty {
                return
            }
            Thread.sleep(forTimeInterval: 0.1)
        }
    }

    private func arcadiaPIDs() -> [pid_t] {
        let process = Process()
        let pipe = Pipe()

        process.executableURL = URL(fileURLWithPath: "/bin/ps")
        process.arguments = ["-axo", "pid=,command="]
        process.standardOutput = pipe
        process.standardError = Pipe()

        do {
            try process.run()
        } catch {
            return []
        }

        process.waitUntilExit()

        let output = pipe.fileHandleForReading.readDataToEndOfFile()
        guard let text = String(data: output, encoding: .utf8) else {
            return []
        }

        let currentPID = ProcessInfo.processInfo.processIdentifier
        let binaryPath = "\(repositoryRoot.path)/Builds/workspace/debug/arcadia"

        return text.split(separator: "\n").compactMap { line -> pid_t? in
            let trimmed = String(line).trimmingCharacters(in: .whitespaces)
            guard let firstSpace = trimmed.firstIndex(where: { $0 == " " || $0 == "\t" }) else {
                return nil
            }

            let pidText = String(trimmed[..<firstSpace])
            let command = String(trimmed[firstSpace...]).trimmingCharacters(in: .whitespaces)

            guard let pid = pid_t(pidText), pid != currentPID else {
                return nil
            }

            if command == binaryPath || command.hasPrefix("\(binaryPath) ") {
                return pid
            }

            return nil
        }
    }

    private func showLaunchError(_ error: Error) {
        let alert = NSAlert()
        alert.messageText = "Arcadia could not be started."
        alert.informativeText = error.localizedDescription
        alert.alertStyle = .warning
        alert.runModal()
    }
}
