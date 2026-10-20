import { computed, onScopeDispose, ref, watch, type Ref } from 'vue'
import { walkthroughBeats, walkthroughSpotlight } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import { advance, enter, leave, leaveTo, moveTo, retreat, startAt, topOf, type WalkStack } from '@/features/diagram/lib/walkthrough/walkthrough-cursor'
import type { SheetDrawing } from '@/features/diagram/lib/sheet-drawing'
import type { DiagramSheet } from '@/features/diagram/model/diagram'

/** A sequence is read as a static diagram or played as a walkthrough. */
export type ViewMode = 'diagram' | 'walkthrough'

export type WalkPace = 'slow' | 'normal' | 'fast'

/** How long a beat stays on screen while playing: long enough to read its card. */
export const BEAT_MILLISECONDS: Readonly<Record<WalkPace, number>> = { slow: 5200, normal: 3200, fast: 1600 }

/**
 * Plays the open sequence one beat at a time. The walkthrough owns which sheet is open while it
 * runs: stepping into a call opens the call's sheet and leaving returns to the caller. A sheet
 * picked by hand starts it again there.
 */
export function useWalkthrough(sheets: Ref<readonly DiagramSheet[]>, sheetIndex: Ref<number>, drawing: Ref<SheetDrawing | null>) {
  const stack = ref<WalkStack>(startAt(sheetIndex.value))
  const playing = ref(false)
  const pace = ref<WalkPace>('normal')

  const sheet = computed(() => sheets.value[sheetIndex.value] ?? null)
  const beats = computed(() => {
    const drawn = drawing.value
    return drawn?.drawing === 'sequence' ? walkthroughBeats(drawn.scene, drawn.calls, sheet.value?.walkthrough ?? null) : []
  })
  const position = computed(() => topOf(stack.value).position)
  const beat = computed(() => beats.value[position.value] ?? null)
  const spotlight = computed(() => walkthroughSpotlight(beats.value, position.value))
  const external = computed<ReadonlySet<string>>(() => new Set(sheet.value?.walkthrough?.external ?? []))
  const atEnd = computed(() => stack.value.length === 1 && beats.value.length > 0 && position.value >= beats.value.length - 1)

  watch(sheetIndex, (index) => {
    if (topOf(stack.value).sheetIndex !== index) stack.value = startAt(index)
  })
  // Leaving a call onward can land one past the caller's last beat; its count is known only once drawn.
  watch(beats, (drawn) => {
    if (drawn.length > 0 && position.value > drawn.length - 1) stack.value = moveTo(stack.value, position.value, drawn.length)
  })

  function apply(next: WalkStack): void {
    stack.value = next
    sheetIndex.value = topOf(next).sheetIndex
  }

  function next(): void {
    if (beats.value.length === 0) return
    const moved = advance(stack.value, beats.value.length)
    if (moved) apply(moved)
    else playing.value = false
  }

  function previous(): void {
    const moved = retreat(stack.value)
    if (moved) apply(moved)
  }

  function goTo(index: number): void {
    if (beats.value.length > 0 && index >= 0) apply(moveTo(stack.value, index, beats.value.length))
  }

  function stepInto(): void {
    const current = beat.value
    if (current === null || current.opens === null || current.opens >= sheets.value.length) return
    apply(enter(stack.value, current.opens, current.message.text))
  }

  function stepOut(): void {
    apply(leave(stack.value, false))
  }

  function backTo(depth: number): void {
    apply(leaveTo(stack.value, depth))
  }

  function restart(): void {
    apply(startAt(stack.value[0]!.sheetIndex))
  }

  /** Playing from the last beat starts over: there is nothing after it to play. */
  function toggle(): void {
    if (!playing.value && atEnd.value) restart()
    playing.value = !playing.value
  }

  let timer: ReturnType<typeof setTimeout> | null = null
  function cancelTimer(): void {
    if (timer !== null) clearTimeout(timer)
    timer = null
  }
  watch(
    [playing, stack, beats, pace],
    () => {
      cancelTimer()
      if (playing.value && beats.value.length > 0) timer = setTimeout(next, BEAT_MILLISECONDS[pace.value])
    },
    { immediate: true }
  )
  onScopeDispose(cancelTimer)

  return { stack, playing, pace, beats, position, beat, spotlight, external, atEnd, next, previous, goTo, stepInto, stepOut, backTo, restart, toggle }
}

export type WalkthroughPlayer = ReturnType<typeof useWalkthrough>
