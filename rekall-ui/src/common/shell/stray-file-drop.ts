/** What takes a dropped file on its own: a writing surface and a file field. */
const TAKES_FILES = '.cm-content[contenteditable="true"], input[type="file"]'

function takesFiles(target: EventTarget | null): boolean {
  const element = target instanceof Element ? target : target instanceof Node ? target.parentElement : null
  return element?.closest(TAKES_FILES) != null
}

function refuseStray(event: DragEvent): void {
  if (event.defaultPrevented || !event.dataTransfer?.types.includes('Files') || takesFiles(event.target)) return
  event.preventDefault()
  event.dataTransfer.dropEffect = 'none'
}

/**
 * A file let go where nothing takes it makes the window open it, replacing the console. Call once;
 * it returns the way to lift the guard.
 */
export function guardStrayFileDrops(target: Window): () => void {
  const events = ['dragover', 'drop'] as const
  events.forEach((name) => target.addEventListener(name, refuseStray))
  return () => events.forEach((name) => target.removeEventListener(name, refuseStray))
}
