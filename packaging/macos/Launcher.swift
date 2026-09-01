// The macOS application launcher.
//
// A .app bundle can only start one executable, and Rekall's executable is a server: double
// clicking it would start an HTTP listener and nothing visible. This process is what the user
// actually launches. It opens the window first, starts the server underneath it, and swaps the
// splash screen for the UI once the port answers, so the gap between the click and the first
// screen is a loading state rather than a bouncing dock icon.
//
// One launcher serves both builds. The flavour is decided by what the bundler put in
// Contents/: the GraalVM binary in Resources/, or a jlink runtime in runtime/ next to the jar.
// Nothing else in this file distinguishes them.

import AppKit
import WebKit

// MARK: - Where the server is

private enum Server {
    /// Kept in sync with SERVER_PORT in application.yaml. The MCP endpoint is registered with
    /// Claude Code as a fixed URL, so this is not a free port picked at startup: a different
    /// port would serve the UI and silently break `claude mcp add`'s registration.
    static let port: Int = Int(ProcessInfo.processInfo.environment["SERVER_PORT"] ?? "") ?? 47355
    static let host = "127.0.0.1"

    static var base: URL { URL(string: "http://\(host):\(port)/")! }
    static var health: URL { URL(string: "http://\(host):\(port)/actuator/health")! }

    /// The window waits this long for the port to answer before giving up. The first start of
    /// the JVM build runs Liquibase against an empty file, which is the slowest case by far.
    static let startupTimeout: TimeInterval = 120
}

// MARK: - Bundle layout

private enum Payload {
    case native(URL)
    case jvm(java: URL, jar: URL)

    /// Resources/rekall-app is the GraalVM binary, runtime/bin/java plus Resources/rekall-app.jar
    /// is the JVM build. Native is checked first so a bundle that somehow carried both would
    /// start the one that boots in a fraction of the time.
    static func detect(in bundle: Bundle) -> Payload? {
        let resources = bundle.bundleURL.appendingPathComponent("Contents/Resources")
        let binary = resources.appendingPathComponent("rekall-app")
        if FileManager.default.isExecutableFile(atPath: binary.path) {
            return .native(binary)
        }
        let java = bundle.bundleURL.appendingPathComponent("Contents/runtime/bin/java")
        let jar = resources.appendingPathComponent("rekall-app.jar")
        if FileManager.default.isExecutableFile(atPath: java.path),
           FileManager.default.isReadableFile(atPath: jar.path) {
            return .jvm(java: java, jar: jar)
        }
        return nil
    }
}

// MARK: - Application

@main
final class Launcher: NSObject, NSApplicationDelegate, WKNavigationDelegate, WKUIDelegate, WKDownloadDelegate {

    private var window: NSWindow!
    private var webView: WKWebView!

    /// nil when the launcher attached to a server that was already running (a `make run` in a
    /// terminal, or a second copy of the app). Quitting must then leave that server alone,
    /// which is exactly what "we did not start it, we do not stop it" gives for free here.
    private var server: Process?
    private var splashLoaded = false
    private var signalSources: [DispatchSourceSignal] = []
    private var folderPicker: FolderPicker!
    private var claudeCodeLauncher: ClaudeCodeLauncher!
    private var pendingStatus: String?
    private let logFile = FileManager.default.homeDirectoryForCurrentUser
        .appendingPathComponent("Library/Logs/Rekall/server.log")

    // NSApplication.delegate is a weak reference, so the delegate has to be owned by something
    // that outlives the run loop.
    private static let shared = Launcher()

    static func main() {
        let app = NSApplication.shared
        app.delegate = shared
        app.setActivationPolicy(.regular)
        app.run()
    }

    // MARK: Lifecycle

