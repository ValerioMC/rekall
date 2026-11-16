import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DossierFrame from '@/features/steps/components/DossierFrame.vue'

function frame(props: Record<string, unknown> = {}) {
  return mount(DossierFrame, {
    props: { pinned: true, ...props },
    slots: { title: 'A step', main: '<p data-testid="detail">Detail</p>' }
  })
}

describe('the dossier frame', () => {
  const bodyOf = (wrapper: ReturnType<typeof frame>) => wrapper.get('[data-testid="detail"]').element.parentElement!.parentElement!

  it('fills the pane, so a step detail takes the width the list leaves it', () => {
    expect(bodyOf(frame()).getAttribute('style')).toBeNull()
  })

  it('holds the header, body and dock to the width it is given, centred', () => {
    const held = frame({ maxWidth: '1040px' })

    expect(bodyOf(held).getAttribute('style')).toContain('max-width: 1040px')
    expect(bodyOf(held).className).toContain('mx-auto')
    expect(held.get('header > div').attributes('style')).toContain('max-width: 1040px')
  })
})
