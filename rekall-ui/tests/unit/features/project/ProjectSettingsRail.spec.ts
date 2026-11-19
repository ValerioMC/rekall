import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectSettingsRail from '@/features/project/components/ProjectSettingsRail.vue'

describe('the settings rail', () => {
  it('marks the open setting and asks for another when one is picked', async () => {
    const rail = mount(ProjectSettingsRail, { props: { selected: 'general' } })

    expect(rail.get('[data-testid="settings-rail-general"]').attributes('aria-current')).toBe('page')
    expect(rail.get('[data-testid="settings-rail-checks"]').attributes('aria-current')).toBeUndefined()
    await rail.get('[data-testid="settings-rail-checks"]').trigger('click')

    expect(rail.emitted('select')).toEqual([['checks']])
  })
})
