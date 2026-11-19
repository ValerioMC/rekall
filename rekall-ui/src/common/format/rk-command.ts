import type { RewriteStyle } from '@/features/claude/model/terminal'

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
 * ready to work. Steps that are running, claimed or done stay. A message is what the plan missed,
 * quoted as one line.
 */
export function rkReplanCommand(anchor: string, message = ''): string {
  const base = rkCommand(anchor)
  if (!base) return ''
  const quoted = quotedLine(message)
  return quoted ? `${base} replan ${quoted}` : `${base} replan`
}

/**
 * The revising line: the session works the pending revisions of the task's thread, each applied
 * to the work or the plan, on top of what is already built.
 */
export function rkReviseCommand(anchor: string): string {
  const base = rkCommand(anchor)
  return base ? `${base} revise` : ''
}

/**
 * The rewriting line: the session corrects the wording of the task's description (`FIX`) or lays it
 * out under standard headings (`RESTRUCTURE`), and stores the result as the new description.
 */
export function rkRewriteCommand(anchor: string, style: RewriteStyle): string {
  const base = rkCommand(anchor)
  return base ? `${base} rewrite ${style.toLowerCase()}` : ''
}

/** The wrapup line, with an optional directive folded in. */
export function rkWrapupCommand(anchor: string, directive = ''): string {
  const base = rkCommand(anchor)
  if (!base) return ''
  const quoted = quotedLine(directive)
  return quoted ? `${base} wrapup ${quoted}` : `${base} wrapup`
}

/**
 * Text as one quoted terminal line, or nothing when it is blank. Newlines are collapsed to spaces so
 * it cannot submit the line early, and `\`/`"` are escaped so it cannot close the quote it sits in.
 */
function quotedLine(text: string): string {
  const flattened = text.trim().replace(/\s+/g, ' ')
  if (!flattened) return ''
  return `"${flattened.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`
}
