import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import WindowsWindowControls from '@/components/console/WindowsWindowControls.vue'

describe('WindowsWindowControls', () => {
  it('draws Windows order, close last', () => {
    const labels = mount(WindowsWindowControls)
      .findAll('button')
      .map((button) => button.attributes('aria-label'))
    expect(labels).toEqual(['Minimize', 'Maximize', 'Close'])
  })

  it('asks the host to minimize, maximize and close', async () => {
    const minimizeWindow = vi.fn().mockResolvedValue(undefined)
    const toggleMaximizeWindow = vi.fn().mockResolvedValue(undefined)
    const closeWindow = vi.fn().mockResolvedValue(undefined)
    window.rekallDesktop = { pickFolder: vi.fn(), platform: 'windows', minimizeWindow, toggleMaximizeWindow, closeWindow }
    const wrapper = mount(WindowsWindowControls)
    for (const button of wrapper.findAll('button')) await button.trigger('click')
    delete window.rekallDesktop
    expect(minimizeWindow).toHaveBeenCalledOnce()
    expect(toggleMaximizeWindow).toHaveBeenCalledOnce()
    expect(closeWindow).toHaveBeenCalledOnce()
  })
})
