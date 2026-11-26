import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { adjacentMenuPlacement, useAnchoredMenu } from '@/composables/useAnchoredMenu'

const VIEWPORT_HEIGHT = 800
const KEY_HEIGHT = 28
const MENU_HEIGHT = 220

async function placementUnderKeyAt(top: number): Promise<Record<string, string>> {
  const key = document.createElement('button')
  key.getBoundingClientRect = () => new DOMRect(600, top, 100, KEY_HEIGHT)
  const popup = document.createElement('div')
  Object.defineProperty(popup, 'offsetHeight', { value: MENU_HEIGHT })
  let anchored!: ReturnType<typeof useAnchoredMenu>
  const host = mount(
    defineComponent({
      setup() {
        anchored = useAnchoredMenu(() => key, 200)
        anchored.menu.value = popup
        return () => null
      }
    })
  )
  await anchored.toggle()
  const placement = anchored.placement.value
  host.unmount()
  return placement
}

describe('useAnchoredMenu', () => {
  beforeEach(() => vi.stubGlobal('innerHeight', VIEWPORT_HEIGHT))

  afterEach(() => vi.unstubAllGlobals())

  it('hangs the menu under its key when there is room', async () => {
    const placement = await placementUnderKeyAt(40)

    expect(placement.top).toBe('72px')
    expect(placement.bottom).toBeUndefined()
  })

  it('opens the menu above a key too low to hold it underneath', async () => {
    const placement = await placementUnderKeyAt(750)

    expect(placement.bottom).toBe('54px')
    expect(placement.top).toBeUndefined()
  })

  it('uses the roomier side when the menu must scroll', async () => {
    vi.stubGlobal('innerHeight', 200)

    const placement = await placementUnderKeyAt(100)

    expect(placement.bottom).toBe('104px')
    expect(placement.maxHeight).toBe('88px')
  })
})

describe('adjacentMenuPlacement', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('flips to the left and clamps the provider rows to the viewport', () => {
    vi.stubGlobal('innerWidth', 800)
    vi.stubGlobal('innerHeight', 500)
    const parent = new DOMRect(560, 390, 200, 90)
    const row = new DOMRect(564, 450, 192, 30)

    expect(adjacentMenuPlacement(parent, row, 120, 240)).toMatchObject({ left: '316px', top: '372px' })
  })

  it('returns inline placement when neither side fits', () => {
    vi.stubGlobal('innerWidth', 480)
    const parent = new DOMRect(140, 250, 200, 160)

    expect(adjacentMenuPlacement(parent, parent, 120, 240)).toBeNull()
  })
})
