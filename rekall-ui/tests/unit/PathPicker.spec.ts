import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import PathPicker from '@/components/ui/PathPicker.vue'
import type { PickerMode } from '@/components/ui/path-picker/picker-mode'
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
    hidden: name.startsWith('.'),
    size: name.endsWith('/') ? null : 245_760
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
  [`${HOME}/Desktop`]: folder(`${HOME}/Desktop`, [
    'screens/', '.cache.png', 'Login Flow.PNG', 'logo.svg', 'notes.txt', 'photo.jpeg', 'spin.gif', 'tile.webp', 'wide.jpg'
  ]),
  [`${HOME}/Desktop/screens`]: folder(`${HOME}/Desktop/screens`, ['readme.md']),
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

function mountPicker(props: Partial<{ root: string | null; start: string | null; mode: PickerMode }> = {}): VueWrapper {
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

  it('lists only folders when it is choosing one, and says so', async () => {
    mountPicker({ mode: 'folder' })
    await settle()

    expect(names()).toEqual(['docs', 'src'])
    expect(wrapper!.find('[data-testid="path-picker"]').attributes('aria-label')).toBe('Choose a folder')
    key({ key: 'ArrowDown' })
    key({ key: 'Enter' })
    expect(wrapper!.emitted('pick')).toEqual([[`${PROJECT}/src`, true]])
  })

  describe('choosing an image', () => {
    const DESKTOP = `${HOME}/Desktop`

    function mountImagePicker(start: string = DESKTOP): VueWrapper {
      return mountPicker({ mode: 'image', start })
    }

    it('is titled for it and lists folders and picture files only', async () => {
      mountImagePicker()
      await settle()

      expect(wrapper!.find('[data-testid="path-picker"]').attributes('aria-label')).toBe('Choose an image')
      expect(wrapper!.find('h2').text()).toBe('Choose an image')
      expect(names()).toEqual(['screens', 'Login Flow.PNG', 'logo.svg', 'photo.jpeg', 'spin.gif', 'tile.webp', 'wide.jpg'])
    })

    it('opens a folder on Enter instead of inserting it, and offers no folder insert', async () => {
      mountImagePicker()
      await settle()
      expect(wrapper!.find('[data-testid="path-picker-insert-folder"]').exists()).toBe(false)
      expect(wrapper!.text()).not.toContain('⌘↵')

      key({ key: 'Enter', metaKey: true })
      key({ key: 'Enter' })
      await settle()

      expect(wrapper!.emitted('pick')).toBeUndefined()
      expect(wrapper!.emitted('visit')!.at(-1)).toEqual([`${DESKTOP}/screens`])
    })

    it('picks the highlighted picture on Enter and names the file it will copy', async () => {
      mountImagePicker()
      await settle()
      key({ key: 'ArrowDown' })
      await flushPromises()

      expect(wrapper!.text()).toContain('Copies')
      expect(wrapper!.find('[data-testid="path-picker-preview"]').text()).toBe(`\`${DESKTOP}/Login Flow.PNG\``)

      key({ key: 'Enter' })
      expect(wrapper!.emitted('pick')).toEqual([[`${DESKTOP}/Login Flow.PNG`, false]])
    })

    it('says a folder without pictures has none, and takes nothing from it', async () => {
      mountImagePicker(`${DESKTOP}/screens`)
      await settle()

      expect(wrapper!.find('[data-testid="path-picker-empty"]').text()).toContain('No pictures in this folder.')
      key({ key: 'Enter' })
      key({ key: 'Enter', metaKey: true })
      expect(wrapper!.emitted('pick')).toBeUndefined()
      expect(wrapper!.find('[data-testid="path-picker-preview"]').text()).toBe('')
    })

    it('stays open on a click on its own button and closes on the path button', async () => {
      mountImagePicker()
      await settle()
      const own = document.body.appendChild(document.createElement('button'))
      own.dataset['pathPickerTrigger'] = 'image'
      const other = document.body.appendChild(document.createElement('button'))
      other.dataset['pathPickerTrigger'] = 'path'

      own.dispatchEvent(new Event('pointerdown', { bubbles: true }))
      expect(wrapper!.emitted('close')).toBeUndefined()
      other.dispatchEvent(new Event('pointerdown', { bubbles: true }))
      expect(wrapper!.emitted('close')).toHaveLength(1)
      own.remove()
      other.remove()
    })
  })
})

