# MCP tools

| Tool | Access | Effect |
|------|--------|--------|
| `rekall_context` | read | Resolve anchors, return markdown |
| `rekall_wrapup` | write | Replace one task's wrapup |
| `rekall_step` | write | Move one step: `open` to `running` to `claimed`; on a flow, a claim's `outcome` follows that route and its `report` reaches the step it leads to |
| `rekall_record_commit` | write | Log a commit of the project's repo folder against one task or step |
| `rekall_propose_step` | write | Add one step as a draft, for a person to promote; `new_session` makes it open a fresh Claude session |
| `rekall_propose_flow` | write | Add a whole flow to a task with no steps: its steps as drafts and the routes between them, each optionally opening a fresh session with `new_session` |
| `rekall_create_task` | write | Create a task or an epic in a project with a description and up to 20 draft steps (none on an epic), optionally filed under an epic |
| `rekall_join_epic` | write | File an existing task under an epic of its project, or move it from another epic |
| `rekall_note` | write | Write a note on one task, or rewrite one it carries by title with `replace` |
| `rekall_project_section` | write | Write a titled markdown section on a task's project, or rewrite one by title with `replace` |
| `rekall_read_sections` | read | List a project's sections by title, or read one whole with `title` |
| `rekall_diagram` | write | Store Mermaid sheets generated for a project or one of its tasks, or replace a diagram by id with `diagram` |
| `rekall_inventory` | write | Replace a project's dependency inventory whole: ecosystem, name, exact version and declaring file per entry |
| `rekall_vulnerabilities` | read | List the advisories that reach a project, or read one whole with `advisory`, with how many commits the project gained since its verdict |
| `rekall_assess_vulnerability` | write | Record whether an advisory really reaches a project: `affected` or `not_affected`, with the reasoning and a plain-words conclusion, on every version of the package it reaches there |

There is no query, get or schema tool. `rekall_create_task` takes a project anchor, a `title`, an optional `label` (the title as a label when omitted; one the project already uses is refused), an optional `description`, an optional `kind` (`task` or `epic`; an epic takes no `steps`), an optional `epic` (the label of an epic of the project to file the task under) and optional `steps` (`[{title, detail}]`). The task starts `TODO` and every step starts as a draft on the staging shelf, exactly as with `rekall_propose_step`, so nothing the session creates is work until you promote it; a request with a repeated step title or more than 20 steps is refused and creates nothing, and an open console lists the new task without a reload. Sections are loaded into no context, so `rekall_read_sections` is how a session reads them: with a project (or a task and its project) it lists the titles with their size, with `title` it returns that section whole, and it changes nothing. `rekall_note` adds a note, or with `replace: true` rewrites the body of the note the task already carries under that title (ignoring case); without `replace` that title is refused, and nothing it does detaches or deletes a note. `rekall_project_section` works the same way on a project's sections: it adds one, or with `replace: true` rewrites the body of the section with that title, and it can never rename, reorder or delete one. `rekall_step` refuses `done`; that state is set by hand in the console. `rekall_step` (on `claimed`) and `rekall_wrapup` take an optional `commit_message`, used only on a project that commits on claim (see [Commit on claim](workflow.md#commit-on-claim)).
