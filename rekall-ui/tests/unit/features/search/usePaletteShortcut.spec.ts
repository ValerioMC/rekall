import { beforeEach, describe, expect, it } from 'vitest'
import { defineComponent } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { usePaletteShortcut } from '@/features/search/composables/usePaletteShortcut'
import { usePaletteStore } from '@/features/search/store/palette.store'
import { useModalGate } from '@/composables/useModalGate'

describe('the palette shortcut', () => {
  const Host = defineComponent({
    setup() {
      usePaletteShortcut()
      return () => null
    }
  })

  beforeEach(() => {
    setActivePinia(createPinia())
  })

  function press(): void {
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true, cancelable: true }))
  }

  it('opens the palette on Cmd+K and closes it on the next press', () => {
    const wrapper = mount(Host)

    press()
    expect(usePaletteStore().isOpen).toBe(true)
    press()
    expect(usePaletteStore().isOpen).toBe(false)
    wrapper.unmount()
  })

  it('does not open over another dialog', () => {
    const wrapper = mount(Host)
    const gate = useModalGate()
    gate.open()

    press()

    expect(usePaletteStore().isOpen).toBe(false)
    gate.close()
    wrapper.unmount()
  })
})
