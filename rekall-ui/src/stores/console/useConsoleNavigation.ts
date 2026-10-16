import { computed } from 'vue'
import type { CompanyId, DocumentId, ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { ConsoleLocation, NavMode, PaneFocus } from '@/model/console-location'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { SearchHit } from '@/features/search/model/search'
import type { ConsoleState } from './console-state'
import type { useConsoleView } from './useConsoleView'
import type { useSelectedTask } from './useSelectedTask'

type TaskPane = Exclude<PaneFocus, 'note'>

/** Where the reader is: the scope, the task and note in view, and which pane is open over them. */
export function useConsoleNavigation(
  state: ConsoleState,
  view: ReturnType<typeof useConsoleView>,
  selected: ReturnType<typeof useSelectedTask>
) {
  const { tasks, documents, scopeCompany, scopeProject, navMode, selectedTaskId, selectedDocId, paneFocus } =
    state
  const { stepToOpen, noteComposerOpen } = state

  const location = computed<ConsoleLocation>(() => ({
    mode: navMode.value,
    taskId: selectedTaskId.value,
    documentId: selectedDocId.value,
    pane: paneFocus.value
  }))

  function selectTask(id: TaskId): void {
    selectedTaskId.value = id
    const first = documents.value.find((document) => document.tasks.some((ref) => ref.id === id))
    selectedDocId.value = first?.id ?? null
    paneFocus.value = 'note'
  }

  /**
   * Browsing tasks, a note has to bring one of its tasks along or the task panes disagree with
   * the editor. Browsing notes, the panes are the note's own, so the task in view is left alone:
   * that is what makes the Notes side a place to manage notes rather than a detour through tasks.
   */
  function selectDocument(id: DocumentId): void {
    selectedDocId.value = id
    paneFocus.value = 'note'
    noteComposerOpen.value = false
    if (navMode.value === 'notes') return
    followDocumentToTask(id)
  }

  function followDocumentToTask(id: DocumentId): void {
    const document = documents.value.find((candidate) => candidate.id === id)
    if (document && !document.tasks.some((ref) => ref.id === selectedTaskId.value)) {
      selectedTaskId.value = document.tasks[0]?.id ?? null
    }
  }

  function setNavMode(mode: NavMode): void {
    navMode.value = mode
    noteComposerOpen.value = false
    if (mode === 'notes') {
      if (selectedDocId.value !== null) paneFocus.value = 'note'
      return
    }
    if (selectedDocId.value !== null) followDocumentToTask(selectedDocId.value)
  }

  /**
   * Puts the console back where a history entry left it. A task or note deleted since is dropped
   * and a task pane without its task falls back to the note; returns what is actually shown.
   */
  function showLocation(target: ConsoleLocation): ConsoleLocation {
    const taskId = tasks.value.some((task) => task.id === target.taskId) ? target.taskId : null
    const documentId = documents.value.some((document) => document.id === target.documentId)
      ? target.documentId
      : null
    navMode.value = target.mode
    selectedTaskId.value = taskId
    selectedDocId.value = documentId
    paneFocus.value = taskId === null ? 'note' : target.pane
    noteComposerOpen.value = false
    return location.value
  }

  function setScope(company: CompanyId | null, project: ProjectId | null = null): void {
    scopeCompany.value = company
    scopeProject.value = project
    if (selected.selectedTask.value && !view.inScope(selected.selectedTask.value)) {
      const first = view.visibleTasks.value[0]
      if (first) selectTask(first.id)
      else {
        selectedTaskId.value = null
        selectedDocId.value = null
      }
    }
  }

  /** A task pane opens only over a task; with none in view the note stays. */
  function openPane(pane: TaskPane): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = pane
  }

  function togglePane(pane: TaskPane): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = paneFocus.value === pane ? 'note' : pane
  }

  /** Brings a task into view on its steps pane, with one step open. */
  function openStep(taskId: TaskId, stepId: TaskStepId): void {
    if (selectedTaskId.value !== taskId) selectTask(taskId)
    stepToOpen.value = stepId
    paneFocus.value = 'steps'
  }

  function clearStepToOpen(): void {
    stepToOpen.value = null
  }

  /** Opens a review item where it is reviewed: the step, or the description's review bar. */
  function openReviewItem(item: ReviewItem): void {
    if (item.stepId) {
      openStep(item.taskId, item.stepId)
      return
    }
    selectTask(item.taskId)
    paneFocus.value = 'description'
  }

  /** Opens whatever a search hit points at, on the pane that shows that text. */
  function openSearchHit(hit: SearchHit): void {
    if (hit.kind === 'NOTE' && hit.documentId) {
      if (navMode.value === 'tasks' && hit.taskId) selectTask(hit.taskId)
      selectDocument(hit.documentId)
      return
    }
    if (!hit.taskId) return
    if (hit.kind === 'STEP' && hit.stepId) {
      openStep(hit.taskId, hit.stepId)
      return
    }
    selectTask(hit.taskId)
    paneFocus.value = hit.kind === 'WRAPUP' ? 'wrapup' : 'description'
  }

  function openNoteComposer(): void {
    noteComposerOpen.value = true
  }

  function closeNoteComposer(): void {
    noteComposerOpen.value = false
  }

  return {
    location,
    selectTask,
    selectDocument,
    setNavMode,
    showLocation,
    setScope,
    openWrapup: () => openPane('wrapup'),
    openDescription: () => openPane('description'),
    openSteps: () => openPane('steps'),
    openTerminal: () => openPane('terminal'),
    toggleDescription: () => togglePane('description'),
    toggleWrapup: () => togglePane('wrapup'),
    toggleSteps: () => togglePane('steps'),
    toggleTerminal: () => togglePane('terminal'),
    openStep,
    clearStepToOpen,
    openReviewItem,
    openSearchHit,
    openNoteComposer,
    closeNoteComposer
  }
}
