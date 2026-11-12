import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import StepCheckResults from '@/features/check/components/StepCheckResults.vue'
import * as api from '@/features/check/api/checks.api'
import { useChecksStore } from '@/features/check/store/checks.store'
import { STEP, TASK, result } from './fixtures'
import type { CheckResult } from '@/features/check/model/check'
import type { TaskStep } from '@/features/steps/model/task-step'

vi.mock('@/features/check/api/checks.api')

const step = { id: STEP, taskId: TASK } as unknown as TaskStep

async function render(results: CheckResult[]) {
  vi.mocked(api.fetchTaskCheckResults).mockResolvedValue(results)
  await useChecksStore().loadResults(TASK)
  const wrapper = mount(StepCheckResults, { props: { step } })
  await flushPromises()
  return wrapper
}

describe('the results of a step\'s checks', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('shows nothing when no check ran for the step', async () => {
    const wrapper = await render([])

    expect(wrapper.find('[data-testid="step-check-results"]').exists()).toBe(false)
  })

  it('opens a failed run on its output and leaves a passed one closed', async () => {
    const wrapper = await render([
      result('r1', 'lint', true, '2026-11-03T10:01:00Z'),
      result('r2', 'tests', false, '2026-11-03T10:02:00Z')
    ])

    const runs = wrapper.findAll('[data-testid="step-check-run"]')
    expect(runs.map((run) => run.attributes('data-passed'))).toEqual(['true', 'false'])
    expect((runs[0]!.find('details').element as HTMLDetailsElement).open).toBe(false)
    expect((runs[1]!.find('details').element as HTMLDetailsElement).open).toBe(true)
    expect(runs[1]!.get('[data-testid="step-check-output"]').text()).toContain('export::pdf ... FAILED')
    expect(runs[1]!.get('[data-testid="step-check-outcome"]').text()).toContain('Failed, exit 101')
    expect(wrapper.get('[data-testid="step-check-summary"]').text()).toBe('1 of 2 failed')
  })

  it('says all passed when none failed', async () => {
    const wrapper = await render([result('r1', 'tests', true, '2026-11-03T10:01:00Z')])

    expect(wrapper.get('[data-testid="step-check-summary"]').text()).toBe('1 passed')
  })
})
