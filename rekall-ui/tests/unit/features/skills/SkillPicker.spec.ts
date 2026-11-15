import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import AppDialog from '@/components/ui/AppDialog.vue'
import SkillPicker from '@/features/skills/components/SkillPicker.vue'
import { useClaudeResourcesStore } from '@/features/claude-resources/store/claudeResources.store'
import type { SkillAssignment } from '@/features/skills/model/skill-assignment'
import { catalog, summary } from '../claude-resources/claude-resources.fixtures'

/** The catalogue panel: skills from the store, ticked into the list the owner holds. It is teleported, so the DOM is read from the body. */
const skill = (name: string, description: string | null = null) =>
  summary(`skills/${name}/SKILL.md`, {
    kind: 'skill',
    name,
    description,
    source: { kind: 'own', name: 'You', detail: null }
  })

const held = (...names: string[]): SkillAssignment[] => names.map((name) => ({ name, arguments: null }))

function mountPicker(
  modelValue: SkillAssignment[],
  inherited: string[] = [],
  focusName: string | null = null
) {
  const anchor = document.createElement('button')
  document.body.appendChild(anchor)
  return mount(SkillPicker, { props: { modelValue, anchor, inherited, focusName }, attachTo: document.body })
}

const argumentsField = (name: string): HTMLInputElement | undefined =>
  Array.from(document.body.querySelectorAll<HTMLInputElement>('[data-testid="skill-arguments"]')).find(
    (field) => field.dataset.argumentsFor === name
  )

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
    const wrapper = mountPicker(held('rk'))
    await flushPromises()

    rowOf('code-review').click()

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([held('rk', 'code-review')])
  })

  it('unticks a skill the owner holds', async () => {
    const wrapper = mountPicker(held('rk', 'code-review'))
    await flushPromises()

    rowOf('rk').click()

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([held('code-review')])
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
    mountPicker(held('vanished'))
    await flushPromises()

    expect(document.body.querySelector('[data-testid="skill-picker-missing"]')?.textContent).toContain(
      'vanished'
    )
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

  it('opens a field for the arguments under a ticked skill and none under the others', async () => {
    mountPicker(held('rk'))
    await flushPromises()

    expect(argumentsField('rk')).toBeDefined()
    expect(argumentsField('code-review')).toBeUndefined()
  })

  it('sends the arguments typed, trimmed, when the field is left', async () => {
    const wrapper = mountPicker(held('rk', 'code-review'))
    await flushPromises()

    const field = argumentsField('rk')!
    field.value = '  redesign website ui '
    field.dispatchEvent(new Event('input'))
    field.dispatchEvent(new Event('change'))

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([
      [
        { name: 'rk', arguments: 'redesign website ui' },
        { name: 'code-review', arguments: null }
      ]
    ])
  })

  it('reads a cleared field as no arguments', async () => {
    const wrapper = mountPicker([{ name: 'rk', arguments: 'redesign' }])
    await flushPromises()

    const field = argumentsField('rk')!
    expect(field.value).toBe('redesign')
    field.value = '   '
    field.dispatchEvent(new Event('input'))
    field.dispatchEvent(new Event('change'))

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([[{ name: 'rk', arguments: null }]])
  })

  it('sends nothing for a field left as it was', async () => {
    const wrapper = mountPicker([{ name: 'rk', arguments: 'redesign' }])
    await flushPromises()

    argumentsField('rk')!.dispatchEvent(new Event('change'))

    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
  })

  it('keeps the text typed when the panel closes before the field is left', async () => {
    const wrapper = mountPicker(held('rk'))
    await flushPromises()

    const field = argumentsField('rk')!
    field.value = 'polish'
    field.dispatchEvent(new Event('input'))
    wrapper.unmount()

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([[{ name: 'rk', arguments: 'polish' }]])
  })

  it('caps the text at what the server accepts', async () => {
    mountPicker(held('rk'))
    await flushPromises()

    expect(argumentsField('rk')!.maxLength).toBe(300)
  })

  it('puts the keyboard on the arguments of the skill it was opened to edit', async () => {
    mountPicker(held('rk', 'code-review'), [], 'code-review')
    await flushPromises()

    expect(document.activeElement).toBe(argumentsField('code-review'))
  })

  it('does not tick the highlighted skill when Enter ends the typing of arguments', async () => {
    const wrapper = mountPicker(held('rk'))
    await flushPromises()
    const field = argumentsField('rk')!
    field.focus()

    field.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))

    expect(wrapper.emitted('update:modelValue')).toBeUndefined()
    expect(document.activeElement).not.toBe(field)
  })

  it('lets a vanished skill carry arguments too', async () => {
    const wrapper = mountPicker([{ name: 'vanished', arguments: 'old' }])
    await flushPromises()

    const field = argumentsField('vanished')!
    field.value = 'new'
    field.dispatchEvent(new Event('input'))
    field.dispatchEvent(new Event('change'))

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([[{ name: 'vanished', arguments: 'new' }]])
  })

  it('leaves a skill the owner holds itself tickable when a wider scope brings the same name', async () => {
    const wrapper = mountPicker([{ name: 'rk', arguments: 'polish' }], ['rk'])
    await flushPromises()

    const row = rowOf('rk')
    expect(row.disabled).toBe(false)
    expect(row.textContent).not.toContain('already on the task')

    row.click()

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([[]])
  })
})
