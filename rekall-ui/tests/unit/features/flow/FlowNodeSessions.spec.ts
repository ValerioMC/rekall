import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import FlowCanvas from '@/features/flow/components/FlowCanvas.vue'
import { NODE_HEIGHT } from '@/features/flow/lib/flow-layout'
import { SESSION_RIBBON_HEIGHT } from '@/features/flow/lib/session-marks'
import { readFlow } from '@/features/flow/lib/step-flow'
import type { TaskStep } from '@/features/catalog/model/catalog'
import { releaseFlow } from './flow.fixtures'

function canvasOf(steps: TaskStep[]) {
  return mount(FlowCanvas, { props: { flow: readFlow(steps), selectedId: null } })
}

function nodeOf(canvas: ReturnType<typeof canvasOf>, stepId: string) {
  return canvas.get(`[data-testid="flow-node"][data-step-id="${stepId}"]`)
}

function releaseFlowWithSessionAtTest(): TaskStep[] {
  return releaseFlow().map((step) => (step.id === 'test' ? { ...step, startsSession: true } : step))
}

describe('the session marks on the flow nodes', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('puts a ribbon on the step that opens a new session', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())
    const opener = nodeOf(canvas, 'test')

    expect(opener.get('[data-testid="flow-node-ribbon"]').text()).toBe('Session 2 · new Claude session')
    expect(opener.find('[data-testid="flow-node-session"]').exists()).toBe(false)
    expect(canvas.findAll('[data-testid="flow-node-ribbon"]')).toHaveLength(1)
  })

  it('gives every other step a chip with its session and a stripe in the session colour', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())

    expect(nodeOf(canvas, 'build').get('[data-testid="flow-node-session"]').text()).toBe('S1')
    expect(nodeOf(canvas, 'ship').get('[data-testid="flow-node-session"]').text()).toBe('S2')
    expect(canvas.findAll('[data-testid="flow-node-stripe"]')).toHaveLength(5)
    expect(nodeOf(canvas, 'build').attributes('style')).not.toBe(nodeOf(canvas, 'ship').attributes('style'))
  })

  it('says the session in the aria-label of each node', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())

    expect(nodeOf(canvas, 'test').attributes('aria-label')).toContain('session 2, opens a new session')
    expect(nodeOf(canvas, 'fix').attributes('aria-label')).toContain(
      'session 2, shares the session of the step before'
    )
    expect(nodeOf(canvas, 'build').attributes('aria-label')).toContain('session 1, where the checklist begins')
  })

  it('makes room for the ribbon in the box of the step that opens a session', () => {
    const canvas = canvasOf(releaseFlowWithSessionAtTest())

    expect(nodeOf(canvas, 'test').element.parentElement!.style.height).toBe(
      `${NODE_HEIGHT + SESSION_RIBBON_HEIGHT}px`
    )
    expect(nodeOf(canvas, 'fix').element.parentElement!.style.height).toBe(`${NODE_HEIGHT}px`)
  })

  it('marks nothing when the task runs in one session', () => {
    const canvas = canvasOf(releaseFlow())

    expect(canvas.findAll('[data-testid="flow-node"]')).toHaveLength(5)
    expect(canvas.find('[data-testid="flow-node-ribbon"]').exists()).toBe(false)
    expect(canvas.find('[data-testid="flow-node-session"]').exists()).toBe(false)
    expect(canvas.find('[data-testid="flow-node-stripe"]').exists()).toBe(false)
    expect(nodeOf(canvas, 'build').attributes('aria-label')).not.toContain('session')
    expect(nodeOf(canvas, 'build').element.parentElement!.style.height).toBe(`${NODE_HEIGHT}px`)
  })
})
