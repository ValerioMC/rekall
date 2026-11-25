import { computed, ref } from 'vue'
import { ApiError } from '@/api/client'
import { exportFileUrl, exportProject } from '@/features/export/api/project-export.api'
import { drawDiagramImages } from '@/features/export/lib/diagram-images'
import { diagramSourcesOf, selectionSize, tickedMarkdown } from '@/features/export/lib/export-items'
import { useProjectComponentsStore } from '@/features/project/store/projectComponents.store'
import { useProjectSectionsStore } from '@/features/project/store/projectSections.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/features/catalog/model/catalog'
import type { ExportFormat, ExportSelection, ProjectExportEntry } from '@/features/export/model/project-export'
import type { ProjectComponentId, ProjectSectionId } from '@/model/branded'

export type ExportPhase =
  | { readonly kind: 'idle' }
  | { readonly kind: 'drawing'; readonly finished: number; readonly total: number }
  | { readonly kind: 'writing' }
  | { readonly kind: 'done'; readonly entry: ProjectExportEntry }
  | { readonly kind: 'failed'; readonly message: string; readonly recorded: boolean }

const SERVER_FAILURE_STATUS = 500

/**
 * The export dialog's state: what is ticked, the format and the phase. `start` draws the diagrams
 * of what is ticked, posts the export and begins the download by link; `cancel` aborts a drawing.
 */
export function useProjectExport(project: Project, initial: ExportSelection) {
  const console_ = useConsoleStore()
  const sections = useProjectSectionsStore()
  const components = useProjectComponentsStore()

  const format = ref<ExportFormat>('PDF')
  const blueprint = ref(initial.blueprint)
  const sectionIds = ref<readonly ProjectSectionId[]>(initial.sectionIds)
  const componentIds = ref<readonly ProjectComponentId[]>(initial.componentIds)
  const phase = ref<ExportPhase>({ kind: 'idle' })
  let drawing: AbortController | null = null

  const selection = computed<ExportSelection>(() => ({ blueprint: blueprint.value, sectionIds: sectionIds.value, componentIds: componentIds.value }))
  const ticked = computed(() => selectionSize(selection.value))
  const busy = computed(() => phase.value.kind === 'drawing' || phase.value.kind === 'writing')

  function toggleSection(id: ProjectSectionId): void {
    sectionIds.value = toggled(sectionIds.value, id)
  }

  function toggleComponent(id: ProjectComponentId): void {
    componentIds.value = toggled(componentIds.value, id)
  }

  function setSections(ids: readonly ProjectSectionId[], on: boolean): void {
    const rest = sectionIds.value.filter((id) => !ids.includes(id))
    sectionIds.value = on ? [...rest, ...ids] : rest
  }

  function setComponents(ids: readonly ProjectComponentId[], on: boolean): void {
    const rest = componentIds.value.filter((id) => !ids.includes(id))
    componentIds.value = on ? [...rest, ...ids] : rest
  }

  async function start(): Promise<void> {
    if (ticked.value === 0 || busy.value) return
    const controller = new AbortController()
    drawing = controller
    try {
      const images = await drawnDiagrams(controller.signal)
      phase.value = { kind: 'writing' }
      const entry = await exportProject(project.id, { ...selection.value, format: format.value, diagrams: images })
      console_.addExport(entry)
      phase.value = { kind: 'done', entry }
      startDownload(entry)
    } catch (failure: unknown) {
      phase.value = controller.signal.aborted ? { kind: 'idle' } : failedWith(failure)
    } finally {
      drawing = null
    }
  }

  async function drawnDiagrams(signal: AbortSignal) {
    const markdown = tickedMarkdown(project.blueprintMarkdown, sections.sectionsOf(project.id), components.componentsOf(project.id), selection.value)
    const sources = diagramSourcesOf(markdown)
    if (sources.length === 0) return []
    phase.value = { kind: 'drawing', finished: 0, total: sources.length }
    return drawDiagramImages(sources, signal, ({ finished, total }) => (phase.value = { kind: 'drawing', finished, total }))
  }

  function cancel(): void {
    if (phase.value.kind === 'drawing') drawing?.abort()
  }

  return { format, selection, ticked, busy, phase, toggleSection, toggleComponent, setSections, setComponents, setBlueprint: (on: boolean) => (blueprint.value = on), start, cancel }
}

function toggled<Id>(ids: readonly Id[], id: Id): Id[] {
  return ids.includes(id) ? ids.filter((candidate) => candidate !== id) : [...ids, id]
}

function failedWith(failure: unknown): ExportPhase {
  if (failure instanceof ApiError) return { kind: 'failed', message: failure.message, recorded: failure.status >= SERVER_FAILURE_STATUS }
  return { kind: 'failed', message: failure instanceof Error ? failure.message : String(failure), recorded: false }
}

function startDownload(entry: ProjectExportEntry): void {
  const link = document.createElement('a')
  link.href = exportFileUrl(entry.id)
  link.download = entry.fileName
  link.hidden = true
  document.body.append(link)
  link.click()
  link.remove()
}
