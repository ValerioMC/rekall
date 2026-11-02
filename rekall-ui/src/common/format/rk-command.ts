export function rkCommand(anchor: string): string {
  return anchor ? `/rk ${anchor}` : ''
}

/**
 * The planning line: a session that reads the task and proposes its checklist, which lands on the
 * Steps pane open and ready to work.
 */
export function rkPlanCommand(anchor: string): string {
  const base = rkCommand(anchor)
  return base ? `${base} plan` : ''
}

/**
 * The flow planning line: the session proposes the steps and the routes between them, open and
 * ready to work, which only a task with no steps, drafts included, accepts.
 */
export function rkPlanFlowCommand(anchor: string): string {
  const base = rkPlanCommand(anchor)
  return base ? `${base} flow` : ''
}

/**
 * The replanning line: the session redesigns the task's draft and open steps and lays the new ones
 * ready to work. Steps that are running, claimed or done stay.
 */
export function rkReplanCommand(anchor: string): string {
  const base = rkCommand(anchor)
  return base ? `${base} replan` : ''
}

/**
 * The wrapup line, with an optional directive folded in. A directive is one line: newlines are
 * collapsed to spaces so it cannot submit the terminal line early, and `\`/`"` are escaped so it
 * cannot close the quote it sits in.
 */
export function rkWrapupCommand(anchor: string, directive = ''): string {
  const base = rkCommand(anchor)
  if (!base) return ''
  const flattened = directive.trim().replace(/\s+/g, ' ')
  if (!flattened) return `${base} wrapup`
  const escaped = flattened.replace(/\\/g, '\\\\').replace(/"/g, '\\"')
  return `${base} wrapup "${escaped}"`
}
