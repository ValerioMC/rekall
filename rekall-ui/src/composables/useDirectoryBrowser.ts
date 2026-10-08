import { computed, nextTick, onUnmounted, ref, watch } from 'vue'
import { fetchDirectory } from '@/api/filesystem.api'
import { ApiError } from '@/api/client'
import { filterEntries, splitPathQuery, type PathRow } from '@/common/path/path-query'
import { pathPickerShowsHidden, setPathPickerShowsHidden } from '@/common/config/path-picker'
import type { DirectoryEntry, DirectoryListing } from '@/model/filesystem'

export interface Crumb {
  readonly label: string
  readonly path: string
  readonly project: boolean
  readonly home: boolean
}

export interface DirectoryBrowserOptions {
  /** The project's folder: what the project button returns to, and the fallback when a folder is gone. */
  root: () => string | null
  /** Where to open: the folder last left, or the root. */
  start: string | null
  /** The folder now in view, once listed. */
  onVisit: (folder: string) => void
  /** After every move to another folder, so the caller can put the focus back on the filter. */
  onNavigate: () => void
  /** The highlighted row changed by a load, for the caller to scroll it into view. */
  onHighlightMoved: () => void
  /** Lists folders only, for a caller that is choosing one. */
  foldersOnly?: () => boolean
}

const SLOW_AFTER_MS = 140
const TYPE_SETTLE_MS = 90

/**
 * A folder of the machine as the path picker browses it. The filter reads like a shell prompt:
 * anything up to a slash names a folder to list first, resolved by the server against the folder
 * in view, and the rest filters its entries. Listings are cached for the picker's life, a load
 * that takes a moment says so, and a folder that is gone falls back to the project, then home.
 */
