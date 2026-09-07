import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import {
  createCompany as apiCreateCompany,
  createProject as apiCreateProject,
  createTask as apiCreateTask,
  deleteCompany as apiDeleteCompany,
  deleteProject as apiDeleteProject,
  deleteTask as apiDeleteTask,
  fetchCompanies,
  fetchProjects,
  fetchTasks,
  updateCompany as apiUpdateCompany,
  updateProject as apiUpdateProject,
  updateTask as apiUpdateTask
} from '@/api/catalog.api'
import type { CompanyInput, ProjectInput, TaskInput } from '@/api/catalog.api'
import {
  createDocument as apiCreateDocument,
  deleteDocument as apiDeleteDocument,
  fetchAllDocuments,
  updateDocument as apiUpdateDocument
} from '@/api/documents.api'
import {
  deleteWrapup as apiDeleteWrapup,
  fetchWrapups,
  saveWrapup as apiSaveWrapup
} from '@/api/wrapups.api'
import {
  createStep as apiCreateStep,
  deleteStep as apiDeleteStep,
  fetchSteps,
  moveStep as apiMoveStep,
  patchStep as apiPatchStep
} from '@/api/steps.api'
import type { TaskStepPatch } from '@/api/steps.api'
import {
  deleteTimeEntry as apiDeleteTimeEntry,
  editTimeEntry as apiEditTimeEntry,
  fetchTimeEntries,
  startTimeEntry as apiStartTimeEntry,
  stopTimeEntry as apiStopTimeEntry
} from '@/api/time-entries.api'
import type { TimeEntryEdit } from '@/api/time-entries.api'
import {
  stepCompletedAt,
  stepIsComplete,
  type Company,
  type Project,
  type RekallDocument,
  type Task,
  type TaskStatus,
  type TaskStep,
  type TimeEntry,
  type Wrapup
} from '@/model/catalog'
import type {
  CompanyId,
  DocumentId,
  ProjectId,
  TaskId,
  TaskStepId,
  TimeEntryId
} from '@/model/branded'

export type NavMode = 'tasks' | 'notes'
export type SaveState = 'saved' | 'unsaved' | 'saving'

export type PaneFocus = 'note' | 'wrapup' | 'description' | 'steps'

