import { computed, ref } from 'vue'

/** The reader's own marks while going through a diff: files read (and so folded) and files folded by hand. */
export function useChangeReview() {
  const viewed = ref<ReadonlySet<string>>(new Set())
  const folded = ref<ReadonlySet<string>>(new Set())

  const isViewed = (path: string): boolean => viewed.value.has(path)
  const isShut = (path: string): boolean => viewed.value.has(path) || folded.value.has(path)

  function toggleViewed(path: string): void {
    viewed.value = flipped(viewed.value, path)
  }

  function toggleShut(path: string): void {
    if (isShut(path)) {
      viewed.value = without(viewed.value, path)
      folded.value = without(folded.value, path)
    } else {
      folded.value = flipped(folded.value, path)
    }
  }

  function reset(): void {
    viewed.value = new Set()
    folded.value = new Set()
  }

  return { viewed: computed(() => viewed.value), isViewed, isShut, toggleViewed, toggleShut, reset }
}

function flipped(set: ReadonlySet<string>, path: string): ReadonlySet<string> {
  return set.has(path) ? without(set, path) : new Set([...set, path])
}

function without(set: ReadonlySet<string>, path: string): ReadonlySet<string> {
  return new Set([...set].filter((entry) => entry !== path))
}
