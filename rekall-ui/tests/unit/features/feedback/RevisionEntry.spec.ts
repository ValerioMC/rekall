import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import RevisionEntry from '@/features/feedback/components/RevisionEntry.vue'
import { applied, revision } from './fixtures'

function view(before: string, after: string, state: 'PENDING' | 'APPLIED' = 'PENDING') {
  const entry = revision(1, before, after)
  return mount(RevisionEntry, { props: { entry: state === 'PENDING' ? entry : { ...entry, ...applied(1, 'Built.'), quotedMarkdown: before, bodyMarkdown: after } } })
}

describe('a revision of the description', () => {
  it('shows what was removed and what was added', () => {
    const wrapper = view('Export to CSV.', 'Export to CSV and XLSX.')

    expect(wrapper.get('[data-testid="revision-removed"]').text()).toContain('Export to CSV.')
    expect(wrapper.get('[data-testid="revision-added"]').text()).toContain('Export to CSV and XLSX.')
  })

  it('folds a long unchanged run and opens it on request', async () => {
    const kept = Array.from({ length: 9 }, (_, index) => `kept ${index}`).join('\n')
    const wrapper = view(`${kept}\nold`, `${kept}\nnew`)

    const fold = wrapper.get('[data-testid="revision-fold"]')
    expect(fold.text()).toContain('8 unchanged lines')
    expect(wrapper.text()).not.toContain('kept 3')

    await fold.trigger('click')

    expect(wrapper.find('[data-testid="revision-fold"]').exists()).toBe(false)
    expect(wrapper.text()).toContain('kept 3')
  })

  it('offers to discard a pending revision, and the entry answers with it', async () => {
    const wrapper = view('a', 'b')

    await wrapper.get('[data-testid="revision-discard"]').trigger('click')

    expect(wrapper.emitted('discard')).toHaveLength(1)
  })

  it('does not offer to discard one a session has applied', () => {
    expect(view('a', 'b', 'APPLIED').find('[data-testid="revision-discard"]').exists()).toBe(false)
  })
})
