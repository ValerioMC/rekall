import { ref, watch, type Ref } from 'vue'
import { ApiError } from '@/api/client'
import { fetchSourceFile } from '@/features/diagram/api/diagrams.api'
import type { SourceExcerpt } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

/** The whole text of the file shown, read once from the project folder and kept while the sheet is open. */
export function useSourceFile(diagramId: Ref<DiagramId>, file: Ref<string | null>) {
  const cache = new Map<string, SourceExcerpt>()
  const excerpt = ref<SourceExcerpt | null>(null)
  const failure = ref<string | null>(null)

  async function load(path: string | null): Promise<void> {
    failure.value = null
    if (path === null) {
      excerpt.value = null
      return
    }
    const known = cache.get(path)
    if (known) {
      excerpt.value = known
      return
    }
    excerpt.value = null
    try {
      const read = await fetchSourceFile(diagramId.value, path)
      cache.set(path, read)
      if (file.value === path) excerpt.value = read
    } catch (caught) {
      if (file.value === path) failure.value = caught instanceof ApiError ? caught.message : 'The code could not be read.'
    }
  }

  watch(file, load, { immediate: true })

  return { excerpt, failure }
}
