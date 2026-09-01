// The bridge between the page and a terminal running Claude Code on one anchor.
//
// Its own file for the same reason FolderPicker is: the wiring is installed with one call and
// can be built into a bare WKWebView, with no bundle, window or server behind it.

import AppKit
import WebKit

/// Opens a terminal that is already loading a working context.
///
/// The button this answers is the shortest path there is between a task in Rekall and a session
/// that has read it: no copy, no paste, no `cd`. What makes that safe to offer is that the page
/// never names a command. It sends a folder, an anchor and one flag, and every one of the three
/// is checked here before a line of shell is written. A note rendered in this window is markdown
/// someone else may have written, and it is one XSS away from being the caller.
///
/// The terminal is launched by opening a script with it rather than by scripting the terminal
/// itself. AppleEvents would put a "Rekall wants to control Terminal" prompt in front of a button
/// whose whole point is that it is one click, and would fail silently for anyone who says no.
final class ClaudeCodeLauncher: NSObject, WKScriptMessageHandlerWithReply {

    static let messageName = "rekallOpenInClaudeCode"

    /// What an anchor is allowed to contain: what `entity:value` needs and nothing that means
    /// anything to a shell. Everything else is refused rather than escaped, because refusing is
    /// a rule that can be read and escaping is a rule that has to be trusted.
    private static let allowedAnchorCharacters = CharacterSet(
        charactersIn: "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789:._- ")

    private static let anchorLimit = 300

    /// Where Claude Code installs itself, tried before anything on PATH.
    private static let homeRelativeBinaries = [".local/bin/claude", ".claude/local/claude", "bin/claude"]

    private static let knownDirectories = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"]

    /// The JavaScript half. It merges rather than assigns: the folder chooser installs a script
    /// of its own onto the same object, and whichever runs second would otherwise erase the first.
    static let bridge = """
        window.rekallDesktop = Object.assign(window.rekallDesktop || {}, {
            openInClaudeCode: function (launch) {
                return window.webkit.messageHandlers.\(messageName).postMessage({
                    directory: String((launch && launch.directory) || ''),
                    anchors: String((launch && launch.anchors) || ''),
                    skipPermissions: Boolean(launch && launch.skipPermissions)
                });
            }
        });
        """

    @discardableResult
    static func install(into configuration: WKWebViewConfiguration) -> ClaudeCodeLauncher {
        let launcher = ClaudeCodeLauncher()
        configuration.userContentController.addScriptMessageHandler(
            launcher, contentWorld: .page, name: messageName)
        configuration.userContentController.addUserScript(
            WKUserScript(source: bridge, injectionTime: .atDocumentStart, forMainFrameOnly: true))
        return launcher
    }

    func userContentController(_ controller: WKUserContentController,
                               didReceive message: WKScriptMessage,
                               replyHandler: @escaping (Any?, String?) -> Void) {
        guard message.name == Self.messageName else {
            replyHandler(nil, "Unknown message \(message.name)")
            return
        }
        guard let body = message.body as? [String: Any],
              let directory = body["directory"] as? String,
              let anchors = body["anchors"] as? String else {
            replyHandler(nil, "The launch was not understood")
            return
        }

        let expanded = (directory as NSString).expandingTildeInPath
        var isDirectory: ObjCBool = false
        guard !expanded.isEmpty,
              FileManager.default.fileExists(atPath: expanded, isDirectory: &isDirectory),
              isDirectory.boolValue else {
            replyHandler(nil, "\(directory.isEmpty ? "No folder" : directory) is not a folder on this machine")
            return
        }
        guard Self.isSafeAnchor(anchors) else {
            replyHandler(nil, "That anchor has characters an anchor cannot have")
            return
        }

        let script = Self.script(directory: expanded,
                                anchors: anchors,
                                skipPermissions: body["skipPermissions"] as? Bool ?? false)
        do {
            let file = try Self.write(script)
            let terminal = Self.terminal()
            let configuration = NSWorkspace.OpenConfiguration()
            configuration.activates = true
            NSWorkspace.shared.open([file], withApplicationAt: terminal, configuration: configuration) { _, error in
                DispatchQueue.main.async {
                    if let error {
                        replyHandler(nil, error.localizedDescription)
                    } else {
                        replyHandler(Self.name(of: terminal), nil)
                    }
                }
            }
        } catch {
            replyHandler(nil, error.localizedDescription)
        }
    }

    // MARK: The command

    static func isSafeAnchor(_ anchors: String) -> Bool {
        let trimmed = anchors.trimmingCharacters(in: .whitespaces)
        return !trimmed.isEmpty
            && trimmed.count <= anchorLimit
            && trimmed.unicodeScalars.allSatisfy(allowedAnchorCharacters.contains)
    }

    /// The whole session in four lines.
    ///
    /// It removes itself first: the shell already holds the file open, so unlinking it leaves the
    /// running script alone and leaves nothing behind in the temporary directory either. `exec`
    /// puts Claude Code in the shell's place, so closing the window ends the session and nothing
    /// else.
    static func script(directory: String, anchors: String, skipPermissions: Bool) -> String {
        let flag = skipPermissions ? " --dangerously-skip-permissions" : ""
        let command = quote(claudeBinary()) + flag + " " + quote("/rk " + anchors.trimmingCharacters(in: .whitespaces))
        return """
            #!/bin/sh
            rm -f "$0"
            cd \(quote(directory)) || exit 1
            exec \(command)

            """
    }

    /// POSIX single quoting: everything inside is literal, and the only character that needs a
    /// way out is the quote itself.
    static func quote(_ value: String) -> String {
        "'" + value.replacingOccurrences(of: "'", with: "'\\''") + "'"
    }

    /// The absolute path to Claude Code, or the bare name when it is not where it installs
    /// itself. A terminal opens a login shell, so the bare name still resolves for anyone whose
    /// profile puts it on PATH; the absolute path is what makes it work for everyone else.
    static func claudeBinary() -> String {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        for candidate in homeRelativeBinaries {
            let path = home + "/" + candidate
            if FileManager.default.isExecutableFile(atPath: path) {
                return path
            }
        }
        for directory in knownDirectories {
            let path = directory + "/claude"
            if FileManager.default.isExecutableFile(atPath: path) {
                return path
            }
        }
        return "claude"
    }

    private static func write(_ script: String) throws -> URL {
        let file = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("rekall-\(UUID().uuidString).command")
        try script.write(to: file, atomically: true, encoding: .utf8)
        try FileManager.default.setAttributes([.posixPermissions: 0o700], ofItemAtPath: file.path)
        return file
    }

    // MARK: The terminal

    /// iTerm when it is installed, the one every Mac has otherwise. Asked of the workspace rather
    /// than looked for in /Applications, so an iTerm somewhere else still counts.
    private static func terminal() -> URL {
        let workspace = NSWorkspace.shared
        return workspace.urlForApplication(withBundleIdentifier: "com.googlecode.iterm2")
            ?? workspace.urlForApplication(withBundleIdentifier: "com.apple.Terminal")
            ?? URL(fileURLWithPath: "/System/Applications/Utilities/Terminal.app")
    }

    private static func name(of application: URL) -> String {
        application.deletingPathExtension().lastPathComponent
    }
}
