import { fetchCompanies, fetchProjects, fetchTags, fetchTasks } from '@/api/catalog.api'
import { fetchCommitReferences } from '@/api/commitReference.api'
import { fetchAllDocuments } from '@/api/documents.api'
import { fetchSkillAssignments } from '@/api/skill-assignments.api'
import { fetchSteps } from '@/api/steps.api'
import { fetchTimeEntries } from '@/api/time-entries.api'
import { fetchWrapups } from '@/api/wrapups.api'
import type { ConsoleState } from './console-state'

/** Reads every kind of row again from the server, one kind at a time or all at once. */
export function useConsoleRefreshes(state: ConsoleState) {
  const {
    companies,
    projects,
    tasks,
    tags,
    documents,
    wrapups,
    steps,
    timeEntries,
    commitReferences,
    taskSkills,
    tagSkills
  } = state

  async function refreshCompanies(): Promise<void> {
    companies.value = await fetchCompanies()
  }

  async function refreshProjects(): Promise<void> {
    projects.value = await fetchProjects()
  }

  async function refreshTasks(): Promise<void> {
    tasks.value = await fetchTasks()
  }

  async function refreshTags(): Promise<void> {
    tags.value = await fetchTags()
  }

  async function refreshDocuments(): Promise<void> {
    documents.value = await fetchAllDocuments()
  }

  async function refreshWrapups(): Promise<void> {
    wrapups.value = await fetchWrapups()
  }

  async function refreshSteps(): Promise<void> {
    steps.value = await fetchSteps()
  }

  async function refreshTimeEntries(): Promise<void> {
    timeEntries.value = await fetchTimeEntries()
  }

  async function refreshCommitReferences(): Promise<void> {
    commitReferences.value = await fetchCommitReferences()
  }

  async function refreshSkillAssignments(): Promise<void> {
    const assignments = await fetchSkillAssignments()
    taskSkills.value = assignments.taskSkills
    tagSkills.value = assignments.tagSkills
  }

  async function refreshEverything(): Promise<void> {
    await Promise.all([
      refreshCompanies(),
      refreshProjects(),
      refreshTasks(),
      refreshTags(),
      refreshDocuments(),
      refreshWrapups(),
      refreshSteps(),
      refreshTimeEntries(),
      refreshCommitReferences(),
      refreshSkillAssignments()
    ])
  }

  /** The first read, which the console waits on. All or nothing: a failed read leaves every list as it was. */
  async function load(): Promise<void> {
    state.isLoading.value = true
    try {
      const [
        loadedCompanies,
        loadedProjects,
        loadedTasks,
        loadedTags,
        loadedDocuments,
        loadedWrapups,
        loadedSteps,
        loadedTimeEntries,
        loadedCommitReferences,
        loadedSkillAssignments
      ] = await Promise.all([
        fetchCompanies(),
        fetchProjects(),
        fetchTasks(),
        fetchTags(),
        fetchAllDocuments(),
        fetchWrapups(),
        fetchSteps(),
        fetchTimeEntries(),
        fetchCommitReferences(),
        fetchSkillAssignments()
      ])
      companies.value = loadedCompanies
      projects.value = loadedProjects
      tasks.value = loadedTasks
      tags.value = loadedTags
      documents.value = loadedDocuments
      wrapups.value = loadedWrapups
      steps.value = loadedSteps
      timeEntries.value = loadedTimeEntries
      commitReferences.value = loadedCommitReferences
      taskSkills.value = loadedSkillAssignments.taskSkills
      tagSkills.value = loadedSkillAssignments.tagSkills
    } finally {
      state.isLoading.value = false
    }
  }

  return {
    load,
    refreshEverything,
    refreshCompanies,
    refreshProjects,
    refreshTasks,
    refreshTags,
    refreshDocuments,
    refreshWrapups,
    refreshTimeEntries,
    refreshCommitReferences,
    refreshSkillAssignments
  }
}
