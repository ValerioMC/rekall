## Scanning components

`/rk project:vega components` means: read the project's folder as a developer who has to run it and
build in it would, and record every part of it as a component, so that a session or a person finds
what each part is, how it is wired and how to change it.

1. Drop the `components` term and call `rekall_context` with the anchors that are left. You change
   no code, no checklist and no task.
2. Read the project's folder, never your memory of it. List its top levels and its manifests
   (`package.json`, `Cargo.toml`, `pom.xml`, `build.gradle`, `go.mod`, `pyproject.toml`,
   `*.csproj`, `Dockerfile`, `docker-compose*`, `Chart.yaml`, `*.tf`, workspace files), its CI,
   its README, its deploy folders. Look through the repository, not at its first level: a
   `backend/` or `services/` folder holds one component per service.
3. Decide the parts. A component is a folder a developer would name when asked "where do I change
   this?":
   - each service, microservice, app, CLI or worker that is built and run on its own;
   - each library or shared module other parts depend on;
   - the configuration that makes the whole work: deployment (Helm, Kubernetes, Terraform, Compose),
     CI/CD, database migrations, shared configuration, scripts;
   - a documentation or tooling folder only when the work routinely lives in it.

   Not a component: `node_modules`, build output, vendored code, a folder of one file, a test folder
   of its own part (that part's tests belong to its description). One folder, one component; a part
   that spans two folders is recorded under the one that holds its entry point.
4. Read what already is: `rekall_read_components` lists the components the project has. One with the
   same name or folder is rewritten, keeping everything it says that is still true, and the name a
   person gave it stays. Never record a second component for a folder that has one.
5. For each part, read enough to write its description from the code: the entry point and how it
   starts, its routes, commands, events or exported API, its configuration and environment
   variables, its tests, the other parts it calls or is called by, how it is built and run. The README
   is a clue to check against the code, not the source.
6. Write each with `rekall_component`, one call each, the `summary` (what it is for and what it owns,
   in two or three sentences) and the `description` in the structure the tool names: Purpose,
   Responsibilities, Stack and layout, Interfaces, Depends on and Used by, Configuration and data,
   Run and develop, Adding a feature, Flow, Gotchas. Facts in tables, one row each, a path where it
   helps. A Mermaid diagram only where it shows how the part is wired and prose would not. Refer to
   other parts by the name their own component carries; write the dependent ones after the ones they
   depend on, so the names exist.
7. A section with nothing true to say is left out, and a fact you could not confirm is left out
   with it: say so in step 8 instead of guessing a command, a variable or a route.
8. Say in a few lines which components you wrote and which you rewrote, and what you could not
   establish. Then stop: no wrapup, no step, no commit.

A written component replaces the whole of what was there, so send all of it every time. Nothing here
reorders or deletes a component: a folder that is gone is left for the person to delete.
