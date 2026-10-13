import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FlowRouteEditor from '@/components/console/flow/FlowRouteEditor.vue'
import { releaseFlow } from './flow.fixtures'

/** The routes out of one step: renamed on blur, retargeted on pick, the step itself never offered. */
function render() {
  const steps = releaseFlow()
  return mount(FlowRouteEditor, { props: { step: steps[2]!, steps, busy: false } })
}

describe('FlowRouteEditor', () => {
  it('lists every route out of the step', () => {
    const rows = render().findAll('[data-testid="flow-route-row"]')

    expect(rows.map((row) => (row.get('input').element as HTMLInputElement).value)).toEqual([
      'app-error',
      'passed'
    ])
  })

  it('never offers the step as its own target', () => {
    const options = render().get('[data-testid="flow-route-add"] select').findAll('option')

    expect(options.map((option) => option.attributes('value'))).not.toContain('test')
  })

  it('renames an outcome when the field is left, and only when it changed', async () => {
    const editor = render()
    const field = editor.get('[data-testid="flow-route-row"] input')

    await field.trigger('blur')
    await field.setValue('App crashed')
    await field.trigger('blur')

    expect(editor.emitted('edit')).toEqual([['test-app-error', { outcome: 'App crashed' }]])
  })

  it('puts back a blanked outcome instead of sending it', async () => {
    const editor = render()
    const field = editor.get('[data-testid="flow-route-row"] input')

    await field.setValue('   ')
    await field.trigger('blur')

    expect(editor.emitted('edit')).toBeUndefined()
    expect((field.element as HTMLInputElement).value).toBe('app-error')
  })

  it('retargets a route as soon as another step is picked', async () => {
    const editor = render()

    await editor.get('[data-testid="flow-route-row"] select').setValue('build')

    expect(editor.emitted('edit')).toEqual([['test-app-error', { toStepId: 'build' }]])
  })

  it('keeps Add disabled until both an outcome and a target are given', async () => {
    const editor = render()
    const add = editor.get('[data-testid="flow-route-add"] button[type="submit"]')
    expect(add.attributes('disabled')).toBeDefined()

    await editor.get('[data-testid="flow-route-add"] input').setValue('rollback')
    expect(add.attributes('disabled')).toBeDefined()
    await editor.get('[data-testid="flow-route-add"] select').setValue('build')

    expect(add.attributes('disabled')).toBeUndefined()
  })
})
