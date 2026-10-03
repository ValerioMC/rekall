import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import NavigatorTaskRow from '@/components/console/NavigatorTaskRow.vue'
import SelectionTrail from '@/components/console/SelectionTrail.vue'
import type { Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

const task: Task = {
  id: 't1' as TaskId,
  label: 'settlement',
  title: 'Settlement',
  status: 'IN_PROGRESS',
  description: null,
  projectId: 'p1' as ProjectId,
  projectLabel: 'vega',
  projectTitle: 'Vega',
  companyName: 'Acme',
  projectRepoFolder: null,
  documentCount: 0,
  stepCount: 0,
  stepsDone: 0,
  draftStepCount: 0,
  hasWrapup: false,
  reviewState: 'OPEN',
  reviewActive: false,
  claimedAt: null,
  acceptedAt: null,
  reviewNote: null,
  tagId: null,
  tagName: null,
  tagIcon: null,
  tagColor: null,
  anchor: 'project:vega task:settlement',
  updatedAt: '2026-10-03T00:00:00Z'
}

describe('SelectionTrail', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('stacks dashes of shrinking length so the tail fades behind one head', () => {
    const lengths = mount(SelectionTrail)
      .findAll('rect')
      .map((rect) => Number(rect.attributes('stroke-dasharray')?.split(' ')[0]))

    expect(lengths.length).toBeGreaterThan(3)
    expect(lengths).toEqual([...lengths].sort((a, b) => b - a))
  })

  it('circles only the selected task row', () => {
    const props = { task, running: false, showContext: false, showCompany: false }

    const selected = mount(NavigatorTaskRow, { props: { ...props, selected: true } })
    const idle = mount(NavigatorTaskRow, { props: { ...props, selected: false } })

    expect(selected.find('[data-testid="selection-trail"]').exists()).toBe(true)
    expect(idle.find('[data-testid="selection-trail"]').exists()).toBe(false)
  })
})
