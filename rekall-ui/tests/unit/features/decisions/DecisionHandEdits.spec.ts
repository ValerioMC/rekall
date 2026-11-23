import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/decisions/api/decisions.api'
import DecisionAddForm from '@/features/decisions/components/DecisionAddForm.vue'
import DecisionRowActions from '@/features/decisions/components/DecisionRowActions.vue'
import { useConsoleStore } from '@/stores/console.store'
import { answered, decision, TASK } from './fixtures'

vi.mock('@/features/decisions/api/decisions.api')

async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve))
}

describe('correcting a decision by hand', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
  })

  it('opens the form on Edit with the answer as it stands, and closes it on Cancel', async () => {
    const page = mount(DecisionRowActions, { props: { decision: answered('replicas', '3') } })

    await page.get('[data-testid="decision-edit"]').trigger('click')
    expect((page.get('[data-testid="decision-edit-answer"]').element as HTMLTextAreaElement).value).toBe('3')
    await page.get('form button[type="button"]').trigger('click')

    expect(page.find('[data-testid="decision-edit-form"]').exists()).toBe(false)
    expect(page.find('[data-testid="decision-edit"]').exists()).toBe(true)
  })

  it('shows the reason beside the answer and saves nothing when the answer is unchanged', async () => {
    const page = mount(DecisionRowActions, { props: { decision: answered('replicas', '3') } })
    await page.get('[data-testid="decision-edit"]').trigger('click')

    await page.get('[data-testid="decision-edit-form"]').trigger('submit')

    expect(page.get('[role="alert"]').text()).toContain('Change the answer or add a note')
    expect(api.editDecision).not.toHaveBeenCalled()
  })

  it('sends the new answer and the note to the store, then closes the form', async () => {
    const before = answered('replicas', '2')
    vi.mocked(api.editDecision).mockResolvedValue(answered('replicas', '3', { previousAnswer: '2', source: 'PERSON' }))
    const page = mount(DecisionRowActions, { props: { decision: before } })
    await page.get('[data-testid="decision-edit"]').trigger('click')

    await page.get('[data-testid="decision-edit-answer"]').setValue('3')
    await page.get('[data-testid="decision-edit-note"]').setValue('Autoscale to 6.')
    await page.get('[data-testid="decision-edit-form"]').trigger('submit')
    await settle()

    expect(api.editDecision).toHaveBeenCalledWith(TASK, before.id, { answer: '3', note: 'Autoscale to 6.' })
    expect(page.find('[data-testid="decision-edit-form"]').exists()).toBe(false)
  })

  it('keeps the form open and shows the server text when the save is refused', async () => {
    vi.mocked(api.editDecision).mockRejectedValue(new Error('A decision is capped at 4000 characters.'))
    const page = mount(DecisionRowActions, { props: { decision: decision('region') } })
    await page.get('[data-testid="decision-edit"]').trigger('click')

    await page.get('[data-testid="decision-edit-answer"]').setValue('eu-west-1')
    await page.get('[data-testid="decision-edit-form"]').trigger('submit')
    await settle()

    expect(page.get('[role="alert"]').text()).toBe('A decision is capped at 4000 characters.')
    expect(page.find('[data-testid="decision-edit-form"]').exists()).toBe(true)
  })

  it('removes the decision only after the confirm names it', async () => {
    const doomed = decision('region')
    vi.mocked(api.removeDecision).mockResolvedValue(undefined)
    const page = mount(DecisionRowActions, { props: { decision: doomed }, attachTo: document.body })

    await page.get('[data-testid="decision-remove"]').trigger('click')
    expect(api.removeDecision).not.toHaveBeenCalled()
    expect(document.body.textContent).toContain('Remove region?')
    const confirm = [...document.querySelectorAll('button')].find((button) => button.textContent?.includes('Remove decision'))
    confirm!.click()
    await settle()

    expect(api.removeDecision).toHaveBeenCalledWith(doomed.id)
    page.unmount()
  })
})

describe('adding a decision by hand', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
    vi.mocked(api.fetchDecisions).mockResolvedValue({ decisions: [], sessionBlock: '' })
  })

  it('offers the step choice only when the task has steps', () => {
    const without = mount(DecisionAddForm, { props: { taskId: TASK, steps: [] } })
    const withSteps = mount(DecisionAddForm, { props: { taskId: TASK, steps: [{ number: 1, title: 'Draw' }, { number: 2, title: 'Wire' }] } })

    expect(without.find('[data-testid="decision-add-step"]').exists()).toBe(false)
    expect(withSteps.findAll('[data-testid="decision-add-step"] option').map((option) => option.text())).toEqual(['The task as a whole', 'Step 1 · Draw', 'Step 2 · Wire'])
  })

  it('asks for the question and the answer before it calls the store', async () => {
    const page = mount(DecisionAddForm, { props: { taskId: TASK, steps: [] } })

    await page.get('[data-testid="decision-add-form"]').trigger('submit')

    expect(page.get('[role="alert"]').text()).toContain('needs its question')
    expect(api.addDecision).not.toHaveBeenCalled()
  })

  it('adds the decision with the step as a number and tells its parent', async () => {
    vi.mocked(api.addDecision).mockResolvedValue(answered('which-namespace', 'billing', { source: 'PERSON' }))
    const page = mount(DecisionAddForm, { props: { taskId: TASK, steps: [{ number: 1, title: 'Draw' }] } })

    await page.get('[data-testid="decision-add-topic"]').setValue('Cluster')
    await page.get('[data-testid="decision-add-question"]').setValue('Which namespace?')
    await page.get('[data-testid="decision-add-answer"]').setValue('billing')
    await page.get('[data-testid="decision-add-step"] select').setValue('1')
    await page.get('[data-testid="decision-add-form"]').trigger('submit')
    await settle()

    expect(api.addDecision).toHaveBeenCalledWith(TASK, { topic: 'Cluster', question: 'Which namespace?', answer: 'billing', stepNumber: 1 })
    expect(page.emitted('added')).toHaveLength(1)
    expect(useConsoleStore().decisionsByTask[TASK]).toBeDefined()
  })
})
