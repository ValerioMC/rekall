import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SkillChip from '@/features/skills/components/SkillChip.vue'

describe('SkillChip', () => {
  it('can be removed when it was assigned on the owner itself', async () => {
    const wrapper = mount(SkillChip, { props: { name: 'rk', origin: 'task' } })

    await wrapper.get('[data-testid="skill-chip-remove"]').trigger('click')

    expect(wrapper.emitted('remove')).toHaveLength(1)
    expect(wrapper.get('[data-testid="skill-chip-remove"]').attributes('aria-label')).toBe('Remove skill rk')
  })

  it('cannot be removed where it only comes from the tag', () => {
    const wrapper = mount(SkillChip, { props: { name: 'rk', origin: 'tag' } })

    expect(wrapper.find('[data-testid="skill-chip-remove"]').exists()).toBe(false)
    expect(wrapper.attributes('data-origin')).toBe('tag')
  })

  it('says in its tooltip when the skill is gone from ~/.claude', () => {
    const wrapper = mount(SkillChip, { props: { name: 'old', missing: true } })

    expect(wrapper.attributes('title')).toContain('is not in ~/.claude')
  })

  it('draws the arguments after the name and names them in its tooltip', () => {
    const wrapper = mount(SkillChip, {
      props: { name: 'impeccable', arguments: 'redesign website ui', origin: 'task' }
    })

    expect(wrapper.get('[data-testid="skill-chip-arguments"]').text()).toBe('redesign website ui')
    expect(wrapper.attributes('title')).toBe('impeccable redesign website ui, assigned to this task')
  })

  it('draws a skill without arguments as the name alone', () => {
    const wrapper = mount(SkillChip, { props: { name: 'rk', origin: 'task' } })

    expect(wrapper.find('[data-testid="skill-chip-arguments"]').exists()).toBe(false)
  })

  it('asks to edit when its name is clicked, if it is editable', async () => {
    const wrapper = mount(SkillChip, { props: { name: 'rk', origin: 'task', editable: true } })

    await wrapper.get('[data-testid="skill-chip-edit"]').trigger('click')

    expect(wrapper.emitted('edit')).toHaveLength(1)
    expect(wrapper.get('[data-testid="skill-chip-edit"]').attributes('aria-label')).toBe(
      'Edit arguments of skill rk'
    )
  })

  it('has nothing to click when it is not editable', () => {
    const wrapper = mount(SkillChip, { props: { name: 'rk', origin: 'tag' } })

    expect(wrapper.find('[data-testid="skill-chip-edit"]').exists()).toBe(false)
  })
})
