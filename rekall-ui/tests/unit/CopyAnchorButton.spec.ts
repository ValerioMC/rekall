import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'

describe('the copy anchor button', () => {
  const writeText = vi.fn<(text: string) => Promise<void>>()

  beforeEach(() => {
    vi.useFakeTimers()
    writeText.mockReset().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('names the command it copies and prints none of it', () => {
    const wrapper = mount(CopyAnchorButton, { props: { anchor: 'project:vega task:report' } })

    expect(wrapper.attributes('aria-label')).toBe('Copy /rk project:vega task:report')
    expect(wrapper.attributes('title')).toBe('Copy /rk project:vega task:report')
    expect(wrapper.text()).toBe('')
  })

  it('copies the /rk command, shows the check for a moment, then goes back', async () => {
    const wrapper = mount(CopyAnchorButton, { props: { anchor: 'project:vega task:report' } })

    await wrapper.trigger('click')

    expect(writeText).toHaveBeenCalledWith('/rk project:vega task:report')
    expect(wrapper.attributes('title')).toBe('Copied')
    expect(wrapper.get('[data-testid="copy-glyph"]').attributes('data-copied')).toBe('true')

    await vi.advanceTimersByTimeAsync(1500)

    expect(wrapper.get('[data-testid="copy-glyph"]').attributes('data-copied')).toBe('false')
  })
})