export const useConsoleStore = defineStore('console', () => {
  const companies = ref<Company[]>([])
  const projects = ref<Project[]>([])
  const tasks = ref<Task[]>([])
  const documents = ref<RekallDocument[]>([])
  const wrapups = ref<Wrapup[]>([])
  const steps = ref<TaskStep[]>([])
  const timeEntries = ref<TimeEntry[]>([])

  const isLoading = ref(true)
  const saveState = ref<SaveState>('saved')

  const scopeCompany = ref<CompanyId | null>(null)
  const scopeProject = ref<ProjectId | null>(null)
  const navMode = ref<NavMode>('tasks')
  const filter = ref('')
  const selectedTaskId = ref<TaskId | null>(null)
  const selectedDocId = ref<DocumentId | null>(null)
  const paneFocus = ref<PaneFocus>('note')

  const projectInScope = (project: Project): boolean =>
    (scopeCompany.value === null || project.companyId === scopeCompany.value) &&
    (scopeProject.value === null || project.id === scopeProject.value)

  const inScope = (task: Task): boolean => {
    if (scopeProject.value !== null) return task.projectId === scopeProject.value
    if (scopeCompany.value === null) return true
    return projects.value.find((p) => p.id === task.projectId)?.companyId === scopeCompany.value
  }

  function matchesTask(task: Task, needle: string): boolean {
    if (!needle.trim()) return true
    const hay =
      `${task.projectLabel} ${task.projectTitle} ${task.label} ${task.title}`.toLowerCase()
    return needle
      .toLowerCase()
      .replace(/(project:|task:|company:)/g, ' ')
      .replace(/\//g, ' ')
      .trim()
      .split(/\s+/)
      .every((part) => hay.includes(part))
  }

  function matchesDocument(document: RekallDocument, needle: string): boolean {
    if (!needle.trim()) return true
    const hay = `${document.title} ${document.kind} ${document.bodyMarkdown}`.toLowerCase()
    return needle.toLowerCase().trim().split(/\s+/).every((part) => hay.includes(part))
  }

  const documentInScope = (document: RekallDocument): boolean =>
    (scopeCompany.value === null && scopeProject.value === null) ||
    document.tasks.some((ref) => {
      const task = tasks.value.find((t) => t.id === ref.id)
      return task !== undefined && inScope(task)
    })

  const visibleTasks = computed(() =>
    tasks.value.filter((task) => inScope(task) && matchesTask(task, filter.value))
  )

  const visibleDocuments = computed(() =>
    documents.value.filter(
      (document) => documentInScope(document) && matchesDocument(document, filter.value)
    )
  )

  const selectedTask = computed(
    () => tasks.value.find((task) => task.id === selectedTaskId.value) ?? null
  )

  const selectedDocument = computed(
    () => documents.value.find((document) => document.id === selectedDocId.value) ?? null
  )

  const scopedCompany = computed(
    () => companies.value.find((company) => company.id === scopeCompany.value) ?? null
  )

  const scopedProject = computed(
    () => projects.value.find((project) => project.id === scopeProject.value) ?? null
  )

  const taskDocuments = computed(() =>
    selectedTaskId.value === null
      ? []
      : documents.value.filter((document) =>
          document.tasks.some((ref) => ref.id === selectedTaskId.value)
        )
  )

  const selectedTaskSteps = computed(() =>
    steps.value
      .filter((step) => step.taskId === selectedTaskId.value)
      .sort((a, b) => a.position - b.position)
  )

  const openStepCount = computed(() => selectedTaskSteps.value.filter((step) => !step.done).length)

  const selectedWrapup = computed(
    () => wrapups.value.find((wrapup) => wrapup.taskId === selectedTaskId.value) ?? null
  )

  const wrapupMissesSteps = computed(() => {
    const wrapup = selectedWrapup.value
    if (!wrapup) return 0
    return selectedTaskSteps.value.filter((step) => {
      if (!stepIsComplete(step.state)) return false
      const at = stepCompletedAt(step)
      return at !== null && at > wrapup.updatedAt
    }).length
  })

  const runningStep = computed(
    () => selectedTaskSteps.value.find((step) => step.state === 'RUNNING') ?? null
  )

  const wrapupIsBehind = computed(() => {
    const wrapup = selectedWrapup.value
    if (!wrapup) return 0
    return taskDocuments.value.filter((document) => document.updatedAt > wrapup.updatedAt).length
  })

  const runningEntries = computed(() => timeEntries.value.filter((entry) => entry.stoppedAt === null))

  const selectedTaskEntries = computed(() =>
    timeEntries.value
      .filter((entry) => entry.taskId === selectedTaskId.value)
      .sort((a, b) => b.startedAt.localeCompare(a.startedAt))
  )

  const recentDocuments = computed(() =>
    [...documents.value]
      .sort((a, b) => b.updatedAt.localeCompare(a.updatedAt))
      .slice(0, 4)
  )

  const scopedProjects = computed(() => projects.value.filter(projectInScope))

  const scopePath = computed<string[]>(() => {
    if (!scopedCompany.value) return []
    return scopedProject.value
      ? [scopedCompany.value.name, scopedProject.value.title]
      : [scopedCompany.value.name]
  })

  const scopeName = computed(() =>
    scopePath.value.length === 0 ? 'All work' : scopePath.value.join(' / ')
  )

  const scopeAnchor = computed(() => {
    if (scopedProject.value) return scopedProject.value.anchor
    return scopedCompany.value ? `company:${scopedCompany.value.name}` : ''
  })

  const elsewhere = computed(() => {
    if ((scopeCompany.value === null && scopeProject.value === null) || !filter.value.trim())
      return null
    const outTasks = tasks.value.filter((t) => !inScope(t) && matchesTask(t, filter.value))
    const outDocs = documents.value.filter(
      (d) => !documentInScope(d) && matchesDocument(d, filter.value)
    )
    const count = navMode.value === 'tasks' ? outTasks.length : outDocs.length
    if (count === 0) return null
    const names = [
      ...new Set(
        navMode.value === 'tasks'
          ? outTasks.map((t) => (scopeProject.value ? t.projectTitle : t.companyName))
          : outDocs.flatMap((d) =>
              d.tasks.map((ref) => (scopeProject.value ? ref.projectTitle : ref.companyName))
            )
      )
    ]
    return { count, names }
  })

  async function load(): Promise<void> {
    isLoading.value = true
    try {
      const [
        loadedCompanies,
        loadedProjects,
        loadedTasks,
        loadedDocuments,
        loadedWrapups,
        loadedSteps,
        loadedTimeEntries
      ] = await Promise.all([
        fetchCompanies(),
        fetchProjects(),
        fetchTasks(),
        fetchAllDocuments(),
        fetchWrapups(),
        fetchSteps(),
        fetchTimeEntries()
      ])
      companies.value = loadedCompanies
      projects.value = loadedProjects
      tasks.value = loadedTasks
      documents.value = loadedDocuments
      wrapups.value = loadedWrapups
      steps.value = loadedSteps
      timeEntries.value = loadedTimeEntries
    } finally {
      isLoading.value = false
    }
  }

  function selectTask(id: TaskId): void {
    selectedTaskId.value = id
    const first = documents.value.find((document) =>
      document.tasks.some((ref) => ref.id === id)
    )
    selectedDocId.value = first?.id ?? null
    paneFocus.value = 'note'
  }

  function selectDocument(id: DocumentId): void {
    selectedDocId.value = id
    paneFocus.value = 'note'
    const document = documents.value.find((candidate) => candidate.id === id)
    if (document && !document.tasks.some((ref) => ref.id === selectedTaskId.value)) {
      selectedTaskId.value = document.tasks[0]?.id ?? null
    }
  }

  function setScope(company: CompanyId | null, project: ProjectId | null = null): void {
    scopeCompany.value = company
    scopeProject.value = project
    if (selectedTask.value && !inScope(selectedTask.value)) {
      const first = visibleTasks.value[0]
      if (first) selectTask(first.id)
      else {
        selectedTaskId.value = null
        selectedDocId.value = null
      }
    }
  }

  async function createCompany(input: CompanyInput): Promise<void> {
    const created = await apiCreateCompany(input)
    companies.value = [...companies.value, created].sort((a, b) => a.name.localeCompare(b.name))
    setScope(created.id)
  }

  async function updateCompany(id: CompanyId, input: CompanyInput): Promise<void> {
    const saved = await apiUpdateCompany(id, input)
    companies.value = companies.value
      .map((company) => (company.id === id ? saved : company))
      .sort((a, b) => a.name.localeCompare(b.name))
    await Promise.all([refreshProjects(), refreshTasks()])
  }

  async function deleteCompany(id: CompanyId): Promise<void> {
    await apiDeleteCompany(id)
    if (scopeCompany.value === id) setScope(null)
    await load()
  }

  async function createProject(input: ProjectInput): Promise<Project> {
    const created = await apiCreateProject(input)
    projects.value = [...projects.value, created].sort((a, b) => a.label.localeCompare(b.label))
    await refreshCompanies()
    setScope(input.companyId, created.id)
    return created
  }

  async function updateProject(id: ProjectId, input: ProjectInput): Promise<void> {
    const saved = await apiUpdateProject(id, input)
    projects.value = projects.value
      .map((project) => (project.id === id ? saved : project))
      .sort((a, b) => a.label.localeCompare(b.label))
    await Promise.all([refreshTasks(), refreshCompanies(), refreshDocuments(), refreshWrapups()])
  }

  async function deleteProject(id: ProjectId): Promise<void> {
    await apiDeleteProject(id)
    if (scopeProject.value === id) setScope(scopeCompany.value)
    await load()
  }

  async function patchProject(
    id: ProjectId,
    patch: Partial<Pick<ProjectInput, 'description' | 'blueprintMarkdown' | 'repoFolder'>>
  ): Promise<void> {
    const current = projects.value.find((project) => project.id === id)
    if (!current) return
    saveState.value = 'saving'
    try {
      const saved = await apiUpdateProject(id, {
        label: current.label,
        title: current.title,
        status: current.status,
        companyId: current.companyId,
        description: 'description' in patch ? patch.description! : current.description,
        blueprintMarkdown:
          'blueprintMarkdown' in patch ? patch.blueprintMarkdown! : current.blueprintMarkdown,
        repoFolder: 'repoFolder' in patch ? patch.repoFolder! : current.repoFolder
      })
      projects.value = projects.value.map((project) => (project.id === id ? saved : project))
      tasks.value = tasks.value.map((task) =>
        task.projectId === id ? { ...task, projectRepoFolder: saved.repoFolder } : task
      )
      saveState.value = 'saved'
    } catch (error) {
      saveState.value = 'unsaved'
      throw error
    }
  }

  function saveProjectDescription(id: ProjectId, description: string): Promise<void> {
    return patchProject(id, { description: description.trim() === '' ? null : description })
  }

  function saveProjectBlueprint(id: ProjectId, blueprintMarkdown: string): Promise<void> {
    return patchProject(id, {
      blueprintMarkdown: blueprintMarkdown.trim() === '' ? null : blueprintMarkdown
    })
  }

  function saveProjectRepoFolder(id: ProjectId, repoFolder: string): Promise<void> {
    return patchProject(id, { repoFolder: repoFolder.trim() === '' ? null : repoFolder.trim() })
  }

  async function createTask(input: TaskInput): Promise<Task> {
    const created = await apiCreateTask(input)
    tasks.value = [...tasks.value, created]
    await Promise.all([refreshProjects(), refreshCompanies()])
    selectTask(created.id)
    await startTimer(created.id)
    return created
  }

  async function updateTask(id: TaskId, input: TaskInput): Promise<void> {
    const saved = await apiUpdateTask(id, input)
    tasks.value = tasks.value.map((task) => (task.id === id ? saved : task))
    await Promise.all([
      refreshDocuments(),
      refreshProjects(),
      refreshCompanies(),
      refreshWrapups()
    ])
  }

  async function deleteTask(id: TaskId): Promise<void> {
    await apiDeleteTask(id)
    tasks.value = tasks.value.filter((task) => task.id !== id)
    wrapups.value = wrapups.value.filter((wrapup) => wrapup.taskId !== id)
    steps.value = steps.value.filter((step) => step.taskId !== id)
    if (selectedTaskId.value === id) {
      selectedTaskId.value = null
      selectedDocId.value = null
      paneFocus.value = 'note'
    }
    await Promise.all([refreshDocuments(), refreshProjects(), refreshCompanies()])
  }

  async function setTaskStatus(id: TaskId, status: TaskStatus): Promise<void> {
    const task = tasks.value.find((candidate) => candidate.id === id)
    if (!task || task.status === status) return
    const saved = await apiUpdateTask(id, {
      label: task.label,
      title: task.title,
      status,
      description: task.description,
      projectId: task.projectId
    })
    tasks.value = tasks.value.map((candidate) => (candidate.id === id ? saved : candidate))
  }

  async function saveTaskDescription(id: TaskId, description: string): Promise<void> {
    const task = tasks.value.find((candidate) => candidate.id === id)
    if (!task) return
    const next = description.trim() === '' ? null : description
    if (task.description === next) return
    saveState.value = 'saving'
    try {
      const saved = await apiUpdateTask(id, {
        label: task.label,
        title: task.title,
        status: task.status,
        description: next,
        projectId: task.projectId
      })
      tasks.value = tasks.value.map((candidate) => (candidate.id === id ? saved : candidate))
      saveState.value = 'saved'
    } catch (error) {
      saveState.value = 'unsaved'
      throw error
    }
  }

  async function createNote(taskId: TaskId): Promise<void> {
    const created = await apiCreateDocument({
      title: 'untitled.md',
      kind: 'notes',
      bodyMarkdown: '',
      taskIds: [taskId]
    })
    documents.value = [created, ...documents.value]
    selectedDocId.value = created.id
    await refreshTasks()
  }

  async function saveNote(
    id: DocumentId,
    patch: Partial<Pick<RekallDocument, 'title' | 'kind' | 'bodyMarkdown'>> & {
      taskIds?: readonly TaskId[]
    }
  ): Promise<void> {
    const current = documents.value.find((document) => document.id === id)
    if (!current) return
    saveState.value = 'saving'
    try {
      const saved = await apiUpdateDocument(id, {
        title: patch.title ?? current.title,
        kind: patch.kind ?? current.kind,
        bodyMarkdown: patch.bodyMarkdown ?? current.bodyMarkdown,
        taskIds: patch.taskIds ?? current.tasks.map((ref) => ref.id)
      })
      documents.value = documents.value.map((document) => (document.id === id ? saved : document))
      saveState.value = 'saved'
      if (patch.taskIds) await refreshTasks()
    } catch (error) {
      saveState.value = 'unsaved'
      throw error
    }
  }

  async function deleteNote(id: DocumentId): Promise<void> {
    await apiDeleteDocument(id)
    documents.value = documents.value.filter((document) => document.id !== id)
    if (selectedDocId.value === id) {
      selectedDocId.value = taskDocuments.value[0]?.id ?? null
    }
    await refreshTasks()
  }

  function openWrapup(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = 'wrapup'
  }

  function openDescription(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = 'description'
  }

  function toggleDescription(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = paneFocus.value === 'description' ? 'note' : 'description'
  }

  function toggleWrapup(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = paneFocus.value === 'wrapup' ? 'note' : 'wrapup'
  }

  function toggleSteps(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = paneFocus.value === 'steps' ? 'note' : 'steps'
  }

  function openSteps(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = 'steps'
  }

  async function saveWrapupBody(taskId: TaskId, bodyMarkdown: string): Promise<void> {
    saveState.value = 'saving'
    try {
      const saved = await apiSaveWrapup(taskId, bodyMarkdown)
      const known = wrapups.value.some((wrapup) => wrapup.id === saved.id)
      wrapups.value = known
        ? wrapups.value.map((wrapup) => (wrapup.id === saved.id ? saved : wrapup))
        : [saved, ...wrapups.value]
      saveState.value = 'saved'
      if (!known) await refreshTasks()
    } catch (error) {
      saveState.value = 'unsaved'
      throw error
    }
  }

  async function removeWrapup(taskId: TaskId): Promise<void> {
    await apiDeleteWrapup(taskId)
    wrapups.value = wrapups.value.filter((wrapup) => wrapup.taskId !== taskId)
    paneFocus.value = 'note'
    await refreshTasks()
  }

  function recountSteps(taskId: TaskId): void {
    const own = steps.value.filter((step) => step.taskId === taskId)
    tasks.value = tasks.value.map((task) =>
      task.id === taskId
        ? { ...task, stepCount: own.length, stepsDone: own.filter((step) => step.done).length }
        : task
    )
  }

  function upsertStep(step: TaskStep): void {
    const known = steps.value.some((candidate) => candidate.id === step.id)
    steps.value = known
      ? steps.value.map((candidate) => (candidate.id === step.id ? step : candidate))
      : [...steps.value, step]
    recountSteps(step.taskId)
  }

  function applyStepEvent(taskId: TaskId, incoming: TaskStep[]): void {
    steps.value = [...steps.value.filter((step) => step.taskId !== taskId), ...incoming]
    recountSteps(taskId)
  }

  async function addStep(taskId: TaskId, title: string, bodyMarkdown?: string): Promise<TaskStep> {
    const created = await apiCreateStep(taskId, title, bodyMarkdown)
    upsertStep(created)
    return created
  }

  async function saveStep(id: TaskStepId, patch: TaskStepPatch): Promise<void> {
    saveState.value = 'saving'
    try {
      upsertStep(await apiPatchStep(id, patch))
      saveState.value = 'saved'
    } catch (error) {
      saveState.value = 'unsaved'
      throw error
    }
  }

  function toggleStep(id: TaskStepId): Promise<void> {
    const step = steps.value.find((candidate) => candidate.id === id)
    if (!step) return Promise.resolve()
    return saveStep(id, { done: !step.done })
  }

  async function moveStep(id: TaskStepId, position: number): Promise<void> {
    const step = steps.value.find((candidate) => candidate.id === id)
    if (!step) return
    const reordered = await apiMoveStep(id, position)
    steps.value = [
      ...steps.value.filter((candidate) => candidate.taskId !== step.taskId),
      ...reordered
    ]
  }

  async function removeStep(id: TaskStepId): Promise<void> {
    const step = steps.value.find((candidate) => candidate.id === id)
    if (!step) return
    await apiDeleteStep(id)
    steps.value = steps.value.filter((candidate) => candidate.id !== id)
    recountSteps(step.taskId)
  }

  function upsertTimeEntry(entry: TimeEntry): void {
    const known = timeEntries.value.some((candidate) => candidate.id === entry.id)
    timeEntries.value = known
      ? timeEntries.value.map((candidate) => (candidate.id === entry.id ? entry : candidate))
      : [entry, ...timeEntries.value]
  }

  async function startTimer(taskId: TaskId): Promise<void> {
    upsertTimeEntry(await apiStartTimeEntry(taskId))
  }

  async function pauseTimer(taskId: TaskId): Promise<void> {
    const stopped = await apiStopTimeEntry(taskId)
    upsertTimeEntry(stopped)
  }

  async function editTimer(id: TimeEntryId, input: TimeEntryEdit): Promise<void> {
    const saved = await apiEditTimeEntry(id, input)
    upsertTimeEntry(saved)
  }

  async function deleteTimer(id: TimeEntryId): Promise<void> {
    await apiDeleteTimeEntry(id)
    timeEntries.value = timeEntries.value.filter((entry) => entry.id !== id)
  }

  async function refreshTasks(): Promise<void> {
    tasks.value = await fetchTasks()
  }

  async function refreshProjects(): Promise<void> {
    projects.value = await fetchProjects()
  }

  async function refreshCompanies(): Promise<void> {
    companies.value = await fetchCompanies()
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

  async function refreshEverything(): Promise<void> {
    await Promise.all([
      refreshCompanies(),
      refreshProjects(),
      refreshTasks(),
      refreshDocuments(),
      refreshWrapups(),
      refreshSteps(),
      refreshTimeEntries()
    ])
  }

  return {
    companies,
    projects,
    scopedProjects,
    scopedCompany,
    scopedProject,
    scopeCompany,
    scopeProject,
    scopePath,
    scopeAnchor,
    createCompany,
    updateCompany,
    deleteCompany,
    tasks,
    documents,
    wrapups,
    steps,
    timeEntries,
    runningEntries,
    selectedTaskEntries,
    isLoading,
    saveState,
    scopeName,
    navMode,
    filter,
    selectedTaskId,
    selectedDocId,
    paneFocus,
    selectedTask,
    selectedDocument,
    selectedWrapup,
    selectedTaskSteps,
    openStepCount,
    runningStep,
    wrapupIsBehind,
    wrapupMissesSteps,
    visibleTasks,
    visibleDocuments,
    taskDocuments,
    recentDocuments,
    elsewhere,
    load,
    selectTask,
    selectDocument,
    setScope,
    createProject,
    updateProject,
    deleteProject,
    saveProjectDescription,
    saveProjectBlueprint,
    saveProjectRepoFolder,
    refreshEverything,
    createTask,
    updateTask,
    deleteTask,
    setTaskStatus,
    saveTaskDescription,
    createNote,
    saveNote,
    deleteNote,
    openWrapup,
    openDescription,
    openSteps,
    toggleDescription,
    toggleWrapup,
    toggleSteps,
    addStep,
    saveStep,
    toggleStep,
    moveStep,
    removeStep,
    applyStepEvent,
    saveWrapupBody,
    removeWrapup,
    startTimer,
    pauseTimer,
    editTimer,
    deleteTimer
  }
})
