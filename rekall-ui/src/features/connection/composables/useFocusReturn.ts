/** Gives focus back to what had it when a dialog or sheet opened, so the keyboard reader is not left at the top of the page. */
export function useFocusReturn() {
  let invoker: HTMLElement | null = null

  function remember(): void {
    invoker = document.activeElement instanceof HTMLElement ? document.activeElement : null
  }

  function restore(): void {
    if (invoker && document.contains(invoker)) invoker.focus()
    invoker = null
  }

  return { remember, restore }
}
