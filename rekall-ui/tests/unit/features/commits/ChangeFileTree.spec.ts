import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ChangeFileTree from '@/features/commits/components/ChangeFileTree.vue'
import type { RepositoryRef, ReviewFile } from '@/features/commits/lib/change-sources'

const project: RepositoryRef = { id: 'project', name: 'vega', position: 0 }
const site: RepositoryRef = { id: 'site', name: 'site', position: 1 }

function file(path: string, repository: RepositoryRef = project): ReviewFile {
  return { id: `${repository.id}::${path}`, repository, path, oldPath: null, kind: 'MODIFIED', binary: false, additions: 2, deletions: 1, hunks: [], sources: [] }
}

const FILES = [file('core/src/commit/model/a.rs'), file('core/src/commit/model/b.rs'), file('core/src/lib.rs'), file('README.md')]

function render(files: readonly ReviewFile[] = FILES, current: string | null = null) {
  return mount(ChangeFileTree, {
    attachTo: document.body,
    props: { files, inScope: new Map(files.map((entry) => [entry.id, entry])), viewed: new Set<string>(), current }
  })
}

const names = (wrapper: ReturnType<typeof render>) => wrapper.findAll('[role="treeitem"]').map((row) => row.text().replace(/\s+/g, ' ').replace(/\+\d+ −\d+/, '').trim())

describe('ChangeFileTree', () => {
  it('shows a path of folders holding nothing else as one element, folders before files', () => {
    const wrapper = render()

    expect(names(wrapper)).toEqual(['core/src', 'commit/model', 'a.rs', 'b.rs', 'lib.rs', 'README.md'])
  })

  it('closes a folder with its chevron row and opens it again', async () => {
    const wrapper = render()

    await wrapper.findAll('[data-testid="change-folder-row"]')[1]!.trigger('click')
    expect(names(wrapper)).toEqual(['core/src', 'commit/model', 'lib.rs', 'README.md'])
    expect(wrapper.findAll('[data-testid="change-folder-row"]')[1]!.attributes('aria-expanded')).toBe('false')

    await wrapper.findAll('[data-testid="change-folder-row"]')[1]!.trigger('click')
    expect(names(wrapper)).toHaveLength(6)
  })

  it('sums the lines of every file below a folder in its row', () => {
    const wrapper = render()

    expect(wrapper.findAll('[data-testid="change-folder-lines"]')[1]!.text()).toBe('+4 −2')
  })

  it('opens the folders above the file that becomes current', async () => {
    const wrapper = render()
    await wrapper.findAll('[data-testid="change-folder-row"]')[0]!.trigger('click')
    expect(names(wrapper)).toEqual(['core/src', 'README.md'])

    await wrapper.setProps({ current: 'project::core/src/commit/model/b.rs' })

    expect(names(wrapper)).toContain('b.rs')
  })

  it('selects a file by its id when its row is pressed', async () => {
    const wrapper = render()

    await wrapper.findAll('[data-testid="change-file-row"]')[0]!.trigger('click')

    expect(wrapper.emitted('select')).toEqual([['project::core/src/commit/model/a.rs']])
  })

  it('moves between rows with the arrow keys and closes or opens a folder with left and right', async () => {
    const wrapper = render()
    const rows = wrapper.findAll<HTMLElement>('[role="treeitem"]')
    rows[0]!.element.focus()

    await rows[0]!.trigger('keydown', { key: 'ArrowDown' })
    expect(document.activeElement).toBe(rows[1]!.element)

    await rows[1]!.trigger('keydown', { key: 'ArrowLeft' })
    expect(names(wrapper)).toEqual(['core/src', 'commit/model', 'lib.rs', 'README.md'])

    await rows[1]!.trigger('keydown', { key: 'ArrowRight' })
    expect(names(wrapper)).toHaveLength(6)
  })

  it('keeps a folder of the same path in two repositories apart, each under its own heading', async () => {
    const wrapper = render([file('src/a.ts'), file('src/a.ts', site)])

    expect(wrapper.findAll('[data-testid="change-tree-repository-name"]').map((heading) => heading.text())).toEqual([expect.stringContaining('vega'), expect.stringContaining('site')])
    await wrapper.findAll('[data-testid="change-folder-row"]')[0]!.trigger('click')
    expect(names(wrapper)).toEqual(['src', 'src', 'a.ts'])
  })
})
