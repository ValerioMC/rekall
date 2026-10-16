import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DiagramGallery from '@/features/diagram/components/gallery/DiagramGallery.vue'
import { ORION, REPORT, VEGA, project, summary, task } from './diagram.fixtures'
import type { PendingGeneration } from '@/features/diagram/store/diagram.store'
import type { TerminalId } from '@/model/branded'

const PENDING: PendingGeneration = { key: 1, projectId: VEGA, taskId: null, request: 'checkout', kinds: ['sequence'], terminalId: 'term-1' as TerminalId, startedAt: Date.now() }

function gallery(props: Partial<InstanceType<typeof DiagramGallery>['$props']> = {}) {
  return mount(DiagramGallery, {
    props: {
      summaries: [summary('d1', VEGA, REPORT), summary('d2', ORION), summary('d3', 'p-elsewhere' as never)],
      projects: [project(VEGA, 'vega'), project(ORION, 'orion')],
      tasks: [task(REPORT, 'report-builder', VEGA)],
      pending: [],
      endedTerminals: new Set<string>(),
      arrivedId: null,
      loaded: true,
      ...props
    }
  })
}

describe('the diagram gallery', () => {
  it('lists the diagrams of the projects in scope, one section per project', () => {
    const wrapper = gallery()
    const cards = wrapper.findAll('[data-testid="diagram-card"]')
    expect(cards.map((card) => card.text().slice(0, 10))).toEqual(['Diagram d1', 'Diagram d2'])
    expect(cards[0]!.text()).toContain('task:report-builder')
  })

  it('opens a diagram from its card', async () => {
    const wrapper = gallery()
    await wrapper.find('[data-testid="diagram-card"]').trigger('click')
    expect(wrapper.emitted('open')).toEqual([['d1']])
  })

  it('shows what is being drawn, and says so when its session ended without a diagram', async () => {
    const drawing = gallery({ pending: [PENDING] })
    expect(drawing.find('[data-testid="generation-card"]').attributes('aria-busy')).toBe('true')
    await drawing.find('[data-testid="generation-watch"]').trigger('click')
    expect(drawing.emitted('watch')).toEqual([[PENDING]])

    const ended = gallery({ pending: [PENDING], endedTerminals: new Set(['term-1']) })
    expect(ended.text()).toContain('Session ended without a diagram')
    expect(ended.find('[data-testid="generation-watch"]').exists()).toBe(false)
  })

  it('invites a first diagram when there is none', () => {
    const wrapper = gallery({ summaries: [] })
    expect(wrapper.find('[data-testid="empty-state"]').exists()).toBe(true)
  })
})
