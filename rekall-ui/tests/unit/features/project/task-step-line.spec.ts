import { describe, expect, it } from 'vitest'
import { stepLine, summaryOf } from '@/features/project/lib/task-step-line'
import { taskRows } from '@/features/task/lib/task-rows'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { StepVisitId } from '@/model/branded'
import { step, task } from './project-task-fixtures'

function lineOf(subject: Task, steps: TaskStep[] = []) {
  return stepLine(taskRows([subject], steps, () => 0)[0]!, steps)
}

const at = (id: string, state: TaskStep['state'], position: number): TaskStep => ({ ...step(id, 'RK-1', state), position })

describe('summaryOf', () => {
  it('passes over headings, tables, pictures and code to the first line of prose', () => {
    const markdown = '## Goal\n\n![shot](a.png)\n```sh\nmake\n```\n| a | b |\n- [ ] Make the **pager** [work](x) with `pageOf`'

    expect(summaryOf(markdown)).toBe('Make the pager work with pageOf')
  })

  it('keeps the name a reference writes', () => {
    expect(summaryOf('> Touches [[rekall-ui]] only')).toBe('Touches rekall-ui only')
  })

  it('is empty for no description', () => {
    expect(summaryOf(null)).toBe('')
    expect(summaryOf('# Title only')).toBe('')
  })
})

describe('stepLine', () => {
  it('names the running step', () => {
    const line = lineOf(task('RK-1', { stepCount: 2 }), [at('s1', 'CLAIMED', 1), at('s2', 'RUNNING', 2)])

    expect(line).toEqual({ tone: 'running', lead: 'Working', text: 'Step s2' })
  })

  it('names the first claimed step', () => {
    const line = lineOf(task('RK-1', { stepCount: 2 }), [at('s2', 'CLAIMED', 2), at('s1', 'CLAIMED', 1)])

    expect(line).toEqual({ tone: 'claimed', lead: 'Claimed', text: 'Step s1' })
  })

  it('falls back to the description for a task handed in whole', () => {
    const line = lineOf(task('RK-1', { reviewActive: true, reviewState: 'CLAIMED', description: 'Fix it' }))

    expect(line).toEqual({ tone: 'claimed', lead: 'Claimed', text: 'Fix it' })
  })

  it('numbers the next open step by its place in the checklist, drafts aside', () => {
    const line = lineOf(task('RK-1', { stepCount: 2 }), [at('d', 'DRAFT', 0), at('s1', 'DONE', 1), at('s2', 'OPEN', 2)])

    expect(line).toEqual({ tone: 'next', lead: 'Step 2', text: 'Step s2' })
  })

  it('follows a flow to the open step it stands on', () => {
    const standing: TaskStep = {
      ...at('s3', 'OPEN', 3),
      visits: [{ id: 'v1' as StepVisitId, stepId: 's3' as TaskStep['id'], ordinal: 1, enteredAt: '2026-09-20T09:00:00Z', leftAt: null, outcome: null, report: null }]
    }

    const line = lineOf(task('RK-1', { stepCount: 2 }), [at('s2', 'OPEN', 2), standing])

    expect(line.lead).toBe('Step 2')
    expect(line.text).toBe('Step s3')
  })

  it('shows the description of a filed task', () => {
    const line = lineOf(task('RK-1', { status: 'DONE', description: 'Shipped' }), [at('s1', 'OPEN', 1)])

    expect(line).toEqual({ tone: 'summary', lead: null, text: 'Shipped' })
  })
})
