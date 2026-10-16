import { describe, expect, it } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import FlowRouteEditor from '@/features/flow/components/FlowRouteEditor.vue'
import type { TaskStepId } from '@/model/branded'
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

  it('offers the end of the flow among the targets', () => {
    const options = render().get('[data-testid="flow-route-add"] select').findAll('option')

    expect(options.map((option) => option.attributes('value'))).toContain('flow:end')
  })

  it('points a route at the end of the flow when End is picked', async () => {
    const editor = render()

    await editor.get('[data-testid="flow-route-row"] select').setValue('flow:end')

    expect(editor.emitted('edit')).toEqual([['test-app-error', { endsFlow: true }]])
  })

  it('fills the target and focuses the outcome when a route was drawn on the canvas', async () => {
    const steps = releaseFlow()
    const editor = mount(FlowRouteEditor, {
      attachTo: document.body,
      props: { step: steps[2]!, steps, busy: false, drawn: null }
    })

    await editor.setProps({ drawn: { to: 'fix' as TaskStepId, nonce: 1 } })
    await flushPromises()

    expect((editor.get('[data-testid="flow-route-add"] select').element as HTMLSelectElement).value).toBe(
      'fix'
    )
    expect(document.activeElement).toBe(editor.get('[data-testid="flow-route-add"] input').element)
    editor.unmount()
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

  it('removes only the route whose cross is pressed', async () => {
    const editor = render()

    await editor.findAll('[data-testid="flow-route-row"] button')[1]!.trigger('click')

    expect(editor.emitted('remove')).toEqual([['test-passed']])
  })

  it('keeps the remove cross from growing a hit area over the panel', () => {
    // `hit-target` paints an absolutely positioned ::after; on an unpositioned button it
    // covers the whole inspector, so any click removed the first route.
    const crosses = render().findAll('[data-testid="flow-route-row"] button')

    for (const cross of crosses) {
      expect(cross.classes()).not.toContain('hit-target')
    }
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
