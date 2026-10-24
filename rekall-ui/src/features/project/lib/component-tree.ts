import type { ProjectComponent } from '@/features/project/model/project-component'

/** A component as the tree shows it: its folder below the project's, or whole when it lies outside. */
export interface ComponentLeaf {
  readonly component: ProjectComponent
  readonly path: string
}

/**
 * The components that share a parent folder. `folder` is that parent below the project's folder
 * (`billing/`), `''` for the project's folder itself, and `null` for components outside it.
 */
export interface ComponentBranch {
  readonly folder: string | null
  readonly leaves: readonly ComponentLeaf[]
}

const withForwardSlashes = (path: string): string => path.trim().replace(/\\/g, '/').replace(/\/+$/, '')

/** The folder below `root`, `.` for the root itself, or `null` when it is not under it. */
export function relativeToRoot(folder: string, root: string): string | null {
  const base = withForwardSlashes(root)
  const path = withForwardSlashes(folder)
  if (path === base) return '.'
  return path.startsWith(`${base}/`) ? path.slice(base.length + 1) : null
}

function parentOf(relative: string): string {
  const cut = relative.lastIndexOf('/')
  return cut < 0 ? '' : `${relative.slice(0, cut)}/`
}

function leafOf(component: ProjectComponent, root: string | null): { branch: string | null; leaf: ComponentLeaf } {
  if (root === null) return { branch: '', leaf: { component, path: component.folder } }
  const relative = relativeToRoot(component.folder, root)
  if (relative === null) return { branch: null, leaf: { component, path: component.folder } }
  return { branch: parentOf(relative), leaf: { component, path: relative } }
}

/** The components grouped by parent folder, branches in the order their first component is listed, leaves in list order. */
export function componentBranches(components: readonly ProjectComponent[], root: string | null): ComponentBranch[] {
  const branches = new Map<string | null, ComponentLeaf[]>()
  for (const component of components) {
    const { branch, leaf } = leafOf(component, root)
    const leaves = branches.get(branch) ?? []
    leaves.push(leaf)
    branches.set(branch, leaves)
  }
  return [...branches].map(([folder, leaves]) => ({ folder, leaves }))
}

/** Whether a component's name, folder or description holds the query, ignoring case. A blank query matches all. */
export function componentMatches(component: ProjectComponent, query: string): boolean {
  const wanted = query.trim().toLowerCase()
  if (wanted === '') return true
  return [component.name, component.folder, component.description].some((text) => text.toLowerCase().includes(wanted))
}
