import type { ConnectionKind, ProjectConnection } from '@/features/connection/model/connection'
import type { DraftStep, GraphDraft } from '@/features/graph/model/deployment-graph'
import type { ConnectionId } from '@/model/branded'

export type StepPresetId = 'push' | 'build' | 'deploy' | 'check' | 'fix' | 'blank'

/** A kind of step the editor adds in one click: its words, and the connection kind it reaches. */
export interface StepPreset {
  readonly id: StepPresetId
  readonly label: string
  readonly title: string
  readonly detail: string
  readonly kind: ConnectionKind | null
}

export const STEP_PRESETS: readonly StepPreset[] = [
  {
    id: 'push',
    label: 'Push commit',
    title: 'Push commit',
    kind: 'GITLAB',
    detail: `Push what is committed on this task to the project's repository through this step's connection. The steps before this one committed it when they were claimed, on a project that commits on claim; on one that does not, commit it first.

Claim \`pushed\` once the push is accepted.`
  },
  {
    id: 'build',
    label: 'Pipeline build',
    title: 'Pipeline build',
    kind: 'GITLAB',
    detail: `Follow the pipeline the push started on this step's connection until it finishes.

Claim \`passed\` when every job succeeds. Claim \`failed\` when one fails, naming the job and why: the flow goes to the fix.`
  },
  {
    id: 'deploy',
    label: 'Deploy',
    title: 'Deploy',
    kind: 'KUBERNETES',
    detail: `Roll the build the pipeline produced out to this step's cluster and namespace, and wait for the rollout to complete.

Claim \`deployed\` once it is complete, or \`failed\` with what stopped it: the flow goes to the fix.`
  },
  {
    id: 'check',
    label: 'Check',
    title: 'Check',
    kind: 'KUBERNETES',
    detail: `Check the deployment on this step's cluster: pods ready and not restarting, logs free of errors, the application answering.

Claim \`healthy\` when it is, which ends the flow. Claim \`error\` with what you saw: the flow goes to the fix.`
  },
  {
    id: 'fix',
    label: 'Fix',
    title: 'Fix',
    kind: null,
    detail: `A later step failed. Read what it reported, find the cause and fix it in the repository: the application code, its tests, the Dockerfile, the manifests or the pipeline definition. Keep the fix to what the failure needs.

Claim \`fixed\` once the fix is in: the flow goes back to the push. Claim \`blocked\`, saying what you saw, when the repository cannot fix it (the cluster is down, a credential is missing, the runner is broken) or the fix belongs to another task, which you open with \`rekall_create_task\`: the flow ends there for a person to decide.

Never change the cluster or the pipeline's settings by hand: a fix that is not in git is lost on the next deploy.`
  },
  { id: 'blank', label: 'Blank step', title: 'New step', kind: null, detail: '' }
]

/** The first connection of `kind` the project binds, or none. */
export function connectionFor(kind: ConnectionKind | null, bindings: readonly ProjectConnection[]): ConnectionId | null {
  if (kind === null) return null
  return bindings.find((binding) => binding.kind === kind)?.connectionId ?? null
}

/** A new step from the preset, keyed `key`, reaching the project's first connection of its kind. */
export function stepFrom(preset: StepPreset, key: string, bindings: readonly ProjectConnection[]): DraftStep {
  return {
    key,
    title: preset.title,
    detail: preset.detail,
    connectionId: connectionFor(preset.kind, bindings),
    maxVisits: null,
    flowX: null,
    flowY: null,
    routes: []
  }
}

/** A title the draft's steps do not use yet: the preset's, numbered when taken. */
export function freeTitle(title: string, steps: readonly DraftStep[]): string {
  const taken = new Set(steps.map((step) => step.title.trim().toLowerCase()))
  if (!taken.has(title.toLowerCase())) return title
  let number = 2
  while (taken.has(`${title} ${number}`.toLowerCase())) number += 1
  return `${title} ${number}`
}

function preset(id: StepPresetId): StepPreset {
  return STEP_PRESETS.find((candidate) => candidate.id === id)!
}

/** How many times the loop may enter the fix before the flow stops to report. */
const RELEASE_RETRIES = 3

/**
 * Push, build, deploy, check, each on the project's first connection of its kind. A failed build
 * or deploy and an unhealthy check go to the fix, which may be entered three times and goes back
 * to the push.
 */
export function releaseLoop(title: string, bindings: readonly ProjectConnection[]): GraphDraft {
  const [push, build, deploy, check, fix] = (['push', 'build', 'deploy', 'check', 'fix'] as const).map((id) =>
    stepFrom(preset(id), id, bindings)
  )
  return {
    title,
    description: 'Push, build, deploy and check; a failure goes to the fix, then back to the push.',
    steps: [
      { ...push!, routes: [{ outcome: 'pushed', toKey: 'build' }] },
      { ...build!, routes: [{ outcome: 'passed', toKey: 'deploy' }, { outcome: 'failed', toKey: 'fix' }] },
      { ...deploy!, routes: [{ outcome: 'deployed', toKey: 'check' }, { outcome: 'failed', toKey: 'fix' }] },
      { ...check!, routes: [{ outcome: 'healthy', toKey: null }, { outcome: 'error', toKey: 'fix' }] },
      { ...fix!, maxVisits: RELEASE_RETRIES, routes: [{ outcome: 'fixed', toKey: 'push' }, { outcome: 'blocked', toKey: null }] }
    ]
  }
}

/** A graph of one blank step, to draw from scratch. */
export function blankGraph(title: string): GraphDraft {
  return { title, description: '', steps: [stepFrom(preset('blank'), 'first', [])] }
}
