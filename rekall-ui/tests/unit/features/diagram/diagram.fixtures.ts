import type { DiagramSummary } from '@/features/diagram/model/diagram'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { CompanyId, DiagramId, ProjectId, TaskId } from '@/model/branded'

/** Catalog and diagram records the diagram screen's tests share. */

export const VEGA = 'p-vega' as ProjectId
export const ORION = 'p-orion' as ProjectId
export const REPORT = 't-report' as TaskId

export function project(id: ProjectId, label: string, repoFolder: string | null = '/code/' + label): Project {
  return {
    id,
    label,
    title: label[0]!.toUpperCase() + label.slice(1),
    status: 'ACTIVE',
    icon: 'folder',
    iconImageVersion: null,
    description: null,
    blueprintMarkdown: null,
    repoFolder,
    autoCommit: false,
    companyId: 'c-acme' as CompanyId,
    companyName: 'acme',
    taskCount: 1,
    anchor: `project:${label}`,
    updatedAt: '2026-10-01T10:00:00Z'
  }
}

export function task(id: TaskId, label: string, projectId: ProjectId): Task {
  return {
    id,
    label,
    title: 'Report builder',
    status: 'IN_PROGRESS',
    description: null,
    projectId,
    projectLabel: 'vega',
    projectTitle: 'Vega',
    companyName: 'acme',
    projectRepoFolder: '/code/vega',
    documentCount: 0,
    stepCount: 0,
    stepsDone: 0,
    draftStepCount: 0,
    hasWrapup: false,
    reviewState: 'OPEN',
    reviewActive: false,
    claimedAt: null,
    acceptedAt: null,
    reviewNote: null,
    tagId: null,
    tagName: null,
    tagIcon: null,
    tagColor: null,
    anchor: `project:vega task:${label}`,
    updatedAt: '2026-10-01T10:00:00Z'
  }
}

export function summary(id: string, projectId: ProjectId, taskId: TaskId | null = null): DiagramSummary {
  return {
    id: id as DiagramId,
    projectId,
    taskId,
    title: `Diagram ${id}`,
    question: 'How does it work?',
    kinds: ['sequence', 'state'],
    walkthrough: false,
    createdAt: '2026-10-01T10:00:00Z',
    updatedAt: '2026-10-01T10:00:00Z'
  }
}
