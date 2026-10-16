import { describe, expect, it } from 'vitest'
import type { Box } from '@/features/diagram/lib/viewport'
import { FLOW_END } from '@/features/flow/lib/route-target'
import { targetAt } from '@/features/flow/composables/useFlowDrag'
import type { TaskStepId } from '@/model/branded'

const boxes = new Map<TaskStepId, Box>([
  ['a' as TaskStepId, { x: 0, y: 0, width: 100, height: 50 }],
  ['b' as TaskStepId, { x: 0, y: 100, width: 100, height: 50 }]
])
const end: Box = { x: 0, y: 200, width: 100, height: 40 }

describe('what a route being drawn would land on', () => {
  it('lands on the step under the pointer', () => {
    expect(targetAt({ x: 50, y: 120 }, 'a' as TaskStepId, boxes, end)).toBe('b')
  })

  it('lands on End when the pointer is over it', () => {
    expect(targetAt({ x: 10, y: 210 }, 'a' as TaskStepId, boxes, end)).toBe(FLOW_END)
  })

  it('never lands on the step the route leaves', () => {
    expect(targetAt({ x: 50, y: 20 }, 'a' as TaskStepId, boxes, end)).toBeNull()
  })

  it('lands on nothing over empty canvas', () => {
    expect(targetAt({ x: 300, y: 300 }, 'a' as TaskStepId, boxes, end)).toBeNull()
  })
})
