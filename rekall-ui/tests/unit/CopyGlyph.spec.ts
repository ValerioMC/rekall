import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'

describe('CopyGlyph', () => {
  it('draws the copy sheets until told the click happened, then a check', async () => {
    const wrapper = mount(CopyGlyph, { props: { copied: false } })
    const glyph = wrapper.get('[data-testid="copy-glyph"]')

    expect(glyph.attributes('data-copied')).toBe('false')
    expect(glyph.find('rect').exists()).toBe(true)
    expect(glyph.text()).toBe('')

    await wrapper.setProps({ copied: true })

    expect(glyph.attributes('data-copied')).toBe('true')
    expect(glyph.find('rect').exists()).toBe(false)
    expect(glyph.find('path').exists()).toBe(true)
  })
})
