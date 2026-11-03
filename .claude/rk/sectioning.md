## Sectioning

`/rk project:vega task:report-builder section "Deployment" "how we ship to production"` means: write a
section named Deployment on that task's project, or rewrite the one it already has under that title.

A section is a page of the project, shown beside its blueprint in the console. A context carries only
its title, in a `<sections>` block, so it costs a line and a later session reads it with
`rekall_read_sections` when the work touches it. Use it for what is
true of the project as a whole (deployment, conventions, architecture, a glossary), never for what
one task built (that is the wrapup) or produced (that is a note).

The first quoted text is the section's title, exactly as I typed it. The second, if there is one,
says what it has to hold, in my words; it is not the section's text. Without it, the section holds
what the task or the step that asked for it describes. This is also what a step like "create a
documentation in a section named xxx" means: the name is the title, the rest of the step is the content.

1. Drop the `section` term and the quoted texts, and call `rekall_context` with the anchors that are
   left, unless that exact task is already loaded in this session.
2. Write the section from the code and from what the quoted text asks for, in markdown, to be read on
   its own by someone who was not here.
3. Call `rekall_project_section` with those anchors, the `title` and the whole section as `body`. If
   the project already has a section under that title and this one supersedes it, pass `replace: true`.
4. Say in one line which section you wrote or rewrote and on which project. Then stop.

`replace` overwrites the body outright and no history keeps the old one, so send the whole section,
not the part that changed. Without it, a title the project already uses is refused. Nothing here
renames, reorders or deletes a section, and writing one does not claim the task or a step.
