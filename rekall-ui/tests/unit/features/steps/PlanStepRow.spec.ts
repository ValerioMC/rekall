import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PlanStepRow from '@/features/steps/components/PlanStepRow.vue'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId, TaskStepId } from '@/model/branded'

const step: TaskStep = {
  id: 'step-1' as TaskStepId,
  taskId: 'task-1' as TaskId,
  title: 'Seed the demo world',
  bodyMarkdown: null,
  state: 'DONE',
  done: true,
  runningAt: null,
  claimedAt: null,
  doneAt: '2026-10-01T08:00:00Z',
  position: 0,
  flowX: null,
  flowY: null,
  maxVisits: null,
  startsSession: false,
  connectionId: null,
  passes: [],
  links: [],
  visits: [],
  createdAt: '2026-10-01T07:00:00Z',
  updatedAt: '2026-10-01T07:00:00Z'
}

describe('PlanStepRow', () => {
  it('anchors the seal hit area to the seal so it cannot cover the rest of the plan', () => {
    const seal = mount(PlanStepRow, {
      props: { step, number: 1, selected: false, isNext: false, allDone: true, isLast: true, canChooseSession: false }
    }).get('[data-testid="plan-step-seal"]')

    expect(seal.classes()).toContain('hit-target')
    expect(seal.classes()).toContain('relative')
  })
})
