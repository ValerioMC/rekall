import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectRow from '@/features/catalog/components/ProjectRow.vue'
import { NO_TOKENS } from '@/features/tokens/model/token-usage'
import { project } from '../project/project-task-fixtures'
import type { ProjectComposition } from '@/features/catalog/lib/project-composition'
import type { Project } from '@/features/catalog/model/catalog'

const QUIET: ProjectComposition = { filed: 0, review: 0, active: 0, todo: 0 }

function render(overrides: Partial<Project> = {}, composition: ProjectComposition = QUIET, reviewing = 0) {
  return mount(ProjectRow, {
    props: { project: project('vega', 'Vega', overrides), composition, reviewing, tokens: NO_TOKENS }
  })
}

describe('a project row', () => {
  it('names the project with its anchor and its status', () => {
    const row = render({ status: 'DONE' })

    expect(row.text()).toContain('Vega')
    expect(row.text()).toContain('project:vega')
    expect(row.find('[data-testid="status-pill"]').text()).toBe('Done')
    expect(row.find('[data-testid="status-pill"]').attributes('data-tone')).toBe('done')
  })

  it('says what waits for review and what is in progress', () => {
    const row = render({ taskCount: 6 }, { filed: 1, review: 2, active: 3, todo: 0 }, 2)

    expect(row.find('[data-testid="project-needs"]').text()).toBe('2 to review')
    expect(row.text()).toContain('6 tasks')
    expect(row.text()).toContain('3 in progress')
  })

  it('leaves out the review count and says so when there are no tasks', () => {
    const row = render({ taskCount: 0 })

    expect(row.find('[data-testid="project-needs"]').exists()).toBe(false)
    expect(row.text()).toContain('No tasks')
  })

  it('opens the project from the row and deletes it from the key alone', async () => {
    const row = render()

    await row.get('[data-testid="project-card"]').trigger('click')
    await row.get('[data-testid="delete-project"]').trigger('click')

    expect(row.emitted('open')).toHaveLength(1)
    expect(row.emitted('delete')).toHaveLength(1)
  })

  it('has no delete key where the row is not deletable', () => {
    const row = mount(ProjectRow, {
      props: { project: project('vega', 'Vega'), composition: QUIET, reviewing: 0, tokens: NO_TOKENS, deletable: false }
    })

    expect(row.find('[data-testid="delete-project"]').exists()).toBe(false)
  })
})
