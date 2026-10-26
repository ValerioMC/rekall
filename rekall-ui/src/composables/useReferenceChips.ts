import { nextTick, onMounted, watch, type Ref } from 'vue'

/**
 * Keeps one kind of `[[...]]` chip in a read view true to the list it names: painted when the page
 * renders (`paint`) and again whenever the list changes, so a rename shows without a reload.
 */
export function useReferenceChips<T>(
  root: Ref<HTMLElement | null>,
  targets: () => T,
  paintUnder: (root: HTMLElement, targets: T) => void
): { paint: () => void } {
  function paint(): void {
    void nextTick(() => {
      if (root.value) paintUnder(root.value, targets())
    })
  }

  onMounted(paint)
  watch(targets, paint)
  return { paint }
}
