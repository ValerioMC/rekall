import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import PathPicker from '@/components/ui/PathPicker.vue'
import { ApiError } from '@/api/client'
import type { DirectoryEntry, DirectoryListing } from '@/model/filesystem'

vi.mock('@/api/filesystem.api', () => ({ fetchDirectory: vi.fn() }))
import { fetchDirectory } from '@/api/filesystem.api'

const HOME = '/Users/me'
const PROJECT = '/Users/me/code/app'

function folder(path: string, names: string[]): DirectoryListing {
  const parts = path.split('/').filter(Boolean)
  const segments = [{ name: '/', path: '/' }, ...parts.map((name, index) => ({
    name,
    path: `/${parts.slice(0, index + 1).join('/')}`
  }))]
  const entries: DirectoryEntry[] = names.map((name) => ({
    name: name.replace(/\/$/, ''),
    path: `${path}/${name.replace(/\/$/, '')}`,
    directory: name.endsWith('/'),
    hidden: name.startsWith('.')
  }))
  return {
    path,
    parent: path === '/' ? null : `/${parts.slice(0, -1).join('/')}`,
    home: HOME,
    segments,
    entries,
    readable: true,
    truncated: false
  }
}

const DISK: Record<string, DirectoryListing> = {
  [PROJECT]: folder(PROJECT, ['.git/', 'docs/', 'src/', '.env', 'README.md', 'package.json']),
  [`${PROJECT}/src`]: folder(`${PROJECT}/src`, ['components/', 'main.ts']),
  '/Users/me/code': folder('/Users/me/code', ['app/', 'other/']),
  [HOME]: folder(HOME, ['code/', 'Downloads/'])
}

/** Resolves the way the server does, for the handful of shapes these tests type. */
function resolve(base: string | null, typed: string): string {
  const start = base ?? HOME
  if (!typed) return start
  if (typed.startsWith('~')) return `${HOME}${typed.slice(1)}`.replace(/\/$/, '')
  if (typed.startsWith('/')) return typed.replace(/\/$/, '') || '/'
  return `${start}/${typed}`.replace(/\/$/, '')
}

let wrapper: VueWrapper | null = null
let stored = new Map<string, string>()

/** This environment hands out no usable storage, so the test brings its own. */
function installStorage(): void {
  stored = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (name: string) => stored.get(name) ?? null,
      setItem: (name: string, value: string) => void stored.set(name, value),
      removeItem: (name: string) => void stored.delete(name),
      clear: () => stored.clear()
    }
  })
}

function mountPicker(props: Partial<{ root: string | null; start: string | null }> = {}): VueWrapper {
  wrapper = mount(PathPicker, {
    attachTo: document.body,
    props: {
      root: PROJECT,
      start: null,
      locate: () => ({ left: 100, top: 100, bottom: 118 }),
      format: (path: string, directory: boolean) => `\`${path}${directory ? '/' : ''}\``,
      ...props
    }
  })
  return wrapper
}

function key(init: KeyboardEventInit): void {
  const field = document.querySelector<HTMLInputElement>('[data-testid="path-picker-filter"]')!
  field.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }))
}

function names(): string[] {
  return wrapper!.findAll('[data-testid="path-picker-row"]').map((row) => row.text().replace(/\s*↵ insert$/, ''))
}

async function settle(): Promise<void> {
  await vi.advanceTimersByTimeAsync(200)
  await flushPromises()
}

beforeEach(() => {
  vi.useFakeTimers()
  installStorage()
  vi.mocked(fetchDirectory).mockReset()
  vi.mocked(fetchDirectory).mockImplementation(async (base, typed = '') => {
    const listing = DISK[resolve(base, typed)]
    if (!listing) throw new ApiError(404, `There is no folder at ${resolve(base, typed)}.`, '')
    return listing
  })
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  vi.useRealTimers()
})

