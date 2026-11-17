import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import ProjectReadingColumn from '@/features/project/components/ProjectReadingColumn.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

describe('ProjectReadingColumn', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('holds its content as wide as the chosen page width, centred', async () => {
    const preferences = usePreferencesStore()
    preferences.taskWidth = 'narrow'
    const wrapper = mount(ProjectReadingColumn, { slots: { default: '<p>text</p>' } })
    const column = wrapper.find('[data-testid="reading-column"]')

    expect(column.attributes('style')).toContain('max-width: 760px')
    expect(column.classes()).toContain('mx-auto')

    preferences.taskWidth = 'full'
    await wrapper.vm.$nextTick()

    expect(column.attributes('style') ?? '').toContain('max-width: none')
  })
})
