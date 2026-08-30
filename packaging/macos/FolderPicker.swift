// The bridge between the page and the one native dialog it needs.
//
// Its own file, and installed through one call, so the same wiring the application ships can be
// built into a bare WKWebView and exercised without a bundle, a window or a server behind it.

import AppKit
import WebKit

// MARK: - The folder chooser

/// The one thing the window can do that a browser tab cannot.
///
/// A web page is never handed a real filesystem path, which is why the database folder is typed
/// rather than browsed everywhere else. Inside the app there is a real NSOpenPanel to open, so
/// the field's folder icon becomes a button and the path it produces is the same absolute path
/// the server is about to validate.
///
/// Its own object rather than another role for the launcher: WKUserContentController retains its
/// message handlers, and the launcher owns the web view that owns the controller.
final class FolderPicker: NSObject, WKScriptMessageHandlerWithReply {

    static let messageName = "rekallPickFolder"

    /// The JavaScript half, installed before the page runs. It is deliberately the smallest
    /// possible surface: one function, returning the promise postMessage already gives back, so
    /// the page has nothing to poll and no callback to register.
    static let bridge = """
        window.rekallDesktop = {
            pickFolder: function (currentPath) {
                return window.webkit.messageHandlers.\(messageName).postMessage({
                    path: typeof currentPath === 'string' ? currentPath : ''
                });
            }
        };
        """

    private let hostWindow: () -> NSWindow?

    init(hostWindow: @escaping () -> NSWindow?) {
        self.hostWindow = hostWindow
    }

    /// Both halves in one call, because they are useless apart: the script names a handler that
    /// has to be registered, and the handler answers a message nothing else sends. The caller
    /// keeps the returned object only to own it - the configuration retains it either way.
    @discardableResult
    static func install(into configuration: WKWebViewConfiguration,
                        hostWindow: @escaping () -> NSWindow?) -> FolderPicker {
        let picker = FolderPicker(hostWindow: hostWindow)
        configuration.userContentController.addScriptMessageHandler(
            picker, contentWorld: .page, name: messageName)
        configuration.userContentController.addUserScript(
            WKUserScript(source: bridge, injectionTime: .atDocumentStart, forMainFrameOnly: true))
        return picker
    }

    func userContentController(_ controller: WKUserContentController,
                               didReceive message: WKScriptMessage,
                               replyHandler: @escaping (Any?, String?) -> Void) {
        guard message.name == Self.messageName else {
            replyHandler(nil, "Unknown message \(message.name)")
            return
        }

        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.allowsMultipleSelection = false
        // The server refuses a folder that does not exist, on purpose: it never creates one
        // itself. Letting the panel create it is what keeps that rule from turning into a
        // detour through Finder.
        panel.canCreateDirectories = true
        panel.prompt = "Use this folder"
        panel.message = "Choose the folder that holds the Rekall database"

        if let body = message.body as? [String: Any],
           let typed = body["path"] as? String, !typed.isEmpty {
            let expanded = (typed as NSString).expandingTildeInPath
            var isDirectory: ObjCBool = false
            if FileManager.default.fileExists(atPath: expanded, isDirectory: &isDirectory), isDirectory.boolValue {
                panel.directoryURL = URL(fileURLWithPath: expanded)
            }
        }

        // A sheet, not a free-floating panel: this belongs to the field that opened it, and a
        // modal run outside the window would let the page be edited underneath it.
        guard let window = hostWindow() else {
            replyHandler(nil, "No window to attach the panel to")
            return
        }
        panel.beginSheetModal(for: window) { response in
            replyHandler(response == .OK ? panel.url?.path : nil, nil)
        }
    }
}
