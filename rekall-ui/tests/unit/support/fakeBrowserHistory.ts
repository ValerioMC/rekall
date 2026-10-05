interface Entry {
  state: unknown
  url: string
}

/**
 * A browser's session history for a router's web history to run on. jsdom moves the location on
 * `history.go` but keeps the old entry's state and never fires `popstate`, which is the half the
 * router reads. The location still follows each entry, through jsdom's own `replaceState`.
 */
export function installFakeBrowserHistory(): () => void {
  const native = window.history
  const moveLocation = (url: string): void => History.prototype.replaceState.call(native, null, '', url)
  const entries: Entry[] = [{ state: null, url: window.location.pathname }]
  let index = 0

  function current(): Entry {
    const entry = entries[index]
    if (!entry) throw new Error(`no history entry at ${index}`)
    return entry
  }

  const fake = {
    scrollRestoration: 'manual' as ScrollRestoration,
    get state(): unknown {
      return current().state
    },
    get length(): number {
      return entries.length
    },
    pushState(state: unknown, _unused: string, url: string): void {
      entries.splice(index + 1, entries.length, { state, url })
      index += 1
      moveLocation(url)
    },
    replaceState(state: unknown, _unused: string, url: string): void {
      entries[index] = { state, url }
      moveLocation(url)
    },
    go(delta: number): void {
      const target = index + delta
      if (target < 0 || target >= entries.length) return
      index = target
      moveLocation(current().url)
      // jsdom's PopStateEvent drops the `state` it is given, so the event carries it itself.
      window.dispatchEvent(Object.assign(new Event('popstate'), { state: current().state }))
    },
    back(): void {
      fake.go(-1)
    },
    forward(): void {
      fake.go(1)
    }
  }

  Object.defineProperty(window, 'history', { value: fake, configurable: true })
  return () => Object.defineProperty(window, 'history', { value: native, configurable: true })
}
