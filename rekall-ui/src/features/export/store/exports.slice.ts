import { computed, ref } from 'vue'
import { deleteExport, fetchExports } from '@/features/export/api/project-export.api'
import type { ProjectExportEntry } from '@/features/export/model/project-export'
import type { ProjectExportId } from '@/model/branded'
import { upsertById } from '../../../stores/console/rows'

/** Every export the server keeps, newest first: read when the Exports page opens, and added to when the dialog finishes one. */
export function useProjectExports() {
  const exportEntries = ref<readonly ProjectExportEntry[]>([])
  const exportsLoadedAt = ref<Date | null>(null)
  const exportsLoaded = computed(() => exportsLoadedAt.value !== null)

  async function loadExports(): Promise<void> {
    exportEntries.value = await fetchExports()
    exportsLoadedAt.value = new Date()
  }

  function addExport(entry: ProjectExportEntry): void {
    exportEntries.value = upsertById(exportEntries.value, entry, 'first')
  }

  async function removeExport(id: ProjectExportId): Promise<void> {
    await deleteExport(id)
    exportEntries.value = exportEntries.value.filter((entry) => entry.id !== id)
  }

  return { exportEntries, exportsLoadedAt, exportsLoaded, loadExports, addExport, removeExport }
}
