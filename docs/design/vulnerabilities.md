# Vulnerabilities

The Security screen answers two questions about a project: does a critical vulnerability (or a high
one, or malware, when the scan is set to look for them) reach a library it uses, and is the
vulnerable code actually reached. The first is a lookup Rekall makes; the
second needs the code read, so a session makes it.

| Piece | What it decides |
|---|---|
| `DependencyInventoryService` (rekall-core, `inventory/`) | A project's dependencies, written whole by `write_in` and read by the console and the scan. `summaries` adds the manifests `manifest_freshness::changed_since` finds modified after the inventory was recorded, or gone, in the project's folder. `inventory::rules` refuses an empty list, more than 5,000 entries, an unknown ecosystem, a version that is a range (`^`, `~`, `>`, `<`, `=`, `*`, `\|`, a space or a comma), and a manifest path that is absolute or climbs out with `..`; a leading `v` before a digit is dropped, and an entry repeating an ecosystem, name and version is kept once |
| `InventoryRecordingService` | The session's write: the inventory and, in the same transaction, `forget_missing_in`, which drops the exposures whose package the new list no longer holds at that version |
| `AdvisoryFeed` | The seam to the advisory database: the latest critical advisories, and the ones affecting a list of `package@version` in one ecosystem within a `ScanScope`. `GithubAdvisoryFeed` in rekall-infra reads `GET /advisories?type=reviewed&severity=critical`, the second form with `ecosystem` and `affects`; with `include_high` it drops the severity and keeps critical and high itself (GitHub filters one severity at a time), and with `include_malware` it asks `type=malware` too. It follows the `Link` header's `next` for up to ten pages, and signs every request with the token in the credential store under `ADVISORY_TOKEN_ACCOUNT` when there is one. A withdrawn advisory, one whose id does not read and a package in an ecosystem Rekall does not inventory are dropped. A 429, or a 403 with no allowance left, is `RateLimited`. `StartOptions::advisory_feed` stands in for it, and the test harness hands every app a silent one |
| `ScanSettingsService` | What a scan looks for (the `scan_scope` row) and the GitHub token, which goes to the OS credential store and is only ever reported as saved or not. A token blank, with spaces or past 255 characters is a 400 |
| `VulnerabilityScanService` | One scan: the 100 newest critical advisories, then per inventoried project and ecosystem the questions `questions` cuts: at most 80 packages, each name once (`rules::package_key`: ignoring case, and pip's separators folded), a second version of a package going to a later question, because GitHub answers which advisories reach any of them and not which version. `reached_in` ties an answer to every package of the question the advisory names whose version its range holds or cannot be read for (`version_range`), or to every one it names when GitHub and the range disagree. Everything is recorded in one transaction: advisories upserted by GHSA id, each answered project's exposures replaced by what was found. A project whose question failed keeps its exposures; once rate limited, the rest are skipped and reported. A mutex lets one scan run at a time; a second is a 409 |
| `version_range` | Whether a version is inside a GitHub range (`>= 4.0.0, < 4.1.0`): release numbers compared as numbers, a pre-release before its release, a post-release after it; anything unreadable is undecided. `AffectedPackage::find` uses it to pick, of the entries an advisory lists for one package, the release line the project's version is on, which is where an exposure's range, first safe version and vulnerable functions come from |
| `VulnerabilityExposureService` | The exposures, one per advisory and package version. `replace_found_in` adds new ones unjudged, keeps a known one with its verdict, and deletes the rest, so a package moved to another version is a new, unjudged exposure. `assess` writes the verdict on every exposure of the advisory in the project, with the project's `HEAD` (`GitRepositoryInspector::head`), and refuses an advisory not exposed on the project, naming the ones that are. `list` and `read` add the commits gained since (`commits_since`, once per project and commit). An exposure is open (a warning) until it is `NOT_AFFECTED` or a fix task is at work on it: a fix task that is `DONE` while the exposure is still found leaves it open and lets it take a new fix |
| `VulnerabilityFixService` (rekall-console) | Creates the fix task through `TaskService::create_in` and ties it to the exposure, in one transaction; a malware exposure's brief asks for removal and for whatever installed it to be treated as compromised. The label is `fix-<ghsa id>`, numbered when taken; the description (`fix_task_rules`) is the brief a session works from. `fix_project` takes every exposure of a project its `FixScope` takes (`fix_briefs_of_project_in`, a 409 when none): `AFFECTED`, the open ones, or `ALL`, every one without a fix task, the ones judged not affected included, which the description marks. It gathers them into one `fix-vulnerabilities` task, adding an open step per exposure through `TaskStepService::add_open_in` (not a draft: the console starts a session on the task at once, and a session may not work a draft) whose detail is that exposure's brief, the advisory's text cut to fit the step's 20,000 characters, and ties each exposure to it |

A scan runs five minutes after the application starts and every six hours (`Instance::start_vulnerability_scans`),
and on `POST /api/vulnerabilities/scan`. Every write publishes `vulnerability` (with how many exposures
a scan found for the first time) or `inventory` on the event stream; the console's `useVulnerabilityStore`
is loaded by the shell and kept current by them, so the Security tab carries the count of open warnings
from any screen.

The console's `features/vulnerability` is one screen cut by view. `lib/security-view` reads the view from
the query (overview, advisories with the one picked, a project and its tab); `lib/project-posture` orders
the rail (`exposed`, `uncovered`, `unreadable`, `clear`); `lib/dependency-filter` ties each dependency to the
exposures on its package and version, with the scan's own name folding, and filters it; `useVirtualRows`
keeps only the visible rows of a long inventory in the DOM. An advisory's description is reporter-written
markdown, so `lib/advisory-description` cuts it into text blocks and nothing renders it as HTML.

Sharp edges: matching is by GitHub's own version semantics, so a version written differently from how
the ecosystem spells it is missed rather than misjudged; `version_range` only picks which release line
to show, and where it cannot read a range the advisory's first entry for the package answers; staleness
is by modification time, so a branch switch that rewrites a lockfile marks the inventory changed even
when its content is not; a session on a project alone, reopened from the live-work bar, lands on the
Diagrams screen whatever it was started for.
