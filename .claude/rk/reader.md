## Writing for a reader

Every text you save on a task is read later by a person, in the console, without your session and
often without the code open: a step's detail, a plan or analysis note, a deviation, the question of
a decision, the wrapup, a diagram and its walkthrough. You wrote it with the code loaded, so every
sentence can be accurate and still mean nothing to them. This file is the order those texts follow
so that a developer can read them. The mode that writes a text says what goes in it; this one says
how it is laid out.

### The order

1. **Scenario.** Who does what, where, with one concrete example: the screen or the command, the
   record, the values. *"In the Plan tab of RK-12 you drag step 4 above step 2; step 4 opens
   Session 2."* Not "the user reorders steps".
2. **What happens, and what changes.** What the person sees in that scenario today and what they
   will see after the work, in behaviour, not in code. For a defect, what goes wrong and what it
   costs them. When nothing visible changes (a refactor, a speed-up), say so plainly and name who
   does feel it: the developer changing that code, a test that needs a network, a call that reads
   more than it shows.
3. **The map.** The parts the scenario passes through, in the order it reaches them, one line each:
   what the part does in plain words, then its name in the code. *"the drag handler that turns a
   drop into requests (`usePlanMoves`) → the step API → ..."*. Only the parts the text needs.
4. **Terms.** A Rekall word a newcomer cannot guess (session, opener, draft, claimed, route, run
   queue) is defined in a clause where it first appears. So is a word of the project's own domain.
5. **The work.** What to build or what was built, where, and how to tell it is done.

Parts 1 to 4 are what a reader without the code is missing; part 5 is what you would have written
anyway. They are as long as the text needs and no longer: a fix in one function gets a one-line
scenario, a change that crosses four layers gets a paragraph and a map.

- Plain words come first and code names after them, never instead of them. A sentence whose subject
  is a type (*"`applyRelayout` writes positions..."*) belongs in the map, not in the opening.
- One example runs through the whole text. The values of the scenario are the ones parts 2 and 3
  follow and the ones the done-when checks.
- The scenario is what the code does, not what it probably does. Read the code before you write
  it; if you did not see a request, a refusal or a screen, it is not in the text.
- Each text stands on its own. A step points to its note for the whole picture, and still says its
  own scenario in a sentence.

### Each kind of text

| Text | How it follows the order |
|---|---|
| A step's detail | Opens with a **Scenario** paragraph (parts 1 and 2), then the map where the step crosses parts, then the build, its tests and the done-when. A diagram in it says in one line which scenario it plays. |
| A plan or analysis note | Opens with the use cases the task touches, each a short scenario, and the terms it uses. A **finding** is the steps that reproduce it (where you are, what you do, what you see), then its consequence for the person, then the code that causes it. |
| A deviation | Opens with the situation where the plan and the code differ, then what was planned, what was built and why. |
| The question of a decision | Says the situation the choice changes and what the person would see with each answer, before the answers. |
| The wrapup | Opens with what a person can now do, in the scenario's terms. Its table of pieces says each piece's job in plain words before its name. The state only, as `wrapping-up.md` says: no before and after. |
| A diagram's scenario | `situation` is who does what and where, `example` the concrete values, `outcome` what they see or get at the end. |
| A walkthrough message | Its first message is the trigger the scenario names. `explain` says what happens to the example at this call; `input` and `output` carry the example's values as they are at that call, not the type's shape, wherever the code makes them concrete. |

### The check

Before you save, read the first paragraph as a developer who has never opened this repository. They
can say what situation it is about (who, where, what they did) without meeting a type name. Read on
to the first code name: they know by then what goes wrong or what changes. If not, rewrite the
opening, not the end.

### Before and after

RV-197's step 2, as it was written:

> Wire the step API and existing Pinia slice to the placement operation from step 1. Keep
> `step-order.ts` responsible for calculating the intended placement for drag, keyboard and menu
> actions. Make each action and its Undo submit one command.

Every sentence is accurate, and a reader stops at "placement operation": which action, and what is
wrong with it today? The same opening, under the contract:

> **Scenario.** In the Plan tab of RK-12, steps 1 to 3 run in Session 1 and step 4 opens Session 2.
> You drag step 4 above step 2. The console sends three requests, one after the other: move step 4
> to position 2, stop step 4 opening a session, make step 5 open one. Each is saved as it arrives.
> If the second fails, the first stays: the checklist shows step 1 alone in Session 1 and steps 4, 2,
> 3 and 5 in Session 2, a grouping you never asked for, with an error and no Undo.
>
> **What changes.** The same drag sends one request, saved whole or not at all, and its Undo is one
> request too.

Only then the map (the drag handler, the step API, the controller, the service) and the build.
