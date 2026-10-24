import type { DeploymentGraphId, ProjectSectionId } from '@/model/branded'

/**
 * The page open on a project. A section or graph id of `null` is its tab with nothing open yet:
 * the project has none, or the one that was open is gone.
 */
export type ProjectPage =
  | { readonly kind: 'blueprint' }
  | { readonly kind: 'section'; readonly id: ProjectSectionId | null }
  | { readonly kind: 'graph'; readonly id: DeploymentGraphId | null }
  | { readonly kind: 'graph-setup' }
  | { readonly kind: 'components' }
  | { readonly kind: 'connections' }
  | { readonly kind: 'settings' }

export type ProjectTab = 'blueprint' | 'sections' | 'graphs' | 'components' | 'connections' | 'settings'

export const BLUEPRINT: ProjectPage = { kind: 'blueprint' }

export function isSection(page: ProjectPage, id: ProjectSectionId): boolean {
  return page.kind === 'section' && page.id === id
}

export function tabOf(page: ProjectPage): ProjectTab {
  switch (page.kind) {
    case 'section':
      return 'sections'
    case 'graph':
    case 'graph-setup':
      return 'graphs'
    default:
      return page.kind
  }
}
