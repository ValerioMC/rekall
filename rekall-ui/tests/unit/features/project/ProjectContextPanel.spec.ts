import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ProjectContextPanel from '@/features/project/components/ProjectContextPanel.vue'
import type { ProjectSection } from '@/features/project/model/project-section'

describe('ProjectContextPanel', () => {
  it('shows the total, each part, and the text kept out of context', () => {
    const wrapper = mount(ProjectContextPanel, {
      props: {
        input: {
          blueprint: 'b'.repeat(8000),
          components: [],
          connections: [],
          sections: [{ title: 'Reference', bodyMarkdown: 'x'.repeat(62_410) } as ProjectSection]
        }
      }
    })

    expect(wrapper.find('[data-testid="context-total"]').text()).toContain('2.0k')
    expect(wrapper.find('[data-testid="context-part-blueprint"]').text()).toContain('2,000')
    expect(wrapper.find('[data-testid="context-held-back"]').text()).toContain('62,410')
  })
})
