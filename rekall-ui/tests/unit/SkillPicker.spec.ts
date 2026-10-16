import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import AppDialog from '@/components/ui/AppDialog.vue'
import SkillPicker from '@/features/skills/components/SkillPicker.vue'
import { useClaudeResourcesStore } from '@/features/claude-resources/store/claudeResources.store'
import { catalog, summary } from './claude-resources.fixtures'

/** The catalogue panel: skills from the store, ticked into the list the owner holds. It is teleported, so the DOM is read from the body. */
const skill = (name: string, description: string | null = null) =>
  summary(`skills/${name}/SKILL.md`, { kind: 'skill', name, description, source: { kind: 'own', name: 'You', detail: null } })

function mountPicker(modelValue: string[], inherited: string[] = []) {
  const anchor = document.createElement('button')
  document.body.appendChild(anchor)
  return mount(SkillPicker, { props: { modelValue, anchor, inherited }, attachTo: document.body })
}

const rows = (): HTMLButtonElement[] =>
  Array.from(document.body.querySelectorAll<HTMLButtonElement>('[data-testid="skill-picker-row"]'))
const rowOf = (name: string): HTMLButtonElement => rows().find((row) => row.textContent?.includes(name))!

beforeEach(() => {
  document.body.innerHTML = ''
  setActivePinia(createPinia())
  const resources = useClaudeResourcesStore()
  resources.catalog = catalog({ skills: [skill('rk', 'Load a Rekall context'), skill('code-review')] })
  resources.loaded = true
})

describe('SkillPicker', () => {
  it('ticks a skill by sending the list with it added', async () => {
    const wrapper = mountPicker(['rk'])
    await flushPromises()

    rowOf('code-review').click()

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([['rk', 'code-review']])
  })

  it('unticks a skill the owner holds', async () => {
    const wrapper = mountPicker(['rk', 'code-review'])
    await flushPromises()

    rowOf('rk').click()

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([['code-review']])
  })

  it('leaves a skill from a wider scope ticked and disabled', async () => {
    mountPicker([], ['rk'])
    await flushPromises()

    const row = rowOf('rk')

    expect(row.disabled).toBe(true)
    expect(row.getAttribute('aria-selected')).toBe('true')
    expect(row.textContent).toContain('already on the task')
  })

  it('narrows the list by the words typed', async () => {
    mountPicker([])
    await flushPromises()

    const field = document.body.querySelector<HTMLInputElement>('[data-testid="skill-picker-filter"]')!
    field.value = 'rekall'
    field.dispatchEvent(new Event('input'))
    await flushPromises()

    expect(rows()).toHaveLength(1)
    expect(rows()[0]!.textContent).toContain('rk')
  })

  it('lists on top a skill the owner holds that the catalogue no longer has', async () => {
    mountPicker(['vanished'])
    await flushPromises()

    expect(document.body.querySelector('[data-testid="skill-picker-missing"]')?.textContent).toContain('vanished')
  })

  it('closes on Escape', async () => {
    const wrapper = mountPicker([])
    await flushPromises()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('hangs above a modal, whose backdrop sits on --z-modal', async () => {
    mountPicker([])
    await flushPromises()

    const panel = document.body.querySelector('[data-testid="skill-picker"]')!

    expect(panel.className).toContain('z-(--z-popover)')
  })

  it('takes Escape from the dialog it opened over', async () => {
    const dialog = mount(AppDialog, { props: { label: 'New task' }, attachTo: document.body })
    const picker = mountPicker([])
    await flushPromises()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(picker.emitted('close')).toHaveLength(1)
    expect(dialog.emitted('dismiss')).toBeUndefined()
    picker.unmount()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(dialog.emitted('dismiss')).toHaveLength(1)
  })
})
