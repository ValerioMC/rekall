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
  it('fills the pane, so a step detail takes the width the list leaves it', () => {
    const body = frame().get('[data-testid="detail"]').element.parentElement!.parentElement!

    expect(body.className).toContain('w-full')
    expect(body.className).not.toContain('max-w')
  })

  it('holds a bounded frame to a page width, centred', () => {
    const body = frame({ bounded: true }).get('[data-testid="detail"]').element.parentElement!.parentElement!

    expect(body.className).toContain('max-w-[76rem]')
    expect(body.className).toContain('mx-auto')
  })
})
