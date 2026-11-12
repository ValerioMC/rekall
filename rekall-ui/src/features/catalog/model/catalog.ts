import type { CompanyId, ProjectId, TagId, TaskId } from '../../../model/branded'
import type { TaskStepState } from '../../steps/model/task-step'
import type { ProjectStatus } from './project-status'
import type { TaskStatus } from './task-status'

/** An epic holds no steps: it gathers the tasks of its project that name it as their epic. */
export const TASK_KINDS = ['TASK', 'EPIC'] as const

export type TaskKind = (typeof TASK_KINDS)[number]

export interface Company {
  readonly id: CompanyId
  readonly name: string
  readonly description: string | null
  /** Set when the company has an uploaded logo; changes with every upload. */
  readonly logoImageVersion: string | null
  readonly projectCount: number
  readonly taskCount: number
  readonly updatedAt: string
}

export interface Project {
  readonly id: ProjectId
  readonly label: string
  readonly title: string
  readonly status: ProjectStatus
  readonly icon: string
  /** Set when the project has an uploaded picture; changes with every upload, so it keys the cache. */
  readonly iconImageVersion: string | null
  readonly description: string | null
  readonly blueprintMarkdown: string | null
  readonly repoFolder: string | null
  /** A session's claim commits the repo folder and logs it; only ever true on a git repository. */
  readonly autoCommit: boolean
  /** The branch accepted work is merged into and a task's branch starts from; null is the folder's current branch. */
  readonly mainBranch: string | null
  /** Accepted work reaches the main branch through a pull request; only ever true with a GitHub or GitLab task manager. */
  readonly pullRequestFlow: boolean
  readonly companyId: CompanyId
  readonly companyName: string
  readonly taskCount: number
  readonly anchor: string
  readonly updatedAt: string
}

/** The project's folder as git sees it: whether auto-commit can be offered, and as whom it would commit. */
export interface ProjectRepository {
  readonly folder: string | null
  readonly exists: boolean
  readonly repository: boolean
  readonly branch: string | null
  readonly userName: string | null
  readonly userEmail: string | null
  readonly autoCommit: boolean
}

export interface Task {
  readonly id: TaskId
  readonly label: string
  readonly title: string
  readonly status: TaskStatus
  readonly description: string | null
  readonly projectId: ProjectId
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyName: string
  readonly projectRepoFolder: string | null
  readonly documentCount: number
  readonly stepCount: number
  readonly stepsDone: number
  readonly draftStepCount: number
  readonly hasWrapup: boolean
  readonly reviewState: TaskStepState
  readonly reviewActive: boolean
  readonly claimedAt: string | null
  readonly acceptedAt: string | null
  readonly tagId: TagId | null
  readonly tagName: string | null
  readonly tagIcon: string | null
  readonly tagColor: string | null
  /** When a deployment graph was laid on the task; a task is laid once. */
  readonly graphLaidAt: string | null
  /** The project's release graph is not laid when the console runs this task. */
  readonly releaseSkipped: boolean
  readonly kind: TaskKind
  /** The epic of the same project this task belongs to, and inherits description and notes from. */
  readonly epicId: TaskId | null
  /** Started from one line of text; Rekall accepts it when the session writes the wrapup. */
  readonly quick: boolean
  /** Accepted by Rekall when the wrapup landed, not by a person. Cleared when it is sent back. */
  readonly autoAccepted: boolean
  /** The tasks of the same project this one waits for before the run queue starts it. */
  readonly dependsOn: readonly TaskId[]
  /** The issue the project's task manager holds for this task: Jira's key, GitLab's iid or GitHub's number. */
  readonly trackerIssueKey: string | null
  readonly trackerIssueUrl: string | null
  /** Why the last write to the task manager failed; the next change to the task tries again. */
  readonly trackerSyncError: string | null
  /** The pull request or merge request of the task's branch, on a project whose pull request flow is on. */
  readonly pullRequestNumber: number | null
  readonly pullRequestUrl: string | null
  /** Why the request was not opened or not merged; the next claim or move of the task tries again. */
  readonly pullRequestFailure: string | null
  readonly pullRequestMergedAt: string | null
  readonly anchor: string
  readonly updatedAt: string
}

export function isEpic(task: Pick<Task, 'kind'>): boolean {
  return task.kind === 'EPIC'
}

export interface Tag {
  readonly id: TagId
  readonly name: string
  readonly icon: string
  readonly color: string
  readonly updatedAt: string
}

export interface TaskRef {
  readonly id: TaskId
  readonly label: string
  readonly title: string
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyName: string
  readonly anchor: string
}

export function slugify(raw: string): string {
  return raw
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/[._-]{2,}/g, '-')
    .replace(/^[._-]+|[._-]+$/g, '')
}

export function anchorOf(record: Company | Project | Task): string {
  return 'anchor' in record ? record.anchor : `company:${record.name}`
}
