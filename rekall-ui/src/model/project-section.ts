import type { ProjectId, ProjectSectionId } from '@/model/branded'

/**
 * A titled markdown section on a project: what the blueprint does not hold. No context ever
 * loads one, so a person or a session can keep as much here as the project needs.
 */
export interface ProjectSection {
  readonly id: ProjectSectionId
  readonly projectId: ProjectId
  readonly title: string
  readonly bodyMarkdown: string
  readonly position: number
  readonly createdAt: string
  readonly updatedAt: string
}

/** Either field may be left out on an edit: the section keeps what it has. */
export interface ProjectSectionInput {
  readonly title?: string
  readonly bodyMarkdown?: string
}

export interface ProjectSectionStreamEvent {
  readonly projectId: ProjectId
}
