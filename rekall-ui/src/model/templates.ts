/*
 * What a new task and a new step start from. Each block answers one question a session asks before
 * it builds, in the order it asks them; the comments say what goes there and never render in Read.
 */
export const TASK_DESCRIPTION_TEMPLATE = `## Goal

<!-- The outcome in one or two sentences, and who notices when it lands. -->

## Requirements

<!-- What the result must do or respect, one checkable line each. -->
-

## Out of scope

<!-- What this task deliberately leaves alone. -->
-

## Constraints

<!-- Where the code goes, what it must reuse, what shape it takes. Delete if none. -->
`

export const STEP_DETAIL_TEMPLATE = `## Change

<!-- What to build and where it lives in the code. -->

## Done when

<!-- How a reviewer can tell, one check each. -->
-
`

/** True while a body still holds nothing but the template it was created from. */
export function isUntouchedTemplate(body: string, template: string): boolean {
  return body.trim() === template.trim()
}
