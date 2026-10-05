import { defineStore } from 'pinia'
import { createConsoleState } from './console/console-state'
import { useCommitReferences } from './console/useCommitReferences'
import { useCompanyWrites } from './console/useCompanyWrites'
import { useConsoleNavigation } from './console/useConsoleNavigation'
import { useConsoleRefreshes } from './console/useConsoleRefreshes'
import { useConsoleView } from './console/useConsoleView'
import { useNoteWrites } from './console/useNoteWrites'
import { useProjectWrites } from './console/useProjectWrites'
import { useSelectedTask } from './console/useSelectedTask'
import { useStepWrites } from './console/useStepWrites'
import { useTagWrites } from './console/useTagWrites'
import { useTaskWrites } from './console/useTaskWrites'
import { useTimers } from './console/useTimers'
import { useWrapupWrites } from './console/useWrapupWrites'

export type { SaveState } from './console/console-state'
export type { NoteDraft } from './console/useNoteWrites'

/**
 * The console's one store: every row the console shows and where the reader is in them. Each
 * concern lives in its own slice under `console/`, built over the shared state; this wires them
 * and states the public surface.
 */
export const useConsoleStore = defineStore('console', () => {
  const state = createConsoleState()
  const refreshes = useConsoleRefreshes(state)
  const view = useConsoleView(state)
  const selected = useSelectedTask(state)
  const navigation = useConsoleNavigation(state, view, selected)
  const timers = useTimers(state, refreshes)
  const companies = useCompanyWrites(state, navigation, refreshes)
  const projects = useProjectWrites(state, navigation, refreshes)
  const tasks = useTaskWrites(state, navigation, refreshes, timers)
  const tags = useTagWrites(state, refreshes)
  const notes = useNoteWrites(state, selected, refreshes, timers)
  const steps = useStepWrites(state, timers)
  const wrapups = useWrapupWrites(state, refreshes, timers)
  const commitReferences = useCommitReferences(state)

  return {
    ...state,
    scopedProjects: view.scopedProjects,
    scopedCompany: view.scopedCompany,
    scopedProject: view.scopedProject,
    scopePath: view.scopePath,
    scopeAnchor: view.scopeAnchor,
    scopeName: view.scopeName,
    visibleTasks: view.visibleTasks,
    visibleDocuments: view.visibleDocuments,
    recentDocuments: view.recentDocuments,
    elsewhere: view.elsewhere,
    ...selected,
    ...navigation,
    load: refreshes.load,
    refreshEverything: refreshes.refreshEverything,
    refreshTags: refreshes.refreshTags,
    refreshCommitReferences: refreshes.refreshCommitReferences,
    runningEntries: timers.runningEntries,
    startTimer: timers.startTimer,
    pauseTimer: timers.pauseTimer,
    editTimer: timers.editTimer,
    deleteTimer: timers.deleteTimer,
    ...companies,
    ...projects,
    ...tasks,
    ...tags,
    ...notes,
    ...steps,
    ...wrapups,
    ...commitReferences
  }
})
