import { afterEach, describe, expect, it } from 'vitest'
import { shallowMount } from '@vue/test-utils'
import AppHeaderChrome from '@/components/shell/AppHeaderChrome.vue'

describe('the header drag region', () => {
  afterEach(() => {
    delete window.rekallDesktop
  })

  it('is deep in the desktop window, so a press on any empty part of the bar drags it', () => {
    window.rekallDesktop = { pickFolder: async () => null, platform: 'macos' }
    const wrapper = shallowMount(AppHeaderChrome)
    expect(wrapper.get('header').attributes('data-tauri-drag-region')).toBe('deep')
  })

  it('is absent in a browser tab', () => {
    const wrapper = shallowMount(AppHeaderChrome)
    expect(wrapper.get('header').attributes('data-tauri-drag-region')).toBeUndefined()
  })
})
