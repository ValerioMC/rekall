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
})