export function useDirectoryBrowser(options: DirectoryBrowserOptions) {
  const folder = ref<string | null>(options.start ?? options.root())
  const query = ref('')
  const parsed = computed(() => splitPathQuery(query.value))
  const listing = ref<DirectoryListing | null>(null)
  const failure = ref<string | null>(null)
  const notice = ref<string | null>(null)
  const loading = ref(false)
  /** Loading for long enough to be worth showing: a cached or local listing never flashes a bar. */
  const slow = ref(false)
  const showHidden = ref(pathPickerShowsHidden())
  const highlighted = ref(0)
  /** The project folder as the server spells it, once known, to mark it in the breadcrumb. */
  const projectPath = ref<string | null>(null)
  /** After going up, the folder just left is highlighted, the way Finder keeps your place. */
  let focusAfterLoad: string | null = null

  const cache = new Map<string, DirectoryListing>()
  let ticket = 0
  let slowTimer: ReturnType<typeof setTimeout> | undefined
  let typeTimer: ReturnType<typeof setTimeout> | undefined

  async function load(base: string | null, typed: string): Promise<void> {
    const mine = ++ticket
    const key = `${base ?? ''}\u0000${typed}`
    const cached = cache.get(key)
    if (cached) {
      apply(cached)
      return
    }
    loading.value = true
    clearTimeout(slowTimer)
    slowTimer = setTimeout(() => {
      if (mine === ticket && loading.value) slow.value = true
    }, SLOW_AFTER_MS)
    try {
      const next = await fetchDirectory(base, typed)
      cache.set(key, next)
      if (mine === ticket) apply(next)
    } catch (error) {
      if (mine !== ticket) return
      if (!typed && listing.value === null && base !== null) {
        fallBackFrom(base)
        return
      }
      failure.value = error instanceof ApiError ? error.message : 'Rekall could not list that folder.'
      settle()
    }
  }

  /** The folder it opened on is gone (moved, unmounted): try the project, then home, and say so. */
  function fallBackFrom(base: string): void {
    const root = options.root()
    const next = base !== root && root ? root : null
    notice.value = `${base} is not there any more, so this opened ${next === null ? 'in your home folder' : 'in the project folder'}.`
    folder.value = next
  }

  function apply(next: DirectoryListing): void {
    listing.value = next
    failure.value = null
    settle()
    options.onVisit(next.path)
    const root = options.root()
    if (root && !projectPath.value && folder.value === root && !parsed.value.folder) {
      projectPath.value = next.path
    }
  }

  function settle(): void {
    loading.value = false
    slow.value = false
    clearTimeout(slowTimer)
  }

  watch(
    () => [folder.value, parsed.value.folder] as const,
    ([base, typed], previous) => {
      clearTimeout(typeTimer)
      const onlyTyping = previous !== undefined && previous[0] === base
      if (onlyTyping) typeTimer = setTimeout(() => void load(base, typed), TYPE_SETTLE_MS)
      else void load(base, typed)
    },
    { immediate: true }
  )

  const rows = computed<PathRow[]>(() => {
    if (!listing.value || failure.value) return []
    const entries = options.foldersOnly?.() ? listing.value.entries.filter((entry) => entry.directory) : listing.value.entries
    return filterEntries(entries, parsed.value.needle, showHidden.value)
  })
  const hiddenCount = computed(() => listing.value?.entries.filter((entry) => entry.hidden).length ?? 0)
  const current = computed<DirectoryEntry | null>(() => rows.value[highlighted.value]?.entry ?? null)

  watch(rows, (next) => {
    const target = focusAfterLoad
    focusAfterLoad = null
    const found = target ? next.findIndex((row) => row.entry.path === target) : -1
    highlighted.value = found >= 0 ? found : 0
    void nextTick(options.onHighlightMoved)
  })

  /** From the filesystem root, or from `~` when the folder is under home: the part above is noise. */
  const crumbs = computed<Crumb[]>(() => {
    const view = listing.value
    if (!view) return []
    const homeAt = view.segments.findIndex((segment) => segment.path === view.home)
    const from = homeAt >= 0 ? homeAt : 0
    return view.segments.slice(from).map((segment, index) => ({
      label: index === 0 && homeAt >= 0 ? '~' : segment.name,
      path: segment.path,
      project: segment.path === projectPath.value,
      home: index === 0 && homeAt >= 0
    }))
  })

  const atProject = computed(() => projectPath.value !== null && listing.value?.path === projectPath.value)
  const atHome = computed(() => listing.value !== null && listing.value.path === listing.value.home)

  function goTo(path: string | null, highlight: string | null = null): void {
    focusAfterLoad = highlight
    query.value = ''
    folder.value = path
    options.onNavigate()
  }

  function open(entry: DirectoryEntry): void {
    if (entry.directory) goTo(entry.path)
  }

  function up(): void {
    const view = listing.value
    if (view?.parent) goTo(view.parent, view.path)
  }

  function jumpToCrumb(index: number): void {
    const crumb = crumbs.value[index]
    if (crumb) goTo(crumb.path, crumbs.value[index + 1]?.path ?? null)
  }

  function goProject(): void {
    const root = options.root()
    if (root) goTo(projectPath.value ?? root)
  }

  function goHome(): void {
    goTo(listing.value?.home ?? null)
  }

  function toggleHidden(): void {
    showHidden.value = !showHidden.value
    setPathPickerShowsHidden(showHidden.value)
  }

  /** Tab on a highlighted name completes it into the filter's folder part, the way a shell does. */
  function complete(entry: DirectoryEntry): void {
    if (entry.directory) open(entry)
    else query.value = `${parsed.value.folder}${entry.name}`
  }

  function move(delta: number): void {
    if (!rows.value.length) return
    highlighted.value = (highlighted.value + delta + rows.value.length) % rows.value.length
  }

  /**
   * Opened somewhere other than the project: learns how the server spells the project folder, so
   * its crumb can be marked, without waiting for someone to go there.
   */
  function learnProjectPath(): void {
    const root = options.root()
    if (!root || folder.value === root) return
    fetchDirectory(root)
      .then((rootListing) => {
        cache.set(`${root}\u0000`, rootListing)
        projectPath.value = rootListing.path
      })
      .catch(() => undefined)
  }

  onUnmounted(() => {
    clearTimeout(slowTimer)
    clearTimeout(typeTimer)
  })

  return {
    query,
    parsed,
    listing,
    failure,
    notice,
    slow,
    showHidden,
    highlighted,
    rows,
    hiddenCount,
    current,
    crumbs,
    atProject,
    atHome,
    open,
    up,
    jumpToCrumb,
    goProject,
    goHome,
    toggleHidden,
    complete,
    move,
    learnProjectPath
  }
}
