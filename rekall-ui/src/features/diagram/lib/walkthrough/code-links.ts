import type { WalkthroughBeat } from './walkthrough-beats'
import type { CodeLinks, SourceLocation } from '@/features/diagram/model/diagram'

/**
 * Where a walkthrough's steps live in the code: the files it touches in the order it reaches
 * them, and the methods a reader can click to walk to the step that calls them.
 */

export function sourcesOf(links: CodeLinks, beat: WalkthroughBeat): readonly SourceLocation[] {
  return links[beat.key] ?? []
}

export function filesOfWalkthrough(beats: readonly WalkthroughBeat[], links: CodeLinks): string[] {
  return [...new Set(beats.flatMap((beat) => sourcesOf(links, beat).map((source) => source.file)))]
}

export function firstBeatInFile(beats: readonly WalkthroughBeat[], links: CodeLinks, file: string): number {
  return beats.findIndex((beat) => sourcesOf(links, beat).some((source) => source.file === file))
}

/** `OrderService::place`, `Order.place(order)` and `place` all name `place`. */
export function methodName(symbol: string | null): string | null {
  if (symbol === null) return null
  const last = symbol.replace(/\(.*$/, '').split(/::|[.#\s/]/).filter((part) => part.length > 0).pop()
  return last !== undefined && /^[A-Za-z_]\w*$/.test(last) ? last : null
}

/** Method name to its step: replies are skipped, and a name shared by steps goes to the next one still to come, else the first. */
export function linkedMethods(beats: readonly WalkthroughBeat[], links: CodeLinks, position: number): ReadonlyMap<string, number> {
  const steps = new Map<string, number[]>()
  beats.forEach((beat, index) => {
    if (beat.motion === 'reply') return
    for (const source of sourcesOf(links, beat)) {
      const name = methodName(source.symbol)
      if (name !== null) steps.set(name, [...(steps.get(name) ?? []), index])
    }
  })
  return new Map([...steps].map(([name, indexes]) => [name, indexes.find((index) => index >= position) ?? indexes[0]!]))
}
