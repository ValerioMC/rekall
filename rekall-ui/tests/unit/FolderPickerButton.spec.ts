import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import FolderPickerButton from '@/features/project/components/FolderPickerButton.vue'
import type { DirectoryListing } from '@/model/filesystem'

vi.mock('@/api/filesystem.api', () => ({ fetchDirectory: vi.fn() }))
import { fetchDirectory } from '@/api/filesystem.api'

const PROJECT = '/Users/me/code/app'

const LISTING: DirectoryListing = {
  path: PROJECT,
  parent: '/Users/me/code',
  home: '/Users/me',
  segments: [{ name: '/', path: '/' }],
  entries: [
    { name: 'billing', path: `${PROJECT}/billing`, directory: true, hidden: false },
    { name: 'README.md', path: `${PROJECT}/README.md`, directory: false, hidden: false }
  ],
  readable: true,
  truncated: false
}

let wrapper: VueWrapper | null = null

function mountButton(current = ''): VueWrapper {
  wrapper = mount(FolderPickerButton, {
    attachTo: document.body,
    props: { root: PROJECT, current },
    attrs: { 'data-testid': 'browse' }
  })
  return wrapper
}

function picker(): HTMLElement | null {
  return document.querySelector<HTMLElement>('[data-testid="path-picker"]')
}

beforeEach(() => {
  vi.mocked(fetchDirectory).mockReset()
  vi.mocked(fetchDirectory).mockResolvedValue(LISTING)
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
})

describe('FolderPickerButton', () => {
  it('opens the folder picker over the project folder', async () => {
    mountButton()
    await wrapper!.get('[data-testid="browse"]').trigger('click')
    await flushPromises()

    expect(picker()?.getAttribute('aria-label')).toBe('Choose a folder')
    expect(fetchDirectory).toHaveBeenCalledWith(PROJECT, '')
  })

  it('opens where the field already points', async () => {
    mountButton('/Users/me/code/app/billing')
    await wrapper!.get('[data-testid="browse"]').trigger('click')
    await flushPromises()

    expect(fetchDirectory).toHaveBeenCalledWith('/Users/me/code/app/billing', '')
  })

  it('emits the chosen folder and closes', async () => {
    mountButton()
    await wrapper!.get('[data-testid="browse"]').trigger('click')
    await flushPromises()
    document
      .querySelector('[data-testid="path-picker-filter"]')!
      .dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', metaKey: true, bubbles: true, cancelable: true }))
    await flushPromises()

    expect(wrapper!.emitted('pick')).toEqual([[PROJECT]])
    expect(picker()).toBeNull()
  })

  it('closes when the button is pressed again', async () => {
    mountButton()
    const button = wrapper!.get('[data-testid="browse"]')
    await button.trigger('click')
    await flushPromises()
    await button.trigger('click')
    await flushPromises()

    expect(picker()).toBeNull()
  })
})
