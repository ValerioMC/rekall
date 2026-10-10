import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import GenerateDiagramDialog from '@/components/diagram/dialogs/GenerateDiagramDialog.vue'
import { ORION, REPORT, VEGA, project, task } from './diagram.fixtures'
import type { GenerationRequest } from '@/stores/diagram.store'

function dialog(initialTaskId: typeof REPORT | null = null) {
  return mount(GenerateDiagramDialog, {
    props: { projects: [project(VEGA, 'vega'), project(ORION, 'orion', null)], tasks: [task(REPORT, 'report-builder', VEGA)], initialProjectId: VEGA, initialTaskId, sending: false },
    attachTo: document.body
  })
}

describe('the generate dialog', () => {
  it('asks for the project alone by default, as a sequence, and shows the line it will type', async () => {
    const wrapper = dialog()
    const submit = (): Element => document.querySelector('[data-testid="generate-submit"]')!
    expect(submit().hasAttribute('disabled')).toBe(true)

    await wrapper.find('[data-testid="generate-request"]').setValue('How does checkout charge?')
    await wrapper.find('[data-testid="kind-state"]').trigger('click')

    expect(document.querySelector('[data-testid="generate-preview"]')?.textContent).toContain('/rk project:vega generate kinds:sequence,state "How does checkout charge?"')
    await wrapper.find('[data-testid="generate-submit"]').trigger('click')
    const [[asked]] = wrapper.emitted('generate') as [[GenerationRequest]]
    expect(asked).toEqual({ projectId: VEGA, taskId: null, request: 'How does checkout charge?', kinds: ['sequence', 'state'] })
    wrapper.unmount()
  })

  it('carries the task it was opened for', async () => {
    const wrapper = dialog(REPORT)
    await wrapper.find('[data-testid="generate-request"]').setValue('report')
    await wrapper.find('[data-testid="generate-submit"]').trigger('click')
    const [[asked]] = wrapper.emitted('generate') as [[GenerationRequest]]
    expect(asked.taskId).toBe(REPORT)
    wrapper.unmount()
  })

  it('refuses to generate without a type', async () => {
    const wrapper = dialog()
    await wrapper.find('[data-testid="generate-request"]').setValue('anything')
    await wrapper.find('[data-testid="kind-sequence"]').trigger('click')
    expect(wrapper.text()).toContain('Pick at least one type.')
    expect(wrapper.find('[data-testid="generate-submit"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })
})
