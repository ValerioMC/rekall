import { onScopeDispose, watch } from 'vue'

const ATTENTION_PREFIX = '• '

/** Marks the browser tab's title while something waits for the person, so a hidden tab shows it. */
export function useAttentionTitle(waiting: () => boolean): void {
  function mark(): void {
    if (!document.title.startsWith(ATTENTION_PREFIX)) document.title = `${ATTENTION_PREFIX}${document.title}`
  }

  function clear(): void {
    if (document.title.startsWith(ATTENTION_PREFIX)) document.title = document.title.slice(ATTENTION_PREFIX.length)
  }

  watch(waiting, (isWaiting) => (isWaiting ? mark() : clear()), { immediate: true })
  onScopeDispose(clear)
}
