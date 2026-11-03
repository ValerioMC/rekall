## Inventorying

`/rk project:vega inventory` means: record what the project is built from, every library and
framework at the exact version it uses, with `rekall_inventory`. Rekall checks that list against
the critical advisories of the GitHub Advisory Database and warns on its Security screen.

1. Drop the `inventory` term and call `rekall_context` with the anchors that are left. You build
   nothing and no checklist changes.
2. Find every manifest and lockfile in the project's folder yourself, in every subfolder that is a
   part of the project: `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `Cargo.lock`,
   `requirements*.txt`, `poetry.lock`, `uv.lock`, `Pipfile.lock`, `pom.xml`, `build.gradle(.kts)`,
   `gradle.lockfile`, `go.mod`/`go.sum`, `composer.lock`, `Gemfile.lock`, `*.csproj`,
   `packages.lock.json`, `pubspec.lock`, `Package.resolved`, `mix.lock`, the workflow files under
   `.github/workflows`. Skip `node_modules`, `target`, `vendor`, build output and anything ignored.
3. Read the exact version each dependency resolves to, from the lockfile when there is one. Where a
   manifest only gives a range and there is no lockfile, use the version the build would pick if the
   code makes it plain, and leave the dependency out if it does not: a guessed version raises false
   warnings. Include what a lockfile pins transitively.
4. Call `rekall_inventory` with the project anchor and the whole list, one entry per package and
   version: `ecosystem` by the advisory database's name (`npm`, `pip`, `maven`, `rust`, `go`,
   `nuget`, `rubygems`, `composer`, `pub`, `swift`, `erlang`, `actions`), `name` as that ecosystem
   writes it, `version`, and `manifest` relative to the project's folder. If the answer is a refusal,
   fix what it names and send the whole list again.
5. Say in two or three lines how many dependencies you recorded per ecosystem, what you left out and
   why. Then stop: no wrapup, no step, no commit.