describe('PathPicker thumbnails', () => {
  const DESKTOP = `${HOME}/Desktop`
  const watched = new Map<Element, IntersectionObserverCallback>()

  class FakeIntersectionObserver {
    constructor(private readonly callback: IntersectionObserverCallback) {}
    observe(target: Element): void {
      watched.set(target, this.callback)
    }
    disconnect(): void {
      for (const [target, callback] of watched) if (callback === this.callback) watched.delete(target)
    }
  }

  function scrollTo(element: Element): void {
    const entry = { isIntersecting: true, target: element } as unknown as IntersectionObserverEntry
    watched.get(element)?.([entry], {} as IntersectionObserver)
  }

  async function showEverything(): Promise<void> {
    for (const element of [...watched.keys()]) scrollTo(element)
    await flushPromises()
  }

  function rowThumbnails(): string[] {
    return wrapper!.findAll('[data-testid="path-picker-row"] img').map((image) => image.attributes('src') ?? '')
  }

  const thumbnailOf = (name: string): string => `/api/image-thumbnail?path=${encodeURIComponent(`${DESKTOP}/${name}`)}`

  beforeEach(() => {
    watched.clear()
    vi.stubGlobal('IntersectionObserver', FakeIntersectionObserver)
  })

  afterEach(() => vi.unstubAllGlobals())

  it('asks for a thumbnail only once its row is in view', async () => {
    mountPicker({ mode: 'image', start: DESKTOP })
    await settle()
    expect(rowThumbnails()).toEqual([])

    const second = wrapper!.findAll('[data-testid="path-picker-row"]')[1]!.find('span').element
    scrollTo(second)
    await flushPromises()

    expect(rowThumbnails()).toEqual([expect.stringContaining(thumbnailOf('Login Flow.PNG'))])
  })

  it('gives every picture row a thumbnail and folders none', async () => {
    mountPicker({ mode: 'image', start: DESKTOP })
    await settle()
    await showEverything()

    expect(rowThumbnails()).toHaveLength(6)
    expect(wrapper!.findAll('[data-testid="path-picker-row"]')[0]!.find('img').exists()).toBe(false)
  })

  it('shows the highlighted picture large, with its size, and follows the arrows', async () => {
    mountPicker({ mode: 'image', start: DESKTOP })
    await settle()
    const preview = (): string => wrapper!.find('[data-testid="picture-preview"]').text()
    expect(preview()).toContain('Open the folder to see its pictures.')

    key({ key: 'ArrowDown' })
    await flushPromises()
    await showEverything()
    expect(preview()).toContain('Login Flow.PNG')
    expect(wrapper!.find('[data-testid="picture-preview-size"]').text()).toBe('240 KB')
    expect(wrapper!.find('[data-testid="picture-preview"] img').attributes('src')).toContain(thumbnailOf('Login Flow.PNG'))

    key({ key: 'ArrowDown' })
    await flushPromises()
    await showEverything()
    expect(wrapper!.find('[data-testid="picture-preview"] img').attributes('src')).toContain(thumbnailOf('logo.svg'))
  })

  it('shows a plain file glyph for a file that cannot be drawn', async () => {
    mountPicker({ mode: 'image', start: DESKTOP })
    await settle()
    await showEverything()
    const row = wrapper!.findAll('[data-testid="path-picker-row"]')[1]!

    await row.find('img').trigger('error')

    expect(row.find('img').exists()).toBe(false)
    expect(row.find('[data-testid="file-glyph"]').exists()).toBe(true)
  })

  it.each(['path', 'folder'] as const)('draws no thumbnail and no preview in %s mode', async (mode) => {
    mountPicker({ mode, start: DESKTOP })
    await settle()
    await showEverything()

    expect(wrapper!.find('img').exists()).toBe(false)
    expect(wrapper!.find('[data-testid="picture-preview"]').exists()).toBe(false)
  })
})
