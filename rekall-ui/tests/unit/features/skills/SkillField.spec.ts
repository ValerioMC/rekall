import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import SkillField from '@/features/skills/components/SkillField.vue'
import { useClaudeResourcesStore } from '@/features/claude-resources/store/claudeResources.store'
import type { EffectiveSkill, SkillAssignment } from '@/features/skills/model/skill-assignment'
import { catalog, summary } from '../claude-resources/claude-resources.fixtures'

const skill = (name: string) =>
  summary(`skills/${name}/SKILL.md`, {
    kind: 'skill',
    name,
    description: null,
    source: { kind: 'own', name: 'You', detail: null }
  })

function mountField(modelValue: SkillAssignment[], inherited: EffectiveSkill[] = []) {
  return mount(SkillField, { props: { modelValue, inherited, owner: 'step' }, attachTo: document.body })
}

const argumentsField = (name: string): HTMLInputElement | undefined =>
  Array.from(document.body.querySelectorAll<HTMLInputElement>('[data-testid="skill-arguments"]')).find(
    (field) => field.dataset.argumentsFor === name
  )

beforeEach(() => {
  document.body.innerHTML = ''
  setActivePinia(createPinia())
  const resources = useClaudeResourcesStore()
  resources.catalog = catalog({ skills: [skill('impeccable'), skill('rk')] })
  resources.loaded = true
})

describe('SkillField', () => {
  it('shows each own skill as name and arguments', () => {
    const wrapper = mountField([
      { name: 'impeccable', arguments: 'redesign website ui' },
      { name: 'rk', arguments: null }
    ])

    const chips = wrapper.findAll('[data-testid="skill-chip"]')

    expect(chips[0]!.text()).toContain('impeccable')
    expect(chips[0]!.text()).toContain('redesign website ui')
    expect(chips[1]!.text()).toBe('rk')
  })

  it('shows what an inherited skill brings, dashed, with its arguments and no way to edit it', () => {
    const wrapper = mountField(
      [],
      [
        { name: 'impeccable', arguments: 'redesign', origin: 'task' },
        { name: 'rk', arguments: null, origin: 'tag' }
      ]
    )

    const chip = wrapper.get('[data-origin="task"]')

    expect(chip.get('[data-testid="skill-chip-arguments"]').text()).toBe('redesign')
    expect(wrapper.find('[data-testid="skill-chip-edit"]').exists()).toBe(false)
  })

  it('opens the catalogue on the arguments of the skill whose chip was clicked', async () => {
    const wrapper = mountField([
      { name: 'impeccable', arguments: 'redesign' },
      { name: 'rk', arguments: null }
    ])

    await wrapper.findAll('[data-testid="skill-chip-edit"]')[1]!.trigger('click')
    await flushPromises()

    expect(document.activeElement).toBe(argumentsField('rk'))
  })

  it('sends the whole list with the new arguments when the typing ends', async () => {
    const wrapper = mountField([{ name: 'impeccable', arguments: 'redesign' }])
    await wrapper.get('[data-testid="skill-chip-edit"]').trigger('click')
    await flushPromises()

    const field = argumentsField('impeccable')!
    field.value = 'polish'
    field.dispatchEvent(new Event('input'))
    field.dispatchEvent(new Event('change'))

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([[{ name: 'impeccable', arguments: 'polish' }]])
  })

  it('removes a skill with its arguments', async () => {
    const wrapper = mountField([
      { name: 'impeccable', arguments: 'redesign' },
      { name: 'rk', arguments: null }
    ])

    await wrapper.findAll('[data-testid="skill-chip-remove"]')[0]!.trigger('click')

    expect(wrapper.emitted('update:modelValue')?.[0]).toEqual([[{ name: 'rk', arguments: null }]])
  })
})
