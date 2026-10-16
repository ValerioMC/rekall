import type { ProjectComponentId, ProjectId } from '@/model/branded'

/**
 * A folder of a project that is a part of it: a module, a service, an app. Every context that
 * loads the project carries its components, so a session knows where each part lives.
 */
export interface ProjectComponent {
  readonly id: ProjectComponentId
  readonly projectId: ProjectId
  readonly name: string
  readonly folder: string
  readonly description: string
  readonly position: number
  readonly createdAt: string
  readonly updatedAt: string
}

/** A field left out on an edit keeps what the component has; a name left out on a create is the folder's. */
export interface ProjectComponentInput {
  readonly name?: string
  readonly folder?: string
  readonly description?: string
}
