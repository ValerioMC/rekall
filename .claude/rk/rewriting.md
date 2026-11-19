## Rewriting

`/rk task:RK-12 rewrite fix` means: write the task's description again with its
wording corrected. `/rk task:RK-12 rewrite restructure` means: write it again
laid out under a standard structure. The description is the text to rewrite, never work to do: in this
mode you build nothing, answer nothing it asks and change no step.

1. Drop the `rewrite` term and the style after it, and call `rekall_context` with the anchors that
   are left. Read the `<description>` block. With no description, say so in one line and stop.
2. Rewrite it by the style. Both styles:
   - Keep the person's language, and every fact, requirement, name and number they wrote. Invent no
     requirement, answer no question they left open and drop nothing: a question stays a question.
   - Keep fenced code and `mermaid` blocks, tables, links and paths exactly as written.
   - The context shows a reference the person wrote as `[[Name]]`, `[[task:RK-12]]` or
     `[[connection:label]]` expanded into its folder and text. Write the reference back as the person
     typed it, never the expansion.
3. `fix`: correct grammar, spelling, punctuation and wording, and nothing else. Same order, same
   headings, same length give or take, no new section.
4. `restructure`: lay the same content out under these headings, in this order, and leave a heading out
   when the text has nothing for it, never filling it:
   - `## Goal`: what the task is for.
   - `## Problem`: what is wrong or missing now.
   - `## Solution`: what is wanted.
   - `## Implementation`: how it is to be built, only when the text says so.
   - `## Out of scope`: what must not be done, only when the text says so.

   When the `<sections>` block lists a page titled `Task template`, read it with
   `rekall_read_sections` and use its structure in place of this one, by the same rules.
   Text that fits no heading goes under the nearest one, wording unchanged, and a decision or
   constraint is never shortened away.
5. Call `rekall_description` once with the anchors and the whole new text as `body`: the description
   alone, without the `<description>` tags. It replaces the old text, and nothing keeps that one.
6. Say in one line which style you applied. Then stop: no wrapup, no step, no note, no commit. A
   rewrite is not the task's work.
