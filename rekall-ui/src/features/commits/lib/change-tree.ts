import type { ReviewFile } from '@/features/commits/lib/change-sources'

export interface TreeFolder {
  readonly kind: 'folder'
  /** The repository and the folder's full path, so two repositories can both have a `src`. */
  readonly id: string
  /** One or more path segments: a chain of folders holding nothing else reads as one element. */
  readonly name: string
  readonly files: readonly ReviewFile[]
  readonly children: readonly TreeNode[]
}

export interface TreeLeaf {
  readonly kind: 'file'
  readonly id: string
  readonly name: string
  readonly file: ReviewFile
}

export type TreeNode = TreeFolder | TreeLeaf

/** One line of the tree as shown: a node, how deep it sits, and whether a folder is open. */
export interface TreeRow {
  readonly node: TreeNode
  readonly depth: number
  readonly open: boolean
}

interface Draft {
  readonly name: string
  readonly path: string
  readonly folders: Map<string, Draft>
  readonly files: ReviewFile[]
}

/**
 * The files of one repository as a folder tree, folders first. A folder whose only content is one
 * folder joins it (`src/commit/model` is one element), the way an IDE compacts empty middle packages.
 */
export function buildChangeTree(repositoryId: string, files: readonly ReviewFile[]): readonly TreeNode[] {
  const root: Draft = { name: '', path: '', folders: new Map(), files: [] }
  for (const file of files) {
    const segments = file.path.split('/')
    segments.pop()
    placeIn(root, segments).files.push(file)
  }
  return nodesOf(repositoryId, root)
}

function placeIn(root: Draft, segments: readonly string[]): Draft {
  let folder = root
  for (const segment of segments) {
    const path = folder.path === '' ? segment : `${folder.path}/${segment}`
    const next = folder.folders.get(segment) ?? { name: segment, path, folders: new Map(), files: [] }
    folder.folders.set(segment, next)
    folder = next
  }
  return folder
}

function nodesOf(repositoryId: string, folder: Draft): readonly TreeNode[] {
  const folders = [...folder.folders.values()].sort((a, b) => a.name.localeCompare(b.name)).map((child) => folderNode(repositoryId, child))
  const leaves = [...folder.files].sort((a, b) => leafName(a).localeCompare(leafName(b))).map<TreeLeaf>((file) => ({ kind: 'file', id: file.id, name: leafName(file), file }))
  return [...folders, ...leaves]
}

function folderNode(repositoryId: string, start: Draft): TreeFolder {
  let draft = start
  let name = start.name
  while (draft.files.length === 0 && draft.folders.size === 1) {
    draft = [...draft.folders.values()][0]!
    name = `${name}/${draft.name}`
  }
  const children = nodesOf(repositoryId, draft)
  return { kind: 'folder', id: `${repositoryId}::${draft.path}/`, name, files: filesUnder(children), children }
}

function filesUnder(nodes: readonly TreeNode[]): readonly ReviewFile[] {
  return nodes.flatMap((node) => (node.kind === 'file' ? [node.file] : node.files))
}

function leafName(file: ReviewFile): string {
  return file.path.slice(file.path.lastIndexOf('/') + 1)
}

/** The tree flattened to the rows that show: the children of a closed folder are left out. */
export function visibleRows(nodes: readonly TreeNode[], closed: ReadonlySet<string>, depth = 0): readonly TreeRow[] {
  return nodes.flatMap<TreeRow>((node) => {
    if (node.kind === 'file') return [{ node, depth, open: false }]
    const open = !closed.has(node.id)
    return [{ node, depth, open }, ...(open ? visibleRows(node.children, closed, depth + 1) : [])]
  })
}

/** The ids of the folders a file sits in, outermost first: what must be open for the file to show. */
export function folderIdsAbove(nodes: readonly TreeNode[], fileId: string): readonly string[] {
  for (const node of nodes) {
    if (node.kind === 'file') {
      if (node.id === fileId) return []
      continue
    }
    if (!node.files.some((file) => file.id === fileId)) continue
    return [node.id, ...folderIdsAbove(node.children, fileId)]
  }
  return []
}
