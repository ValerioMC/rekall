import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FlowVisitBudget from '@/features/flow/components/FlowVisitBudget.vue'
import type { StepVisit } from '@/features/catalog/model/catalog'
import type { StepVisitId, TaskStepId } from '@/model/branded'
import { flowStep } from './flow.fixtures'

function visit(ordinal: number): StepVisit {
  return {
    id: `visit-${ordinal}` as StepVisitId,
    stepId: 'deploy' as TaskStepId,
    ordinal,
    enteredAt: '2026-10-01T09:00:00Z',
    leftAt: '2026-10-01T09:05:00Z',
    outcome: 'deployed',
    report: null
  }
}

/** A step's visit budget: typed and kept on blur, cleared to lift it, refused out of range. */
function render(maxVisits: number | null, entered = 0) {
  const step = { ...flowStep('deploy', 1, 'OPEN', [], Array.from({ length: entered }, (_, at) => visit(at + 1))), maxVisits }
  return mount(FlowVisitBudget, { props: { step, busy: false } })
}

async function type(budget: ReturnType<typeof render>, text: string): Promise<void> {
  const input = budget.get('[data-testid="flow-visit-budget-input"]')
  await input.setValue(text)
  await input.trigger('blur')
}

describe('FlowVisitBudget', () => {
  it('keeps a whole number in range', async () => {
    const budget = render(null)

    await type(budget, '3')

    expect(budget.emitted('limit')).toEqual([[3]])
  })

  it('lifts the budget when the field is cleared', async () => {
    const budget = render(2)

    await type(budget, '')

    expect(budget.emitted('limit')).toEqual([[null]])
  })

  it('snaps back on a value out of range or unchanged, and sends nothing', async () => {
    const budget = render(2)

    for (const text of ['0', '100', '2.5', '2']) await type(budget, text)

    expect(budget.emitted('limit')).toBeUndefined()
    expect((budget.get('[data-testid="flow-visit-budget-input"]').element as HTMLInputElement).value).toBe('2')
  })

  it('shows what is used and marks a spent budget', () => {
    const used = render(2, 2).get('[data-testid="flow-visit-budget-used"]')

    expect(used.text()).toBe('2 of 2 used')
    expect(used.classes()).toContain('text-danger')
  })
})
