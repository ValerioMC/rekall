## Reorganizing

`/rk project:lightcraft reorganize` means: read the project's pages, group them by what they are
about, and put each in the context it belongs to, so that the page list reads as a few concepts
instead of one heap. You move pages and name contexts; you do not write, rename or delete a page.

1. Drop the `reorganize` term and call `rekall_context` with the anchors that are left. You change
   no code, no checklist and no task.
2. Read every page of the project, whole: `rekall_read_sections` lists them by context, then call it
   again with each `title`. A page's context today is a hint, not a verdict: the pages may all sit in
   one context, or in the wrong ones. If the project has no page, say so and stop.
3. Decide the concepts from what the pages say, never from their titles alone. A concept is a thing a
   reader looks for as one: "Architecture", "Operations", "Team", "Onboarding", "Security". Aim for
   three to seven contexts, none with a single page unless nothing else fits it, and none called
   "Misc", "Other" or "General" unless the project already uses that title. Reuse a context the
   project already has when it still fits, with the title it has.
4. Order the contexts the way a newcomer would read them, and the pages inside each from the one that
   explains the most to the ones that go into detail. A page that another page links to with
   `[[page:Title]]` sits near it when that reads naturally; the link works wherever the page is.
5. Send the whole arrangement in one `rekall_reorganize_sections` call: every page of the project,
   once, by its exact title. The call is refused, and nothing moves, if a page is left out, named
   twice or unknown: read the refusal, correct the list, call again.
6. Say in a few lines which contexts the project has now and why each page went where it did, so I
   can move one back. Then stop: no wrapup, no step, no commit.
