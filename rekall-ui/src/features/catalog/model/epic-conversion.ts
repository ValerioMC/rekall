import type { TaskId } from '@/model/branded'

/** What turning a task into an epic would delete: the parts of a task an epic cannot hold. */
export interface EpicConversion {
  readonly taskId: TaskId
  readonly steps: number
  readonly stepsDone: number
  readonly routes: number
  readonly hasWrapup: boolean
  readonly commits: number
  readonly deviations: number
  readonly waitsFor: number
  readonly neededBy: number
  readonly skills: number
  readonly hasPullRequest: boolean
  readonly epicCode: string | null
  readonly epicTitle: string | null
  readonly queued: boolean
}

export interface ConversionLoss {
  readonly key: string
  readonly what: string
  readonly detail: string
  readonly count: number
}

function plural(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? '' : 's'}`
}

function planLoss(conversion: EpicConversion): ConversionLoss[] {
  if (conversion.steps === 0) return []
  const routes = conversion.routes > 0 ? ` and the flow of ${plural(conversion.routes, 'route')} between them` : ''
  return [
    {
      key: 'plan',
      what: 'Plan',
      detail: `${plural(conversion.steps, 'step')} (${conversion.stepsDone} done)${routes}`,
      count: conversion.steps
    }
  ]
}

function dependencyLoss(conversion: EpicConversion): ConversionLoss[] {
  const count = conversion.waitsFor + conversion.neededBy
  if (count === 0) return []
  const parts = [
    conversion.waitsFor > 0 ? `waits for ${plural(conversion.waitsFor, 'task')}` : '',
    conversion.neededBy > 0 ? `${plural(conversion.neededBy, 'task')} wait${conversion.neededBy === 1 ? 's' : ''} for it` : ''
  ].filter(Boolean)
  return [{ key: 'dependencies', what: 'Dependencies', detail: parts.join('; '), count }]
}

function countedLoss(key: string, what: string, detail: string, count: number): ConversionLoss[] {
  return count > 0 ? [{ key, what, detail, count }] : []
}

function flagLoss(key: string, what: string, detail: string, present: boolean): ConversionLoss[] {
  return present ? [{ key, what, detail, count: 1 }] : []
}

/** The rows of the warning, in the order the work goes. A part the task does not have is not listed. */
export function conversionLosses(conversion: EpicConversion): readonly ConversionLoss[] {
  return [
    ...planLoss(conversion),
    ...flagLoss('wrapup', 'Wrapup', 'What the implementation looks like now', conversion.hasWrapup),
    ...countedLoss('commits', 'Commits', `${plural(conversion.commits, 'logged commit')}; git keeps the commits`, conversion.commits),
    ...countedLoss('deviations', 'Deviations', 'They measure a plan, and the plan goes', conversion.deviations),
    ...dependencyLoss(conversion),
    ...countedLoss('skills', 'Skills', `${plural(conversion.skills, 'skill')} assigned to the task and its steps`, conversion.skills),
    ...flagLoss('pull-request', 'Pull request', 'The link goes; the request stays on the task manager', conversion.hasPullRequest),
    ...flagLoss('epic', 'Its own epic', `Leaves ${conversion.epicTitle ?? ''}; an epic belongs to none`, conversion.epicCode !== null)
  ]
}
