import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import CommitDiffView from '@/components/console/CommitDiffView.vue'

describe('CommitDiffView', () => {
  it('says nothing is recorded when there is no diff', () => {
    const wrapper = mount(CommitDiffView, { props: { diff: null } })

    expect(wrapper.get('[data-testid="commit-diff-empty"]').text()).toContain('No diff recorded')
    expect(wrapper.find('[data-testid="commit-diff-view"]').exists()).toBe(false)
  })

  it('colors added and removed lines apart from context and hunk headers', () => {
    const diff = [
      'diff --git a/a.txt b/a.txt',
      '@@ -1,2 +1,2 @@',
      '+new line',
      '-old line',
      ' unchanged line'
    ].join('\n')
    const wrapper = mount(CommitDiffView, { props: { diff } })

    const lines = wrapper.findAll('code')
    expect(lines[0]!.classes()).toContain('text-text-subtle')
    expect(lines[1]!.classes()).toContain('text-accent')
    expect(lines[2]!.classes()).toContain('text-safe')
    expect(lines[3]!.classes()).toContain('text-danger')
    expect(lines[4]!.classes()).toContain('text-text-muted')
  })
})