describe('PathPicker', () => {
  it('opens in the project folder, folders first, with dotfiles left out', async () => {
    mountPicker()
    await settle()

    expect(fetchDirectory).toHaveBeenCalledWith(PROJECT, '')
    expect(names()).toEqual(['docs', 'src', 'README.md', 'package.json'])
    const crumbs = wrapper!.findAll('[data-testid="path-picker-crumb"]').map((crumb) => crumb.text())
    expect(crumbs).toEqual(['~', 'code', 'app'])
    expect(wrapper!.find('[data-testid="path-picker-crumb"][data-project]').text()).toBe('app')
    expect(document.activeElement).toBe(wrapper!.find('[data-testid="path-picker-filter"]').element)
  })

  it('previews exactly what the highlighted row would insert', async () => {
    mountPicker()
    await settle()
    key({ key: 'ArrowDown' })
    await flushPromises()

    expect(wrapper!.find('[data-testid="path-picker-preview"]').text()).toBe(`\`${PROJECT}/src/\``)
  })

  it('inserts the highlighted file on Enter and the folder in view on ⌘Enter', async () => {
    mountPicker()
    await settle()
    key({ key: 'ArrowDown' })
    key({ key: 'ArrowDown' })
    key({ key: 'Enter' })
    key({ key: 'Enter', metaKey: true })

    expect(wrapper!.emitted('pick')).toEqual([
      [`${PROJECT}/README.md`, false],
      [PROJECT, true]
    ])
  })

  it('opens a folder with the right arrow and goes back up with backspace, keeping the place', async () => {
    mountPicker()
    await settle()
    key({ key: 'ArrowDown' })
    key({ key: 'ArrowRight' })
    await settle()
    expect(names()).toEqual(['components', 'main.ts'])

    key({ key: 'Backspace' })
    await settle()
    expect(names()[0]).toBe('docs')
    expect(wrapper!.find('[aria-selected="true"]').text()).toContain('src')
    expect(wrapper!.emitted('visit')!.map(([path]) => path)).toEqual([PROJECT, `${PROJECT}/src`, PROJECT])
  })

  it('reads a typed slash as a folder to list and the rest as a filter', async () => {
    mountPicker()
    await settle()
    await wrapper!.find('[data-testid="path-picker-filter"]').setValue('src/ma')
    await settle()

    expect(fetchDirectory).toHaveBeenLastCalledWith(PROJECT, 'src/')
    expect(names()).toEqual(['main.ts'])
    expect(wrapper!.find('mark.path-match').text()).toBe('ma')
  })

  it('says so when a typed folder does not exist', async () => {
    mountPicker()
    await settle()
    await wrapper!.find('[data-testid="path-picker-filter"]').setValue('nowhere/')
    await settle()

    expect(wrapper!.find('[data-testid="path-picker-failure"]').text()).toContain('There is no folder at')
    expect(wrapper!.find('[data-testid="path-picker-filter"]').attributes('aria-invalid')).toBe('true')
  })

  it('shows dotfiles on the toggle, and when the filter starts with a dot', async () => {
    mountPicker()
    await settle()
    await wrapper!.find('[data-testid="path-picker-filter"]').setValue('.e')
    await settle()
    expect(names()).toEqual(['.env'])

    await wrapper!.find('[data-testid="path-picker-filter"]').setValue('')
    await wrapper!.find('[data-testid="path-picker-hidden"]').trigger('click')
    expect(names()).toContain('.git')
    expect(stored.get('rekall.pathPicker.showHidden')).toBe('true')
  })

  it('clears a typed filter on the first Escape and closes on the second', async () => {
    mountPicker()
    await settle()
    await wrapper!.find('[data-testid="path-picker-filter"]').setValue('src')
    key({ key: 'Escape' })
    await flushPromises()
    expect((wrapper!.find('[data-testid="path-picker-filter"]').element as HTMLInputElement).value).toBe('')
    expect(wrapper!.emitted('close')).toBeUndefined()

    key({ key: 'Escape' })
    expect(wrapper!.emitted('close')).toHaveLength(1)
  })

  it('goes home and back to the project from the header', async () => {
    mountPicker()
    await settle()
    await wrapper!.find('[data-testid="path-picker-home"]').trigger('click')
    await settle()
    expect(names()).toEqual(['code', 'Downloads'])

    await wrapper!.find('[data-testid="path-picker-project"]').trigger('click')
    await settle()
    expect(names()).toEqual(['docs', 'src', 'README.md', 'package.json'])
  })

  it('falls back to home and says why when the project folder is gone', async () => {
    mountPicker({ root: '/Users/me/gone' })
    await settle()

    expect(wrapper!.find('[data-testid="path-picker-notice"]').text()).toContain('/Users/me/gone is not there any more')
    expect(names()).toEqual(['code', 'Downloads'])
  })

  it('keeps every keystroke from reaching the console behind it', async () => {
    const behind = vi.fn()
    window.addEventListener('keydown', behind)
    mountPicker()
    await settle()
    key({ key: 'j' })
    window.removeEventListener('keydown', behind)

    expect(behind).not.toHaveBeenCalled()
  })

  it('closes on a click outside it', async () => {
    mountPicker()
    await settle()
    document.body.dispatchEvent(new Event('pointerdown', { bubbles: true }))

    expect(wrapper!.emitted('close')).toHaveLength(1)
  })
})
