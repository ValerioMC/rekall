// The bridge between the page and the system's notification centre.
//
// Its own file for the same reason FolderPicker is: the wiring is installed with one call and
// can be built into a bare WKWebView, with no window or server behind it.

import AppKit
import UserNotifications
import WebKit

/// Posts the notification the console asks for when a session claims work while Rekall is in
/// the background. WKWebView has no Notification API of its own, so without this the app would be
/// the one place the console could not tell anyone that something is waiting for review.
///
/// The page sends a title and a body and nothing else: no action, no URL, no sound to choose. Both
/// are cut to a length a banner can show, and the system asks for permission itself the first
/// time one is posted; a refusal resolves the page's promise to false rather than failing it.
final class Notifier: NSObject, WKScriptMessageHandlerWithReply, UNUserNotificationCenterDelegate {

    static let messageName = "rekallNotify"

    private static let titleLimit = 120
    private static let bodyLimit = 300

    /// The JavaScript half. It merges rather than assigns, like the other bridges on the object.
    static let bridge = """
        window.rekallDesktop = Object.assign(window.rekallDesktop || {}, {
            notify: function (notice) {
                return window.webkit.messageHandlers.\(messageName).postMessage({
                    title: String((notice && notice.title) || ''),
                    body: String((notice && notice.body) || '')
                });
            }
        });
        """

    @discardableResult
    static func install(into configuration: WKWebViewConfiguration) -> Notifier {
        let notifier = Notifier()
        configuration.userContentController.addScriptMessageHandler(
            notifier, contentWorld: .page, name: messageName)
        configuration.userContentController.addUserScript(
            WKUserScript(source: bridge, injectionTime: .atDocumentStart, forMainFrameOnly: true))
        // UNUserNotificationCenter needs a bundle to belong to; a bare build has none, and there
        // the bridge still answers, with false.
        if Bundle.main.bundleIdentifier != nil {
            UNUserNotificationCenter.current().delegate = notifier
        }
        return notifier
    }

    func userContentController(_ controller: WKUserContentController,
                               didReceive message: WKScriptMessage,
                               replyHandler: @escaping (Any?, String?) -> Void) {
        guard message.name == Self.messageName else {
            replyHandler(nil, "Unknown message \(message.name)")
            return
        }
        guard Bundle.main.bundleIdentifier != nil else {
            replyHandler(false, nil)
            return
        }
        guard let body = message.body as? [String: Any],
              let title = body["title"] as? String, !title.isEmpty else {
            replyHandler(nil, "A notification needs a title")
            return
        }
        let text = body["body"] as? String ?? ""

        let center = UNUserNotificationCenter.current()
        center.requestAuthorization(options: [.alert, .sound]) { granted, _ in
            guard granted else {
                DispatchQueue.main.async { replyHandler(false, nil) }
                return
            }
            let content = UNMutableNotificationContent()
            content.title = String(title.prefix(Self.titleLimit))
            content.body = String(text.prefix(Self.bodyLimit))
            content.sound = .default
            let request = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil)
            center.add(request) { error in
                DispatchQueue.main.async { replyHandler(error == nil, nil) }
            }
        }
    }

    /// Shows the banner even while Rekall is frontmost; the page only asks when it is not looked at.
    func userNotificationCenter(_ center: UNUserNotificationCenter,
                                willPresent notification: UNNotification,
                                withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        completionHandler([.banner, .sound])
    }

    /// A click on the banner brings the window back, where the review queue is waiting.
    func userNotificationCenter(_ center: UNUserNotificationCenter,
                                didReceive response: UNNotificationResponse,
                                withCompletionHandler completionHandler: @escaping () -> Void) {
        DispatchQueue.main.async {
            NSApp.activate(ignoringOtherApps: true)
            completionHandler()
        }
    }
}
