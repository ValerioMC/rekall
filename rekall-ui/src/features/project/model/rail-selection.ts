import type { DeploymentGraphId, ProjectSectionId } from '@/model/branded'

/** The page open on a project: its blueprint, one of its sections, one of its deployment graphs, or a new graph being set up. */
export type RailSelection =
  | { readonly kind: 'blueprint' }
  | { readonly kind: 'section'; readonly id: ProjectSectionId }
  | { readonly kind: 'graph'; readonly id: DeploymentGraphId }
  | { readonly kind: 'graph-setup' }

export const BLUEPRINT: RailSelection = { kind: 'blueprint' }

export function isSection(selection: RailSelection, id: ProjectSectionId): boolean {
  return selection.kind === 'section' && selection.id === id
}
