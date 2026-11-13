export const TASK_TABS = ['plan', 'wrapup', 'notes', 'description', 'time', 'deviations'] as const

export type TaskTab = (typeof TASK_TABS)[number]

export const TASK_TAB_LABEL: Readonly<Record<TaskTab, string>> = {
  plan: 'Plan',
  wrapup: 'Wrapup',
  notes: 'Notes',
  description: 'Description',
  time: 'Time',
  deviations: 'Deviations'
}

/** The tab a path segment names; a missing or unknown one is the Plan. */
export function taskTabOf(segment: string | undefined): TaskTab {
  return TASK_TABS.find((tab) => tab === segment) ?? 'plan'
}
