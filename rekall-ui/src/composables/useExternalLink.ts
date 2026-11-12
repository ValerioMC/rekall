import { canOpenExternalLink, openExternalLink } from '@/common/native/desktop'
import { useToastStore } from '@/stores/toast.store'

/**
 * Sends a link to the browser through the desktop window, which opens no second window of its own.
 * In a browser tab the link keeps its own behaviour.
 */
export function useExternalLink() {
  function follow(event: MouseEvent, url: string): void {
    if (!canOpenExternalLink()) return
    event.preventDefault()
    openExternalLink(url).catch((caught: unknown) => useToastStore().notifyError(caught))
  }

  return { follow }
}
