import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import FeedbackThread from '@/features/feedback/components/FeedbackThread.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { Feedback } from '@/features/feedback/model/feedback'
import { applied, appliedInPass, entry, running, TASK } from './fixtures'

const stubs = { AppMarkdownEditor: true }

function thread(entries: Feedback[]) {
  return mount(FeedbackThread, { props: { entries }, global: { stubs } })
}

describe('the thread under the description', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    useConsoleStore().selectedTaskId = TASK
  })

  it('invites the first message when nothing is waiting, with the composer under it', () => {
    const wrapper = thread([])

    expect(wrapper.find('[data-testid="feedback-empty"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="feedback-composer"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="feedback-pass"]').exists()).toBe(false)
  })

  it('lists what waits under Next run, above the composer, and passes their requests on', async () => {
    const wrapper = thread([applied(1, 'Done.'), entry(2)])

    const next = wrapper.get('[data-testid="feedback-next-run"]')
    expect(next.findAll('[data-testid="feedback-entry"]').map((item) => item.attributes('data-state'))).toEqual(['PENDING'])
    expect(wrapper.get('[data-testid="feedback-next-count"]').text()).toBe('1 waiting')
    expect(next.find('[data-testid="feedback-composer"]').exists()).toBe(true)

    await wrapper.get('[data-testid="feedback-delete"]').trigger('click')

    expect(wrapper.emitted('remove')).toEqual([[entry(2)]])
  })

  it('shows each pass as its own group, newest first, the running one marked and open', () => {
    const wrapper = thread([appliedInPass(1, 1, 'Done.'), running(2, 2), running(3, 2)])

    const passes = wrapper.findAll('[data-testid="feedback-pass"]')
    expect(passes.map((pass) => pass.get('[data-testid="feedback-pass-title"]').text())).toEqual(['Pass 2', 'Pass 1'])
    expect(passes[0]?.attributes('data-running')).toBe('true')
    expect(passes[0]?.get('[data-testid="feedback-pass-status"]').text()).toBe('0 of 2 answered')
    expect(passes[0]?.find('[data-testid="feedback-pass-live"]').exists()).toBe(true)
    expect(passes[0]?.findAll('[data-testid="feedback-entry"]')).toHaveLength(2)
    expect(passes[1]?.get('[data-testid="feedback-pass-status"]').text()).toBe('Done')
  })

  it('opens the newest done pass and folds an older one to a line per entry with its reply', async () => {
    const wrapper = thread([appliedInPass(1, 1, 'Fixed the totals.\nMore.'), appliedInPass(2, 2, 'Renamed it.')])

    const [newest, older] = wrapper.findAll('[data-testid="feedback-pass"]')
    expect(newest?.findAll('[data-testid="feedback-entry"]')).toHaveLength(1)
    expect(older?.findAll('[data-testid="feedback-entry"]')).toHaveLength(0)
    expect(older?.get('[data-testid="feedback-pass-summary"]').text()).toContain('Fixed the totals.')

    await older?.get('[data-testid="feedback-pass-toggle"]').trigger('click')

    expect(older?.findAll('[data-testid="feedback-entry"]')).toHaveLength(1)
  })

  it('keeps what was applied before passes in a folded group of its own', () => {
    const wrapper = thread([applied(1, 'Old answer.')])

    const group = wrapper.get('[data-testid="feedback-pass"]')
    expect(group.get('[data-testid="feedback-pass-title"]').text()).toBe('Before passes')
    expect(group.find('[data-testid="feedback-pass-summary"]').exists()).toBe(true)
  })

  it('aims the composer at an applied entry when Add detail is chosen on it', async () => {
    const wrapper = thread([appliedInPass(4, 1, 'Fixed.')])

    await wrapper.get('[data-testid="feedback-follow-up"]').trigger('click')

    expect(wrapper.get('[data-testid="feedback-following"]').text()).toContain('Adding detail to #4')
  })

  it('makes no edit or delete offer on an entry in a running pass', () => {
    const wrapper = thread([running(1, 1)])

    expect(wrapper.find('[data-testid="feedback-edit"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="feedback-delete"]').exists()).toBe(false)
  })
})
