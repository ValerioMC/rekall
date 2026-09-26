/// The JavaScript half, installed before the page runs. It merges rather than assigns, as the
/// three Swift bridges did, and turns a rejected invoke into an `Error` the console can read.
pub const BRIDGE: &str = r#"
(function () {
  if (!window.__TAURI_INTERNALS__) return;
  var call = function (command, args) {
    return window.__TAURI_INTERNALS__.invoke(command, args).catch(function (failure) {
      throw new Error(typeof failure === 'string' ? failure : (failure && failure.message) || String(failure));
    });
  };
  window.rekallDesktop = Object.assign(window.rekallDesktop || {}, {
    pickFolder: function (currentPath) {
      return call('pick_folder', { path: typeof currentPath === 'string' ? currentPath : '' });
    },
    openInClaudeCode: function (launch) {
      return call('open_in_claude_code', { launch: {
        directory: String((launch && launch.directory) || ''),
        anchors: String((launch && launch.anchors) || ''),
        skipPermissions: Boolean(launch && launch.skipPermissions)
      } });
    },
    notify: function (notice) {
      return call('notify', { notice: {
        title: String((notice && notice.title) || ''),
        body: String((notice && notice.body) || '')
      } });
    },
    closeWindow: function () {
      return call('close_window', {});
    },
    minimizeWindow: function () {
      return call('minimize_window', {});
    },
    toggleMaximizeWindow: function () {
      return call('toggle_maximize_window', {});
    }
  });
})();
"#;
