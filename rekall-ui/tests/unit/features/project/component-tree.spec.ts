import { describe, expect, it } from 'vitest'
import { componentBranches, componentMatches, relativeToRoot } from '@/features/project/lib/component-tree'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId, ProjectId } from '@/model/branded'

function component(name: string, folder: string, description = ''): ProjectComponent {
  return {
    id: name as ProjectComponentId,
    projectId: 'p-1' as ProjectId,
    name,
    folder,
    description,
    position: 0,
    createdAt: '2026-10-24T10:00:00Z',
    updatedAt: '2026-10-24T10:00:00Z'
  }
}

const shape = (branches: ReturnType<typeof componentBranches>) =>
  branches.map((branch) => [branch.folder, branch.leaves.map((leaf) => leaf.path)])

describe('a folder below the project folder', () => {
  it('is the part after the root, whatever separators either side uses', () => {
    expect(relativeToRoot('/work/shop/billing/ledger', '/work/shop/')).toBe('billing/ledger')
    expect(relativeToRoot('C:\\work\\shop\\api', 'C:\\work\\shop')).toBe('api')
  })

  it('is the root itself as a dot, and nothing for a folder elsewhere or a sibling sharing the prefix', () => {
    expect(relativeToRoot('/work/shop/', '/work/shop')).toBe('.')
    expect(relativeToRoot('/elsewhere/api', '/work/shop')).toBeNull()
    expect(relativeToRoot('/work/shopfront', '/work/shop')).toBeNull()
  })
})

describe('the components tree', () => {
  it('hangs each component off its parent folder, in the order the components are listed', () => {
    const branches = componentBranches(
      [
        component('gateway', '/work/shop/gateway'),
        component('invoices', '/work/shop/billing/invoices'),
        component('orders', '/work/shop/orders'),
        component('ledger', '/work/shop/billing/ledger'),
        component('sdk', '/elsewhere/sdk')
      ],
      '/work/shop'
    )

    expect(shape(branches)).toEqual([
      ['', ['gateway', 'orders']],
      ['billing/', ['billing/invoices', 'billing/ledger']],
      [null, ['/elsewhere/sdk']]
    ])
  })

  it('keeps every component at the top, with its whole folder, when the project has no folder', () => {
    const branches = componentBranches([component('api', '/work/api'), component('ui', '/work/ui')], null)

    expect(shape(branches)).toEqual([['', ['/work/api', '/work/ui']]])
  })
})

describe('filtering components', () => {
  const ledger = component('ledger', '/work/shop/billing/ledger', 'Double-entry ledger')

  it('matches the name, the folder or the description, ignoring case', () => {
    expect(componentMatches(ledger, 'LEDG')).toBe(true)
    expect(componentMatches(ledger, 'billing')).toBe(true)
    expect(componentMatches(ledger, 'double-entry')).toBe(true)
    expect(componentMatches(ledger, 'search')).toBe(false)
  })

  it('lets everything through on a blank query', () => {
    expect(componentMatches(ledger, '   ')).toBe(true)
  })
})
