import { defineStore } from 'pinia'
import { createConsoleState } from './console/console-state'
import { useCommitReferences } from '../features/commits/store/useCommitReferences'
import { useCompanyWrites } from '../features/catalog/store/useCompanyWrites'
import { useConsoleNavigation } from './console/useConsoleNavigation'
import { useConsoleRefreshes } from './console/useConsoleRefreshes'
import { useConsoleView } from './console/useConsoleView'
import { useJiraImport } from '../features/connection/store/useJiraImport'
import { useNoteWrites } from '../features/notes/store/useNoteWrites'
import { useProjectWrites } from '../features/catalog/store/useProjectWrites'
import { useSelectedTask } from './console/useSelectedTask'
import { useSkillAssignments } from '../features/skills/store/useSkillAssignments'
import { useStepFlowWrites } from '../features/flow/store/useStepFlowWrites'
import { useStepWrites } from '../features/steps/store/useStepWrites'
import { useTagWrites } from '../features/tags/store/useTagWrites'
import { useTaskWrites } from '../features/catalog/store/useTaskWrites'
import { useTimers } from '../features/time/store/useTimers'
import { useWrapupWrites } from '../features/wrapup/store/useWrapupWrites'

export type { SaveState } from './console/console-state'
export type { NoteDraft } from '../features/notes/store/useNoteWrites'

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
  const stepFlow = useStepFlowWrites(steps)
  const wrapups = useWrapupWrites(state, refreshes, timers)
  const commitReferences = useCommitReferences(state)
  const skills = useSkillAssignments(state)
  const jiraImport = useJiraImport(state, refreshes)

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
    ...stepFlow,
    ...wrapups,
    ...commitReferences,
    ...skills,
    ...jiraImport
  }
})