    func applicationDidFinishLaunching(_ notification: Notification) {
        trapSignals()
        buildMenu()
        buildWindow()
        showSplash()
        NSApp.activate(ignoringOtherApps: true)
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in self?.boot() }
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    /// ⌘Q and the red button both reach applicationWillTerminate, but `killall Rekall` and a
    /// logout do not: AppKit installs no handler for SIGTERM, so the default action would take
    /// this process down and leave the server it started running with nothing attached to it.
    private func trapSignals() {
        for number in [SIGTERM, SIGINT] {
            signal(number, SIG_IGN)
            let source = DispatchSource.makeSignalSource(signal: number, queue: .main)
            source.setEventHandler { NSApp.terminate(nil) }
            source.resume()
            signalSources.append(source)
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
        guard let server, server.isRunning else { return }
        // SIGTERM, not SIGKILL: server.shutdown is graceful, so this is what lets the H2 file be
        // closed properly instead of left with a lock file behind it.
        server.terminate()
        let deadline = Date().addingTimeInterval(10)
        while server.isRunning && Date() < deadline {
            usleep(50_000)
        }
        if server.isRunning {
            kill(server.processIdentifier, SIGKILL)
        }
    }

    // MARK: Startup sequence

    private func boot() {
        if serverIsAnswering() {
            status("Connecting to the running instance…")
            DispatchQueue.main.async { [weak self] in self?.openUI() }
            return
        }

        guard let payload = Payload.detect(in: Bundle.main) else {
            fail(title: "Nothing to start",
                 detail: "This bundle carries neither the native binary nor a Java runtime. "
                       + "It was assembled incorrectly: rebuild it with `make dmg-native` or `make dmg-jvm`.")
            return
        }

        status("Starting the server…")
        do {
            server = try spawn(payload)
        } catch {
            fail(title: "The server did not start", detail: error.localizedDescription)
            return
        }

        let startedAt = Date()
        let deadline = startedAt.addingTimeInterval(Server.startupTimeout)
        var slowNoticeShown = false
        while Date() < deadline {
            if let server, !server.isRunning {
                fail(title: "The server stopped while starting",
                     detail: "It exited with code \(server.terminationStatus). The most common cause is "
                           + "port \(Server.port) already being used by something that is not Rekall.")
                return
            }
            if serverIsAnswering() {
                DispatchQueue.main.async { [weak self] in self?.openUI() }
                return
            }
            if !slowNoticeShown && Date().timeIntervalSince(startedAt) > 15 {
                slowNoticeShown = true
                status("Still starting. The first run creates the database…")
            }
            usleep(250_000)
        }

        fail(title: "The server did not answer",
             detail: "Port \(Server.port) stayed silent for \(Int(Server.startupTimeout)) seconds. "
                   + "The log is at \(logFile.path).")
    }

    private func spawn(_ payload: Payload) throws -> Process {
        let process = Process()
        switch payload {
        case .native(let binary):
            process.executableURL = binary
        case .jvm(let java, let jar):
            process.executableURL = java
            process.arguments = ["-jar", jar.path]
        }

        // An .app is launched with the working directory set to /, and
        // DatabaseLocationEnvironmentPostProcessor adopts a ./data folder holding an H2 file when
        // ~/.rekall/config.json does not exist yet. Pointing that relative path at a directory of
        // our own keeps the adoption rule from resolving against whatever happens to sit at the
        // root of the disk, and gives the process somewhere writable to fall back on.
        let workingDirectory = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/Rekall")
        try FileManager.default.createDirectory(at: workingDirectory, withIntermediateDirectories: true)
        process.currentDirectoryURL = workingDirectory

        var environment = ProcessInfo.processInfo.environment
        environment["SERVER_PORT"] = String(Server.port)
        process.environment = environment

        // Without this the server's own diagnostics go to a pipe nobody reads, and a failure to
        // start would be reported here as a bare exit code with no way to find out why.
        try FileManager.default.createDirectory(at: logFile.deletingLastPathComponent(),
                                                withIntermediateDirectories: true)
        FileManager.default.createFile(atPath: logFile.path, contents: nil)
        if let handle = try? FileHandle(forWritingTo: logFile) {
            try? handle.truncate(atOffset: 0)
            process.standardOutput = handle
            process.standardError = handle
        }

        try process.run()
        return process
    }

    /// True when something that looks like Rekall answers. 200 is a healthy server, 503 is one
    /// that started but reports a component down (an unplugged database folder, which the UI has
    /// its own screen for). A 404 is some other application holding the port, and must not be
    /// mistaken for an instance to attach to.
    private func serverIsAnswering() -> Bool {
        var request = URLRequest(url: Server.health)
        request.timeoutInterval = 2
        request.cachePolicy = .reloadIgnoringLocalCacheData

        var code: Int?
        let done = DispatchSemaphore(value: 0)
        URLSession.shared.dataTask(with: request) { _, response, _ in
            code = (response as? HTTPURLResponse)?.statusCode
            done.signal()
        }.resume()
        _ = done.wait(timeout: .now() + 3)
        return code == 200 || code == 503
    }

    // MARK: Window

    private func buildWindow() {
        window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1440, height: 900),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false)
        window.title = "Rekall"
        window.titlebarAppearsTransparent = true
        window.minSize = NSSize(width: 960, height: 600)
        // The UI is dark first and paints its own background. Letting the window follow the
        // system theme would put a white titlebar and a white flash on top of it.
        window.appearance = NSAppearance(named: .darkAqua)
        window.backgroundColor = NSColor(srgbRed: 0.031, green: 0.035, blue: 0.047, alpha: 1)

