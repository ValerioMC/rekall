import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import SkillMark from '@/components/skills/SkillMark.vue'

describe('SkillMark', () => {
  it('renders nothing for no skills', () => {
    expect(mount(SkillMark, { props: { skills: [] } }).find('[data-testid="skill-mark"]').exists()).toBe(false)
  })

  it('shows the count and says each skill and its origin in its accessible name and tooltip', () => {
    const wrapper = mount(SkillMark, {
      props: { skills: [{ name: 'rust-standards', origin: 'task' }, { name: 'code-review', origin: 'tag' }] }
    })
    const mark = wrapper.get('[data-testid="skill-mark"]')

    expect(mark.text()).toBe('2')
    expect(mark.attributes('aria-label')).toBe('2 skills: rust-standards, code-review')
    expect(mark.attributes('title')).toBe('rust-standards, from this task\ncode-review, from its tag')
  })

  it('uses the singular for one skill', () => {
    const wrapper = mount(SkillMark, { props: { skills: [{ name: 'rk', origin: 'step' }] } })

    expect(wrapper.get('[data-testid="skill-mark"]').attributes('aria-label')).toBe('1 skill: rk')
  })
})
