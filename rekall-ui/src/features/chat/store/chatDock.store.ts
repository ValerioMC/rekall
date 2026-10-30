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
 * Where the chat window sits in the console: shown or hidden, how wide, expanded or not. Width and
 * visibility are remembered per viewer. `areaWidth` is the room beside the navigator and note
 * list; the window docks inline while the note pane keeps 360 px, and overlays it otherwise.
 */
export const useChatDockStore = defineStore('chatDock', () => {
  const hidden = ref(readHidden())
  const width = ref(readWidth())
  const expanded = ref(false)
  const areaWidth = ref(Number.POSITIVE_INFINITY)

  const overlays = computed(() => areaWidth.value - width.value < NOTE_MIN_WIDTH)
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

  function setWidth(value: number): void {
    width.value = Math.round(Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, value)))
    remember(WIDTH_KEY, String(width.value))
  }

  function observe(area: number): void {
    areaWidth.value = area
  }

  return {
    hidden,
    width,
    expanded,
    shownWidth,
    maxWidth,
    mode,
    show: () => setHidden(false),
    hide: () => setHidden(true),
    toggle: () => setHidden(!hidden.value),
    setWidth,
    resetWidth: () => setWidth(DEFAULT_WIDTH),
    nudge: (delta: number) => setWidth(shownWidth.value + delta),
    expand: () => (expanded.value = true),
    restore: () => (expanded.value = false),
    toggleExpanded: () => (expanded.value = !expanded.value),
    observe
  }
})
