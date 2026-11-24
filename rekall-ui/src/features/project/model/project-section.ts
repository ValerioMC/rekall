import type { ProjectId, ProjectSectionId, SectionContextId } from '@/model/branded'

/**
 * A titled markdown section on a project, kept in one of its contexts: what the blueprint does not
 * hold. No task context ever loads one, so a person or a session can keep as much here as the
 * project needs.
 */
export interface ProjectSection {
  readonly id: ProjectSectionId
  readonly projectId: ProjectId
  readonly contextId: SectionContextId
  readonly title: string
  readonly bodyMarkdown: string
  readonly position: number
  readonly createdAt: string
  readonly updatedAt: string
}

/** Any field may be left out on an edit: the section keeps what it has. On a create, no context puts it in the project's first. */
export interface ProjectSectionInput {
  readonly title?: string
  readonly bodyMarkdown?: string
  readonly contextId?: SectionContextId
}

export interface ProjectSectionStreamEvent {
  readonly projectId: ProjectId
}