        let configuration = WKWebViewConfiguration()
        configuration.suppressesIncrementalRendering = false
        // Both halves have to be in place before the web view is built: the configuration is
        // copied at initialisation, so a handler added afterwards is added to nothing.
        folderPicker = FolderPicker.install(into: configuration) { [weak self] in self?.window }
        claudeCodeLauncher = ClaudeCodeLauncher.install(into: configuration)
        webView = WKWebView(frame: .zero, configuration: configuration)
        webView.navigationDelegate = self
        webView.uiDelegate = self
        webView.allowsBackForwardNavigationGestures = false
        // Without this the web view paints white until the first frame of the page arrives, so
        // every launch would open on a white flash in front of a dark application.
        webView.underPageBackgroundColor = NSColor(srgbRed: 0.031, green: 0.035, blue: 0.047, alpha: 1)

        window.contentView = webView
        window.setFrameAutosaveName("RekallMainWindow")
        window.center()
        window.makeKeyAndOrderFront(nil)
    }

    private func openUI() {
        webView.load(URLRequest(url: Server.base))
    }

    // MARK: Splash and errors

    private func status(_ text: String) {
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            guard splashLoaded else { pendingStatus = text; return }
            let escaped = text.replacingOccurrences(of: "\\", with: "\\\\")
                              .replacingOccurrences(of: "\"", with: "\\\"")
            webView.evaluateJavaScript("window.rekallStatus && window.rekallStatus(\"\(escaped)\")")
        }
    }

    private func fail(title: String, detail: String) {
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            splashLoaded = false
            webView.loadHTMLString(Self.errorPage(title: title, detail: detail, log: logFile.path),
                                   baseURL: nil)
        }
    }

    private func showSplash() {
        webView.loadHTMLString(Self.splashPage(), baseURL: nil)
    }

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        guard webView.url == nil || webView.url?.scheme == "about" else { return }
        splashLoaded = true
        if let pendingStatus {
            self.pendingStatus = nil
            status(pendingStatus)
        }
    }

    // MARK: Navigation policy

    func webView(_ webView: WKWebView,
                 decidePolicyFor navigationAction: WKNavigationAction,
                 decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        guard let url = navigationAction.request.url, let host = url.host else {
            decisionHandler(.allow)
            return
        }
        if host == Server.host || host == "localhost" {
            decisionHandler(.allow)
        } else {
            // Anything not served by the local instance belongs in the browser, not in a window
            // with no address bar and no way back.
            NSWorkspace.shared.open(url)
            decisionHandler(.cancel)
        }
    }

    /// The export link is a plain href to /api/export, and a zip is not something a web view can
    /// display: without this it would answer the click by doing nothing at all.
    func webView(_ webView: WKWebView,
                 decidePolicyFor navigationResponse: WKNavigationResponse,
                 decisionHandler: @escaping (WKNavigationResponsePolicy) -> Void) {
        decisionHandler(navigationResponse.canShowMIMEType ? .allow : .download)
    }

    func webView(_ webView: WKWebView, navigationResponse: WKNavigationResponse, didBecome download: WKDownload) {
        download.delegate = self
    }

    func webView(_ webView: WKWebView, navigationAction: WKNavigationAction, didBecome download: WKDownload) {
        download.delegate = self
    }

    func webView(_ webView: WKWebView,
                 createWebViewWith configuration: WKWebViewConfiguration,
                 for navigationAction: WKNavigationAction,
                 windowFeatures: WKWindowFeatures) -> WKWebView? {
        if let url = navigationAction.request.url {
            NSWorkspace.shared.open(url)
        }
        return nil
    }

    // MARK: Downloads

    func download(_ download: WKDownload,
                  decideDestinationUsing response: URLResponse,
                  suggestedFilename: String,
                  completionHandler: @escaping (URL?) -> Void) {
        let downloads = FileManager.default.urls(for: .downloadsDirectory, in: .userDomainMask).first
            ?? FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Downloads")
        completionHandler(Self.unusedPath(in: downloads, named: suggestedFilename))
    }

    func downloadDidFinish(_ download: WKDownload) {
        guard let url = download.progress.fileURL else { return }
        NSWorkspace.shared.activateFileViewerSelecting([url])
    }

    func download(_ download: WKDownload, didFailWithError error: Error, resumeData: Data?) {
        let alert = NSAlert()
        alert.messageText = "The download failed"
        alert.informativeText = error.localizedDescription
        alert.runModal()
    }

    /// WKDownload refuses a destination that already exists, so exporting twice in a row would
    /// fail on the second attempt rather than write export-1.zip.
    private static func unusedPath(in directory: URL, named filename: String) -> URL {
        let candidate = directory.appendingPathComponent(filename)
        guard FileManager.default.fileExists(atPath: candidate.path) else { return candidate }
        let base = candidate.deletingPathExtension().lastPathComponent
        let ext = candidate.pathExtension
        for index in 1...999 {
            let name = ext.isEmpty ? "\(base)-\(index)" : "\(base)-\(index).\(ext)"
            let next = directory.appendingPathComponent(name)
            if !FileManager.default.fileExists(atPath: next.path) { return next }
        }
        return directory.appendingPathComponent("\(base)-\(UUID().uuidString).\(ext)")
    }

    // MARK: Menu

    /// Built in code because a bundle with no menu gets no ⌘Q, and, more importantly, no Edit
    /// menu: the standard cut/copy/paste selectors are what route ⌘C and ⌘V into the web view,
    /// so without them the note editor cannot paste.
    private func buildMenu() {
        let mainMenu = NSMenu()

        let appItem = NSMenuItem()
        let appMenu = NSMenu()
        appMenu.addItem(withTitle: "About Rekall", action: #selector(NSApplication.orderFrontStandardAboutPanel(_:)), keyEquivalent: "")
        appMenu.addItem(.separator())
        appMenu.addItem(withTitle: "Hide Rekall", action: #selector(NSApplication.hide(_:)), keyEquivalent: "h")
        appMenu.addItem(.separator())
        appMenu.addItem(withTitle: "Quit Rekall", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        appItem.submenu = appMenu
        mainMenu.addItem(appItem)

        let editItem = NSMenuItem()
        let editMenu = NSMenu(title: "Edit")
        editMenu.addItem(withTitle: "Undo", action: Selector(("undo:")), keyEquivalent: "z")
        editMenu.addItem(withTitle: "Redo", action: Selector(("redo:")), keyEquivalent: "Z")
        editMenu.addItem(.separator())
        editMenu.addItem(withTitle: "Cut", action: #selector(NSText.cut(_:)), keyEquivalent: "x")
        editMenu.addItem(withTitle: "Copy", action: #selector(NSText.copy(_:)), keyEquivalent: "c")
        editMenu.addItem(withTitle: "Paste", action: #selector(NSText.paste(_:)), keyEquivalent: "v")
        editMenu.addItem(withTitle: "Select All", action: #selector(NSText.selectAll(_:)), keyEquivalent: "a")
        editItem.submenu = editMenu
        mainMenu.addItem(editItem)

        let viewItem = NSMenuItem()
        let viewMenu = NSMenu(title: "View")
        viewMenu.addItem(withTitle: "Reload", action: #selector(reload), keyEquivalent: "r")
        viewMenu.addItem(withTitle: "Open Server Log", action: #selector(openLog), keyEquivalent: "")
        viewMenu.addItem(.separator())
        viewMenu.addItem(withTitle: "Enter Full Screen", action: #selector(NSWindow.toggleFullScreen(_:)), keyEquivalent: "f")
        viewItem.submenu = viewMenu
        mainMenu.addItem(viewItem)

        NSApp.mainMenu = mainMenu
    }

    @objc private func reload() {
        if webView.url?.host == Server.host {
            webView.reload()
        } else {
            showSplash()
            DispatchQueue.global(qos: .userInitiated).async { [weak self] in self?.boot() }
        }
    }

    @objc private func openLog() {
        NSWorkspace.shared.open(logFile)
    }

    // MARK: Pages
    //
    // Inline rather than resource files: the palette is three colours copied from the UI's own
    // tokens, and a splash screen that has to be loaded from disk is one more thing that can be
    // missing from the bundle at the exact moment it is needed to report that something is.

    private static func chrome() -> String {
        """
        <meta charset="utf-8">
        <style>
          :root { color-scheme: dark; }
          html, body { height: 100%; margin: 0; }
          body {
            background: #08090c; color: #f2f5f9;
            font: 400 14px/1.5 -apple-system, BlinkMacSystemFont, 'Fira Sans', sans-serif;
            display: flex; align-items: center; justify-content: center;
            -webkit-user-select: none; user-select: none;
          }
          .panel { text-align: center; max-width: 34rem; padding: 2rem; }
          .mark {
            font-size: 0.75rem; letter-spacing: 0.42em; text-transform: uppercase;
            color: #f5a524; margin-bottom: 2rem;
          }
          .status { color: #a3acba; min-height: 1.5rem; }
          .detail { color: #7a8494; font-size: 12.5px; margin-top: 0.75rem; }
          .title { font-size: 1rem; font-weight: 500; margin-bottom: 0.5rem; }
          .path { font-family: 'Fira Code', ui-monospace, monospace; color: #5fd0e0; font-size: 12px; }
          .spinner {
            width: 22px; height: 22px; margin: 0 auto 1.5rem;
            border: 2px solid #1c222c; border-top-color: #f5a524; border-radius: 50%;
            animation: spin 0.7s linear infinite;
          }
          @keyframes spin { to { transform: rotate(360deg); } }
        </style>
        """
    }

    private static func splashPage() -> String {
        """
        <!doctype html><html><head>\(chrome())</head>
        <body>
          <div class="panel">
            <div class="mark">Rekall</div>
            <div class="spinner"></div>
            <div class="status" id="status">Starting…</div>
          </div>
          <script>
            window.rekallStatus = function (text) {
              document.getElementById('status').textContent = text;
            };
          </script>
        </body></html>
        """
    }

    private static func errorPage(title: String, detail: String, log: String) -> String {
        func escape(_ value: String) -> String {
            value.replacingOccurrences(of: "&", with: "&amp;")
                 .replacingOccurrences(of: "<", with: "&lt;")
                 .replacingOccurrences(of: ">", with: "&gt;")
        }
        return """
        <!doctype html><html><head>\(chrome())</head>
        <body>
          <div class="panel">
            <div class="mark">Rekall</div>
            <div class="title">\(escape(title))</div>
            <div class="detail">\(escape(detail))</div>
            <div class="detail">Log: <span class="path">\(escape(log))</span></div>
            <div class="detail">View &rsaquo; Reload tries again.</div>
          </div>
        </body></html>
        """
    }
}
