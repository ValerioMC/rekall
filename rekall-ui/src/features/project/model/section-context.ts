import type { ProjectSection } from '@/features/project/model/project-section'
import type { ProjectId, ProjectSectionId, SectionContextId } from '@/model/branded'

/** A titled group of a project's sections: its pages are kept by what they are about. */
export interface SectionContext {
  readonly id: SectionContextId
  readonly projectId: ProjectId
  readonly title: string
  readonly position: number
  readonly createdAt: string
  readonly updatedAt: string
}

/** One context of an arrangement, with the pages it holds in order. */
export interface ContextPlacement {
  readonly contextId: SectionContextId
  readonly sectionIds: readonly ProjectSectionId[]
}

/** A project's contexts and sections as the server arranged them, each list in the order shown. */
export interface SectionLayout {
  readonly contexts: readonly SectionContext[]
  readonly sections: readonly ProjectSection[]
}
