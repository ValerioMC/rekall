## Rewriting

`/rk task:RK-12 rewrite fix` means: write the task's description again with its
wording corrected, as a brief description built from its steps and decisions too. `/rk task:RK-12 rewrite restructure` means: write it again
laid out under a standard structure, the steps and decisions placed in it. The description, the steps and the decisions are the text
to rewrite from, never work to do: in this mode you build nothing, answer nothing they ask and change no step.

1. Drop the `rewrite` term and the style after it, and call `rekall_context` with the anchors that
   are left. Read the `<description>` block, the `<steps>` and the `<decisions>`. With no description,
   no step and no decision, say so in one line and stop.
2. Rewrite by the style. Both styles:
   - Keep the person's language, and every fact, requirement, name and number they wrote. Invent no
     requirement, answer no question they left open and drop nothing: a question stays a question.
   - Steps and decisions are source material, never changed: add, edit or tick no step, record no
     decision. The text may use them, and it never invents a requirement they do not carry.
   - The context shows a done step as its title only, and a decision `(was X)` as history: quote a done
     step by its title and do not write a replaced answer into the text.
   - Keep fenced code and `mermaid` blocks, tables, links and paths exactly as written.
   - The context shows a reference the person wrote as `[[Name]]`, `[[task:RK-12]]` or
     `[[connection:label]]` expanded into its folder and text. Write the reference back as the person
     typed it, never the expansion.
3. `fix`:
   - With steps or decisions: write a brief description of the task from the description text, the
     steps and the decisions together: what it is for, what was settled and the shape of the work. Do
     not copy the checklist step by step. Every fact in the old text survives. With no description,
     write it from the steps and decisions alone.
   - With neither steps nor decisions: correct grammar, spelling, punctuation and wording, and
     nothing else. Same order, same headings, same length give or take, no new section.
4. `restructure`: lay the same content out under these headings, in this order, and leave a heading out
   when the text has nothing for it, never filling it:
   - `## Goal`: what the task is for.
   - `## Problem`: what is wrong or missing now.
   - `## Solution`: what is wanted.
   - `## Implementation`: how it is to be built, only when the text or the steps say so. The steps count
     as the text saying how it is built: draw this heading from them, open steps from their detail and
     done steps as already done.
   - `## Out of scope`: what must not be done, only when the text says so.

   When the `<sections>` block lists a page titled `Task template`, read it with
   `rekall_read_sections` and use its structure in place of this one, by the same rules.
   Text that fits no heading goes under the nearest one, wording unchanged, and a decision or
   constraint is never shortened away. A settled decision (`key: answer`) goes under the heading it
   belongs to, its wording kept. An open one (`key: ? ...`) stays a question. One that is `on hold`
   stays marked as on hold.
5. Call `rekall_description` once with the anchors and the whole new text as `body`: the description
   alone, without the `<description>` tags. It replaces the old text, and nothing keeps that one.
6. Say in one line which style you applied. Then stop: no wrapup, no step, no note, no commit. A
   rewrite is not the task's work.
