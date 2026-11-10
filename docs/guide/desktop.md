# The macOS application

A disk image for Apple Silicon is published with every `v*` tag pushed on a commit of `main`.

| | |
|---|---|
| Download | [Rekall-macos-arm64.dmg](https://github.com/ValerioMC/rekall/releases/latest/download/Rekall-macos-arm64.dmg) |
| Needs | macOS 13 or later, Apple Silicon |
| Install | Drag Rekall onto Applications, then run the `xattr` command below once |

The link always resolves to the newest tagged release. A push to `main` builds nothing; a release is cut with `git tag v0.1.1 && git push origin v0.1.1`, and a tag on a commit that is not on `main` fails before anything is built. It builds on a `macos-15` runner from `.github/workflows/release.yml`. The same release carries `rekall-server` for Linux and Windows, with the console embedded, and the [Windows installer](#windows-installer), `Rekall-windows-x64-setup.exe`, built on a `windows-latest` runner by the same script you run locally.

## Version and updates

A release build carries the tag it was built from (`v0.1.1` shows as `v0.1.1`; the workflow compiles it in as `REKALL_VERSION`); a local build shows the crate's version. `GET /api/version` asks GitHub for the latest release and reports whether its tag is newer than the running version, with the file for the platform the build runs on (the disk image on macOS) or the release page when the release has none; `?refresh=true` skips the remembered answer. The answer is remembered for six hours (five minutes after a failure) and a restart forgets it. Only `MAJOR.MINOR.PATCH` tags are compared.

The header shows no version. The desktop app checks once when it starts and, when a newer release exists, opens an update dialog (**Later** / **Install**); nothing is shown when GitHub cannot be reached. **Settings → Version** shows the running version, checks again on request and carries the same **Install** button, in a browser tab too (there it links the download).

| Property | Default | Meaning |
|---|---|---|
| `rekall.update-check.enabled` | `true` | `false` never contacts GitHub; Settings says checks are off |
| `rekall.update-check.url` | the repository's `releases/latest` API | The endpoint answering with the latest release |

In the macOS app **Install** puts the new version in place itself (`rekall-infra/src/version/update_installer.rs`, called by the `installUpdate()` bridge in `rekall-app/desktop/src/update_install.rs`):

1. It downloads the release's `.dmg` from inside the app, and only from `github.com/ValerioMC/rekall/releases/download/`.
2. It mounts the image with `hdiutil`, copies `Rekall.app` with `ditto` beside the running one as `.Rekall.app.new`, and swaps the two. The old bundle is kept as `.Rekall.app.old` until the new one is in place, and is put back if the swap fails.
3. It restarts into the new version, stopping the server cleanly first and reopening the app through the system so the new window comes to the front, not behind other apps. When a Claude session is live it asks first, in the console's own dialog, as quitting does (the system dialog only stands in when the console cannot show one).

The file never passes through a browser, so it gets no `com.apple.quarantine` and the new version opens without `xattr`. When the install fails (the app runs from the mounted disk image, `/Applications` is not writable, the release has no disk image), the dialog says why and its button opens the download in the browser. A browser tab, and the desktop app on other platforms, keep the plain download link. The first install from a browser download still needs the `xattr` command below once: only a Developer ID signature with notarization removes that.

A real install of the published release into a temporary folder (network, macOS only): `cargo test -p rekall-app live_ -- --ignored`.

## Build locally

```bash
cargo install tauri-cli --version "^2" --locked   # once

make dmg      # Rekall.app and its disk image, under target/release/bundle
make app      # the above, then mount the disk image and run Rekall from it
make install  # the above, then copy Rekall.app into /Applications
```

The app is a Tauri v2 window (`rekall-app/desktop`) over the same server `rekall-server` runs, started in the same process. `make app` installs nothing: it quits a running Rekall, ejects a Rekall disk image left mounted by an earlier build, mounts the new one and opens `Rekall.app` from the mounted volume (`scripts/macos-app.sh`), so what you try is exactly what another machine gets and a copy in `/Applications`, if there is one, is left alone. Eject the volume in Finder when you are done. To install it, drag it onto Applications from that window as anyone else would, or run `make install`, which does the same drag-and-drop by quitting any running Rekall, mounting the disk image, replacing `/Applications/Rekall.app` and ejecting the image (`scripts/macos-install.sh`).

`make desktop` builds the app's binary alone, on any platform with the WebView SDK (WebKitGTK on Linux); it is not a default workspace member, so `cargo build` and `cargo test` work on a machine without one.

### Windows installer

`make` does not run on Windows; `scripts/windows-installer.ps1` does the same job in one command:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\windows-installer.ps1 [-Version 0.1.1] [-SkipFrontend]
```

It checks the toolchain (Rust MSVC, the Visual C++ Build Tools, Node 22, pnpm 9), installs the Tauri CLI once, builds the console into `rekall-ui/dist`, and has `cargo tauri build --bundles nsis` produce `target\release\bundle\nsis\Rekall_<version>_x64-setup.exe`. The setup installs for the current user, so it needs no administrator rights. `-Version` stamps the installer and compiles it in as `REKALL_VERSION`, as the release workflow does for a tag; `-SkipFrontend` reuses the `dist` already built.

The Windows-only bundle settings (NSIS target, per-user install, the `.ico`) are in `rekall-app/desktop/tauri.windows.conf.json`, which Tauri merges over `tauri.conf.json` on Windows only, so the macOS build does not see them. `icons/icon.ico` is rendered from `scripts/icon-master.png` by `cargo tauri icon` the first time and is committed. The installer is not signed, so SmartScreen warns on first run. The release workflow's `build-installer-windows` job runs this same script with the tag's version and publishes the result as [Rekall-windows-x64-setup.exe](https://github.com/ValerioMC/rekall/releases/latest/download/Rekall-windows-x64-setup.exe), a name without a version so the link does not move. The console's update check on Windows still looks for the `-windows-x64.exe` asset, which is the server executable, not this installer.

The dock icon is `rekall-app/desktop/icons/icon@2x.png`, a 1024 master on Apple's icon grid (named `@2x` so Tauri's bundler reads its pixel size as retina density). It, the PWA icons, `apple-touch-icon.png` and `rekall-ui/public/favicon.svg` (a 256 copy wrapped in an SVG, which the console also shows as its logo) are all rendered from `scripts/icon-master.png`, a 1024 plate with transparent rounded corners: replace the master, run `make icons` (needs Google Chrome), and commit what it writes. The bundle build only reads them, so it needs neither Chrome nor the SVG.

## Running the app

The window fades in as a centered splash card (`rekall-app/desktop/splash/`, artwork `splash_screen.png`) sized at 40% of the screen's width, between 640 and 960 points, in the artwork's proportions, while the server boots; once port 47355 answers and the splash has been up for at least 1.5 s, the window fades out, grows to fill the screen unseen, waits for the console to finish loading (up to 5 s) and fades back in on it. Fades need macOS window opacity; elsewhere the window is simply shown and resized. Quitting stops the server and closes the database cleanly. When a Claude session is still live (a terminal started with **Run here** or by the run queue), `⌘Q`, the console's close button and the window's own close first ask whether to quit, since the session ends with the server; **Keep working** returns to the app. `rekall-app/desktop/src/exit_guard.rs` does the asking, in the console's own dialog (`LeaveDialog.vue`, answered through the `answerLeave()` bridge) and in a system dialog only when the console is not there to show it. Nothing is asked when no session is live, when the app attached to a server it did not start (the sessions outlive the window), or on `kill` and logout; Dock > Quit bypasses the question, because macOS ends the process there without telling the app. Server output goes to `~/Library/Logs/Rekall/server.log` (View > Open Server Log).

On macOS the window keeps the system's own close, minimize and maximize buttons, floated over the console's header (an overlay title bar with no bar and no title, `main_window.rs`); the header leaves the left edge clear for them. On Windows the window has no native frame: the console's header is the title bar and draws the three buttons itself (`WindowsWindowControls.vue`), flush with the right edge, as tall as the header, in Windows' order (minimize, maximize, close). Either way the header is draggable and maximized by double-click. The host tells the page which platform it was built for (`window.rekallDesktop.platform`, set in `bridges/bridge_script.rs`). Where the window keeps its native frame (Linux, a browser tab) nothing is drawn.

The header's left edge is a menu key (three lines) beside the back and forward arrows. Pressing the key hides the arrows and opens the screen tabs, with their sub-menus and alert counts, across the header from the key rightwards; a press anywhere outside folds them back and the arrows return. A window narrower than 46rem opens them as a vertical menu instead (`HeaderNavigation.vue`).

The app uses the same port, the same `~/.rekall/config.json` and the same MCP endpoint as `make run`. If a Rekall server already answers on 47355, the app attaches to it instead of starting a second one, and leaves it running on quit. The folder icon in the database field opens the system folder chooser, which a browser tab cannot do; in a browser that field stays a typed input.

The bundle is signed ad hoc, which is enough for the machine that built it. A disk image downloaded through a browser on another machine is quarantined and needs one command before it opens the first time (updates installed from the app do not, see *Version and updates*):

```bash
xattr -dr com.apple.quarantine /Applications/Rekall.app
```

