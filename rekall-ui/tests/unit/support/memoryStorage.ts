/** A `localStorage` of the test's own: this environment hands out no usable one. */
export function installStorage(over: Partial<Storage> = {}): Map<string, string> {
  const values = new Map<string, string>()
  Object.defineProperty(window, 'localStorage', {
    configurable: true,
    value: {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => void values.set(key, value),
      removeItem: (key: string) => void values.delete(key),
      clear: () => values.clear(),
      ...over
    }
  })
  return values
}
