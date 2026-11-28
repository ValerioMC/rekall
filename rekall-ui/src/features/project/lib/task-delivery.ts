import { requestKindOf } from '@/features/catalog/lib/pull-request-request'
import type { Task } from '@/features/catalog/model/catalog'

export type RequestState = 'open' | 'merged' | 'failed'

export interface TaskDelivery {
  readonly issue: { readonly label: string; readonly url: string | null } | null
  readonly request: { readonly label: string; readonly url: string | null; readonly state: RequestState; readonly failure: string | null } | null
}

function issueOf(task: Task): TaskDelivery['issue'] {
  const key = task.trackerIssueKey
  if (key === null) return null
  return { label: /^\d+$/.test(key) ? `#${key}` : key, url: task.trackerIssueUrl }
}

function requestState(task: Task): RequestState {
  if (task.pullRequestMergedAt !== null) return 'merged'
  return task.pullRequestFailure === null ? 'open' : 'failed'
}

function requestOf(task: Task): TaskDelivery['request'] {
  if (task.pullRequestNumber === null && task.pullRequestFailure === null) return null
  const kind = requestKindOf(task.pullRequestUrl)
  const number = task.pullRequestNumber === null ? '' : ` ${kind.prefix}${task.pullRequestNumber}`
  return {
    label: `${kind.short}${number}`,
    url: task.pullRequestUrl,
    state: requestState(task),
    failure: task.pullRequestFailure
  }
}

/** The issue and the pull or merge request a task is delivered through, as far as it has either. */
export function deliveryOf(task: Task): TaskDelivery {
  return { issue: issueOf(task), request: requestOf(task) }
}

export function hasDelivery(task: Task): boolean {
  const delivery = deliveryOf(task)
  return delivery.issue !== null || delivery.request !== null
}
