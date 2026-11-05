import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import WindowControls from '@/components/console/WindowControls.vue'
import type { DesktopPlatform } from '@/common/native/desktop'

function mountOn(platform: DesktopPlatform | null, host: Partial<Window['rekallDesktop']> = {}) {
  if (platform) window.rekallDesktop = { pickFolder: vi.fn(), platform, ...host }
  return mount(WindowControls)
}

const labels = (wrapper: ReturnType<typeof mountOn>) => wrapper.findAll('button').map((b) => b.attributes('aria-label'))

describe('WindowControls', () => {
  afterEach(() => {
    delete window.rekallDesktop
  })

  it('draws the macOS order, close first, in the macOS build', () => {
    expect(labels(mountOn('macos'))).toEqual(['Close', 'Minimize', 'Maximize'])
  })

  it('draws the Windows order, close last, in the Windows build', () => {
    expect(labels(mountOn('windows'))).toEqual(['Minimize', 'Maximize', 'Close'])
  })

  it('draws nothing where the window keeps its native frame', () => {
    expect(mountOn('linux').find('[data-testid="window-controls"]').exists()).toBe(false)
    delete window.rekallDesktop
    expect(mountOn(null).find('[data-testid="window-controls"]').exists()).toBe(false)
  })

  it('asks the host to close, minimize and maximize', async () => {
    const closeWindow = vi.fn().mockResolvedValue(undefined)
    const minimizeWindow = vi.fn().mockResolvedValue(undefined)
    const toggleMaximizeWindow = vi.fn().mockResolvedValue(undefined)
    const wrapper = mountOn('windows', { closeWindow, minimizeWindow, toggleMaximizeWindow })
    for (const button of wrapper.findAll('button')) await button.trigger('click')
    expect(minimizeWindow).toHaveBeenCalledOnce()
    expect(toggleMaximizeWindow).toHaveBeenCalledOnce()
    expect(closeWindow).toHaveBeenCalledOnce()
  })
})
