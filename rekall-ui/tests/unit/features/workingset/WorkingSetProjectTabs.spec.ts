import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import WorkingSetProjectTabs from '@/features/workingset/components/WorkingSetProjectTabs.vue'
import { groupByProject } from '@/features/workingset/lib/group-by-project'
import type { ProjectId, TaskId } from '@/model/branded'
import { company, open, project, task } from './fixtures'

const GROUPS = groupByProject(
  [open('t1', 'vega'), open('t2', 'vega'), open('t3', 'ledger')],
  [company('c2', 'Northwind'), company('c1', 'Helix Labs')],
  [project('vega', 'Vega', 'c2'), project('ledger', 'Ledger', 'c1')],
  [task('t1', 'vega'), task('t2', 'vega'), task('t3', 'ledger')]
)

function tabs(reviewCounts: ReadonlyMap<TaskId, number> = new Map(), currentProjectId: ProjectId | null = null) {
  return mount(WorkingSetProjectTabs, { props: { groups: GROUPS, currentProjectId, reviewCounts } })
}

describe('the project tab row', () => {
  it('shows a tab per project under the company, companies by name, with the open count', () => {
    const row = tabs()

    expect(row.findAll('[data-testid="working-set-company"]').map((caption) => caption.text())).toEqual(['Helix Labs', 'Northwind'])
    expect(row.findAll('[data-testid="working-set-project"]').map((tab) => tab.get('[data-testid="working-set-open-count"]').text())).toEqual(['1', '2'])
  })

  it('shows the review count only for a project with work waiting', () => {
    const row = tabs(new Map([['t2' as TaskId, 2]]))

    const counts = row.findAll('[data-testid="working-set-project"]').map((tab) => tab.find('[data-testid="working-set-review-count"]').exists())
    expect(counts).toEqual([false, true])
    expect(row.findAll('[data-testid="working-set-project"]')[1]?.attributes('aria-label')).toBe('Vega, 2 open, 2 to review')
  })

  it('marks the current project and emits the id of the tab pressed', async () => {
    const row = tabs(new Map(), 'vega' as ProjectId)

    const [ledger, vega] = row.findAll('[data-testid="working-set-project"]')
    expect([ledger?.attributes('aria-selected'), vega?.attributes('aria-selected')]).toEqual(['false', 'true'])
    await ledger?.trigger('click')

    expect(row.emitted('select')).toEqual([['ledger']])
  })
})
