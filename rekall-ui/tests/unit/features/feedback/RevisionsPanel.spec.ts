import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import RevisionsPanel from '@/features/feedback/components/RevisionsPanel.vue'
import type { Feedback } from '@/features/feedback/model/feedback'
import { applied, appliedInPass, entry, running } from './fixtures'

function panel(entries: Feedback[]) {
  return mount(RevisionsPanel, { props: { entries }, global: { stubs: { AppMarkdownEditor: true } } })
}

describe('the revisions of the description', () => {
  it('says how revisions come about when there are none', () => {
    const wrapper = panel([])

    expect(wrapper.get('[data-testid="revisions-empty"]').text()).toContain('every change to its description')
    expect(wrapper.find('[data-testid="revisions-waiting"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="feedback-pass"]').exists()).toBe(false)
  })

  it('lists what waits first, and passes a discard on', async () => {
    const wrapper = panel([appliedInPass(1, 1, 'Done.'), entry(2)])

    const waiting = wrapper.get('[data-testid="revisions-waiting"]')
    expect(waiting.findAll('[data-testid="revision-card"]').map((card) => card.attributes('data-state'))).toEqual(['PENDING'])

    await waiting.get('[data-testid="revision-discard"]').trigger('click')

    expect(wrapper.emitted('discard')).toEqual([[entry(2)]])
  })

  it('shows each pass as its own group, newest first, the running one marked and open', () => {
    const wrapper = panel([appliedInPass(1, 1, 'Done.'), running(2, 2), running(3, 2)])

    const passes = wrapper.findAll('[data-testid="feedback-pass"]')
    expect(passes.map((pass) => pass.get('[data-testid="feedback-pass-title"]').text())).toEqual(['Pass 2', 'Pass 1'])
    expect(passes[0]?.attributes('data-running')).toBe('true')
    expect(passes[0]?.get('[data-testid="feedback-pass-status"]').text()).toBe('0 of 2 answered')
    expect(passes[0]?.find('[data-testid="feedback-pass-live"]').exists()).toBe(true)
    expect(passes[0]?.findAll('[data-testid="revision-card"]')).toHaveLength(2)
    expect(passes[1]?.get('[data-testid="feedback-pass-status"]').text()).toBe('Done')
  })

  it('opens the newest done pass and folds an older one to a line per revision with its reply', async () => {
    const wrapper = panel([appliedInPass(1, 1, 'Fixed the totals.\nMore.'), appliedInPass(2, 2, 'Renamed it.')])

    const [newest, older] = wrapper.findAll('[data-testid="feedback-pass"]')
    expect(newest?.findAll('[data-testid="revision-card"]')).toHaveLength(1)
    expect(older?.findAll('[data-testid="revision-card"]')).toHaveLength(0)
    expect(older?.get('[data-testid="feedback-pass-summary"]').text()).toContain('Fixed the totals.')

    await older?.get('[data-testid="feedback-pass-toggle"]').trigger('click')

    expect(older?.findAll('[data-testid="revision-card"]')).toHaveLength(1)
  })

  it('keeps what was applied before passes in a folded group of its own, last', () => {
    const wrapper = panel([applied(1, 'Old answer.'), appliedInPass(2, 1, 'Done.')])

    const titles = wrapper.findAll('[data-testid="feedback-pass-title"]').map((title) => title.text())
    expect(titles).toEqual(['Pass 1', 'Before passes'])
    expect(wrapper.findAll('[data-testid="feedback-pass"]')[1]?.find('[data-testid="feedback-pass-summary"]').exists()).toBe(true)
  })

  it('reads newest first: what waits, the passes, then what came before passes', () => {
    const wrapper = panel([applied(1, 'Old.'), appliedInPass(2, 1, 'Done.'), entry(3)])

    const sections = wrapper.findAll('[data-testid="revisions-panel"] > section').map((section) => section.attributes('data-testid') === 'revisions-waiting' ? 'Waiting' : section.get('[data-testid="feedback-pass-title"]').text())

    expect(sections).toEqual(['Waiting', 'Pass 1', 'Before passes'])
  })

  it('offers no discard on a revision in a running pass', () => {
    expect(panel([running(1, 1)]).find('[data-testid="revision-discard"]').exists()).toBe(false)
  })
})
