import { computed, onBeforeUnmount, onMounted, ref, watch, type Ref } from 'vue'
import { findTextRanges } from '@/common/markdown/text-matches'

const MATCHES_HIGHLIGHT = 'rekall-find'
const CURRENT_HIGHLIGHT = 'rekall-find-current'

/** Searchable instances, oldest first: ⌘F belongs to the last one mounted, the pane on top (a dialog over a page). */
const searchable: symbol[] = []

function paintHighlights(matches: Range[], current: Range | null): void {
  if (typeof CSS === 'undefined' || !CSS.highlights || typeof Highlight === 'undefined') return
  CSS.highlights.set(MATCHES_HIGHLIGHT, new Highlight(...matches))
  if (current) CSS.highlights.set(CURRENT_HIGHLIGHT, new Highlight(current))
  else CSS.highlights.delete(CURRENT_HIGHLIGHT)
}

function clearHighlights(): void {
  if (typeof CSS === 'undefined' || !CSS.highlights) return
  CSS.highlights.delete(MATCHES_HIGHLIGHT)
  CSS.highlights.delete(CURRENT_HIGHLIGHT)
}

function isFindShortcut(event: KeyboardEvent): boolean {
  return (event.metaKey || event.ctrlKey) && !event.shiftKey && !event.altKey && event.code === 'KeyF'
}

/**
 * Find-in-page for rendered markdown, which has no editor to search in. ⌘F / Ctrl+F opens it;
 * matches are painted with the CSS Custom Highlight API, so the rendered DOM is never touched.
 */
export function useReadModeSearch(root: Ref<HTMLElement | null>, options: { enabled: () => boolean }) {
  const identity = Symbol('read-mode-search')
  const open = ref(false)
  const query = ref('')
  const position = ref(0)
  /** Bumped on every ⌘F, so a bar that is already open takes focus again. */
  const focusRequests = ref(0)
  let matches: Range[] = []
  const count = ref(0)
  const label = computed(() => (query.value ? `${count.value ? position.value + 1 : 0}/${count.value}` : ''))

  function render(): void {
    const current = matches[position.value] ?? null
    paintHighlights(matches, current)
    current?.startContainer.parentElement?.scrollIntoView({ block: 'center' })
  }

  function search(): void {
    const page = root.value?.querySelector('.md-editor-preview')
    matches = open.value && page ? findTextRanges(page, query.value) : []
    count.value = matches.length
    position.value = 0
    render()
  }

  function step(direction: 1 | -1): void {
    if (matches.length === 0) return
    position.value = (position.value + direction + matches.length) % matches.length
    render()
  }

  function close(): void {
    open.value = false
    query.value = ''
  }

  function onKeydown(event: KeyboardEvent): void {
    if (!isFindShortcut(event) || searchable[searchable.length - 1] !== identity) return
    event.preventDefault()
    open.value = true
    focusRequests.value++
  }

  function register(): void {
    const registered = searchable.includes(identity)
    if (options.enabled() && !registered) searchable.push(identity)
    if (!options.enabled() && registered) searchable.splice(searchable.indexOf(identity), 1)
  }

  watch([open, query], search)
  // The preview renders after its text changes, and a re-render drops the nodes the ranges point into.
  let rendering: MutationObserver | null = null
  watch(open, (isOpen) => {
    rendering?.disconnect()
    rendering = null
    if (!isOpen) return clearHighlights()
    // Never the whole root: the bar repaints its own counter on every step, and that would restart the search.
    const rendered = root.value?.querySelector('.md-editor')
    if (typeof MutationObserver === 'undefined' || !rendered) return
    rendering = new MutationObserver(search)
    rendering.observe(rendered, { childList: true, subtree: true, characterData: true })
  })

  watch(options.enabled, register)

  onMounted(() => {
    register()
    window.addEventListener('keydown', onKeydown)
  })
  onBeforeUnmount(() => {
    if (searchable.includes(identity)) searchable.splice(searchable.indexOf(identity), 1)
    window.removeEventListener('keydown', onKeydown)
    rendering?.disconnect()
    if (open.value) clearHighlights()
  })

  return { open, query, label, count, focusRequests, next: () => step(1), previous: () => step(-1), close }
}
