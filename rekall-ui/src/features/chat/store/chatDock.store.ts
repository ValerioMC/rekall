import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export type DockMode = 'inline' | 'overlay' | 'full' | 'hidden'

const WIDTH_KEY = 'rekall.chat.width'
const HIDDEN_KEY = 'rekall.chat.hidden'

export const DEFAULT_WIDTH = 440
export const MIN_WIDTH = 360
export const MAX_WIDTH = 720
export const NOTE_MIN_WIDTH = 360
export const OVERLAY_MARGIN = 48

function readWidth(): number {
  try {
    const stored = Number(window.localStorage.getItem(WIDTH_KEY))
    return Number.isFinite(stored) && stored >= MIN_WIDTH && stored <= MAX_WIDTH ? stored : DEFAULT_WIDTH
  } catch {
    return DEFAULT_WIDTH
  }
}

function readHidden(): boolean {
  try {
    return window.localStorage.getItem(HIDDEN_KEY) === 'true'
  } catch {
    return false
  }
}

function remember(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value)
  } catch {
  }
}

/**
 * Where the chat window sits: shown or hidden, how wide, expanded or not. Width and visibility are
 * remembered per viewer. `areaWidth` is the room it docks in; on the console it docks inline while
 * the note pane keeps 360 px, and overlays it otherwise, on any other page it always overlays.
 * `requested` opens it with no session yet, for its empty state.
 */
export const useChatDockStore = defineStore('chatDock', () => {
  const hidden = ref(readHidden())
  const width = ref(readWidth())
  const expanded = ref(false)
  const requested = ref(false)
  const areaWidth = ref(Number.POSITIVE_INFINITY)
  const inlineAllowed = ref(true)

  const overlays = computed(() => !inlineAllowed.value || areaWidth.value - width.value < NOTE_MIN_WIDTH)
  const maxWidth = computed(() => {
    const room = overlays.value ? areaWidth.value - OVERLAY_MARGIN : areaWidth.value - NOTE_MIN_WIDTH
    return Math.max(MIN_WIDTH, Math.min(MAX_WIDTH, room))
  })
  const shownWidth = computed(() => Math.min(width.value, maxWidth.value))
  const mode = computed<DockMode>(() => {
    if (hidden.value) return 'hidden'
    if (expanded.value) return 'full'
    return overlays.value ? 'overlay' : 'inline'
  })

  function setHidden(value: boolean): void {
    hidden.value = value
    remember(HIDDEN_KEY, String(value))
  }

  function open(): void {
    requested.value = true
    setHidden(false)
  }

  function hide(): void {
    requested.value = false
    setHidden(true)
  }

  function setWidth(value: number): void {
    width.value = Math.round(Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, value)))
    remember(WIDTH_KEY, String(width.value))
  }

  function observe(area: number, inline: boolean): void {
    areaWidth.value = area
    inlineAllowed.value = inline
  }

  return {
    hidden,
    width,
    expanded,
    requested,
    shownWidth,
    maxWidth,
    mode,
    open,
    hide,
    setWidth,
    resetWidth: () => setWidth(DEFAULT_WIDTH),
    nudge: (delta: number) => setWidth(shownWidth.value + delta),
    expand: () => (expanded.value = true),
    restore: () => (expanded.value = false),
    toggleExpanded: () => (expanded.value = !expanded.value),
    observe
  }
})
