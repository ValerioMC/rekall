import type { TaskStepLine } from '@/features/project/model/project-task-list'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskRow } from '@/features/task/model/task-row'

const FENCE = /^(```|~~~)/
const NOT_PROSE = /^(#{1,6}\s|\||<!--|!\[|(-{3,}|\*{3,}|_{3,})$)/
const BLOCK_MARKER = /^(>\s*|[-*+]\s+(\[[ xX]\]\s+)?|\d+[.)]\s+)+/

function plainText(line: string): string {
  return line
    .replace(BLOCK_MARKER, '')
    .replace(/\[\[([^\]]+)\]\]/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/(\*\*|__|`)/g, '')
    .trim()
}

/** The first line of prose a description holds, without its markdown: headings, tables, pictures and code are passed over. */
export function summaryOf(markdown: string | null): string {
  let fenced = false
  for (const raw of (markdown ?? '').split('\n')) {
    const line = raw.trim()
    if (FENCE.test(line)) {
      fenced = !fenced
      continue
    }
    if (fenced || NOT_PROSE.test(line)) continue
    const text = plainText(line)
    if (text !== '') return text
  }
  return ''
}

function checklistOf(steps: readonly TaskStep[]): TaskStep[] {
  return steps.filter((step) => step.state !== 'DRAFT').sort((a, b) => a.position - b.position)
}

/** Where a flow stands, else the first step still open. */
function nextStep(checklist: readonly TaskStep[]): TaskStep | undefined {
  const open = checklist.filter((step) => step.state === 'OPEN')
  return open.find((step) => step.visits.some((visit) => visit.leftAt === null)) ?? open[0]
}

/** The line of one task, from its own steps: what is running, else what waits for review, else what comes next. */
export function stepLine(row: TaskRow, steps: readonly TaskStep[]): TaskStepLine {
  const checklist = checklistOf(steps)
  const summary = summaryOf(row.task.description)
  if (row.running) {
    const running = checklist.find((step) => step.state === 'RUNNING')
    return { tone: 'running', lead: 'Working', text: running?.title ?? summary }
  }
  if (row.needs > 0) {
    const claimed = checklist.find((step) => step.state === 'CLAIMED')
    return { tone: 'claimed', lead: 'Claimed', text: claimed?.title ?? summary }
  }
  const next = row.kind === 'filed' ? undefined : nextStep(checklist)
  if (next) return { tone: 'next', lead: `Step ${checklist.indexOf(next) + 1}`, text: next.title }
  return { tone: 'summary', lead: null, text: summary }
}
