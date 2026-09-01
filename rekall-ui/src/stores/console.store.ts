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
  deleteTimeEntry as apiDeleteTimeEntry,
  editTimeEntry as apiEditTimeEntry,
  fetchTimeEntries,
  startTimeEntry as apiStartTimeEntry,
  stopTimeEntry as apiStopTimeEntry
} from '@/api/time-entries.api'
import type { TimeEntryEdit } from '@/api/time-entries.api'
import type {
  Company,
  Project,
  RekallDocument,
  Task,
  TaskStatus,
  TimeEntry,
  Wrapup
} from '@/model/catalog'
import type { CompanyId, DocumentId, ProjectId, TaskId, TimeEntryId } from '@/model/branded'

export type NavMode = 'tasks' | 'notes'
export type SaveState = 'saved' | 'unsaved' | 'saving'

/**
 * What the right-hand pane is showing.
 *
 * Three states rather than a wrapup pretending to be a note: they are edited differently, and
 * two of them have no title, no kind and no other task they could belong to. The description
 * and the wrapup are separate for the same reason they are separate columns: one is the brief
 * the work is measured against, the other is where the work got to.
 */
export type PaneFocus = 'note' | 'wrapup' | 'description'

/** Everything the console shows, loaded once and kept in step by the actions below. */
export const useConsoleStore = defineStore('console', () => {
  const companies = ref<Company[]>([])
  const projects = ref<Project[]>([])
  const tasks = ref<Task[]>([])
  const documents = ref<RekallDocument[]>([])
  const wrapups = ref<Wrapup[]>([])
  const timeEntries = ref<TimeEntry[]>([])

  const isLoading = ref(true)
  const saveState = ref<SaveState>('saved')

  /**
   * How far in you are looking: everything, one company, or one project inside it.
   *
   * Two nullable ids rather than a mode plus an id, because the pair is the state: a project
   * scope implies its company, and there is no third combination to represent.
   */
  const scopeCompany = ref<CompanyId | null>(null)
  const scopeProject = ref<ProjectId | null>(null)
  const navMode = ref<NavMode>('tasks')
  const filter = ref('')
  const selectedTaskId = ref<TaskId | null>(null)
  const selectedDocId = ref<DocumentId | null>(null)
  const paneFocus = ref<PaneFocus>('note')

  // ------------------------------------------------------------------ reading

  const projectInScope = (project: Project): boolean =>
    (scopeCompany.value === null || project.companyId === scopeCompany.value) &&
    (scopeProject.value === null || project.id === scopeProject.value)

  const inScope = (task: Task): boolean => {
    if (scopeProject.value !== null) return task.projectId === scopeProject.value
    if (scopeCompany.value === null) return true
    return projects.value.find((p) => p.id === task.projectId)?.companyId === scopeCompany.value
  }

  /**
   * Both names are searched, because either is what you remember.
   *
   * You reach for a task by the anchor you have typed a hundred times or by the sentence you
   * called it in a meeting, and which one surfaces first is not something to make anyone think
   * about.
   */
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

  /** The notes on the selected task, which is what the middle pane lists. */
  const taskDocuments = computed(() =>
    selectedTaskId.value === null
      ? []
      : documents.value.filter((document) =>
          document.tasks.some((ref) => ref.id === selectedTaskId.value)
        )
  )

  /** The wrapup of the task in view, or null while nobody has written one. */
  const selectedWrapup = computed(
    () => wrapups.value.find((wrapup) => wrapup.taskId === selectedTaskId.value) ?? null
  )

  /**
   * The notes on this task that have been written since the wrapup was.
   *
   * A wrapup goes stale silently, which is the one way it can start lying. It cannot be
   * detected in general, but the cheap half can: if you have written notes since, the state it
   * describes is at least older than what you know. Counted rather than judged, and shown as a
   * remark rather than a warning.
   */
  const wrapupIsBehind = computed(() => {
    const wrapup = selectedWrapup.value
    if (!wrapup) return 0
    return taskDocuments.value.filter((document) => document.updatedAt > wrapup.updatedAt).length
  })

  /** Every session currently open, across however many tasks are being worked in parallel. */
  const runningEntries = computed(() => timeEntries.value.filter((entry) => entry.stoppedAt === null))

  /** The sessions on the task in view, most recently started first. */
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

  /** The path you are looking at, shown as a breadcrumb rather than a single opaque name. */
  const scopePath = computed<string[]>(() => {
    if (!scopedCompany.value) return []
    return scopedProject.value
      ? [scopedCompany.value.name, scopedProject.value.title]
      : [scopedCompany.value.name]
  })

  const scopeName = computed(() =>
    scopePath.value.length === 0 ? 'All work' : scopePath.value.join(' / ')
  )

  /** What you would type after `/rk` to load everything currently in view. */
  const scopeAnchor = computed(() => {
    if (scopedProject.value) return scopedProject.value.anchor
    return scopedCompany.value ? `company:${scopedCompany.value.name}` : ''
  })

  /**
   * What the current search would find if the project scope were dropped.
   *
   * A scoped search that silently hides results teaches you the note is gone, and the next
   * thing you do is write it a second time.
   */
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

  // ------------------------------------------------------------------ loading

  async function load(): Promise<void> {
    isLoading.value = true
    try {
      const [
        loadedCompanies,
        loadedProjects,
        loadedTasks,
        loadedDocuments,
        loadedWrapups,
        loadedTimeEntries
      ] = await Promise.all([
        fetchCompanies(),
        fetchProjects(),
        fetchTasks(),
        fetchAllDocuments(),
        fetchWrapups(),
        fetchTimeEntries()
      ])
      companies.value = loadedCompanies
      projects.value = loadedProjects
      tasks.value = loadedTasks
      documents.value = loadedDocuments
      wrapups.value = loadedWrapups
      timeEntries.value = loadedTimeEntries
    } finally {
      isLoading.value = false
    }
  }

  // ------------------------------------------------------------------ selecting

  function selectTask(id: TaskId): void {
    selectedTaskId.value = id
    const first = documents.value.find((document) =>
      document.tasks.some((ref) => ref.id === id)
    )
    selectedDocId.value = first?.id ?? null
    // Moving to another task leaves the wrapup pane: what was on screen described the task you
    // just left, and showing the next one's in its place is how the two get confused.
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

  // ------------------------------------------------------------------ companies

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
    // A company's name is on every project and task response, so those have to be read again.
    await Promise.all([refreshProjects(), refreshTasks()])
  }

  async function deleteCompany(id: CompanyId): Promise<void> {
    await apiDeleteCompany(id)
    if (scopeCompany.value === id) setScope(null)
    await load()
  }

  // ------------------------------------------------------------------ projects

  async function createProject(input: ProjectInput): Promise<Project> {
    const created = await apiCreateProject(input)
    projects.value = [...projects.value, created].sort((a, b) => a.label.localeCompare(b.label))
    await refreshCompanies()
    setScope(input.companyId, created.id)
    return created
  }

  /**
   * Changing the label changes the anchor, and every task under it carries that anchor in its
   * own, so the task list is read again rather than patched in place.
   */
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
      // Its tasks carry a copy of the folder, because the button that opens a session lives on
      // a task. Without this the pane goes on showing the answer from before the save, and the
      // button stays disabled on a project that now has somewhere to open.
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

  /** Where a session on this project opens. Cleared to null, never stored as an empty path. */
  function saveProjectRepoFolder(id: ProjectId, repoFolder: string): Promise<void> {
    return patchProject(id, { repoFolder: repoFolder.trim() === '' ? null : repoFolder.trim() })
  }

  // ------------------------------------------------------------------ tasks

  async function createTask(input: TaskInput): Promise<Task> {
    const created = await apiCreateTask(input)
    tasks.value = [...tasks.value, created]
    await Promise.all([refreshProjects(), refreshCompanies()])
    selectTask(created.id)
    // A task is worked the moment it exists; the timer says so rather than waiting to be told.
    await startTimer(created.id)
    return created
  }

  /** A task can move to another project, so both projects' counts and its notes go stale. */
  async function updateTask(id: TaskId, input: TaskInput): Promise<void> {
    const saved = await apiUpdateTask(id, input)
    tasks.value = tasks.value.map((task) => (task.id === id ? saved : task))
    await Promise.all([
      refreshDocuments(),
      refreshProjects(),
      refreshCompanies(),
      // The anchor a wrapup carries is built from both labels, so moving or renaming a task
      // moves it too.
      refreshWrapups()
    ])
  }

  async function deleteTask(id: TaskId): Promise<void> {
    await apiDeleteTask(id)
    tasks.value = tasks.value.filter((task) => task.id !== id)
    // The row is gone in the database too: a wrapup describes one task and cascades with it.
    wrapups.value = wrapups.value.filter((wrapup) => wrapup.taskId !== id)
    if (selectedTaskId.value === id) {
      selectedTaskId.value = null
      selectedDocId.value = null
      paneFocus.value = 'note'
    }
    await Promise.all([refreshDocuments(), refreshProjects(), refreshCompanies()])
  }

  /** Status is one keystroke, so it sends the record back unchanged apart from that field. */
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

  /**
   * The description, saved on its own from the pane that shows it.
   *
   * It goes out the way a status change does rather than the way a rename does: the record is
   * sent back with only that field moved, and none of the cascading reloads `updateTask` owes
   * to a label or a project change, because neither an anchor nor a note attachment can move
   * when a sentence is corrected.
   */
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

  // ------------------------------------------------------------------ notes

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

  /**
   * Saves the whole note, always with its full set of tasks.
   *
   * The endpoint replaces the set on every write, so sending only the task in view would
   * silently detach the note from every other one.
   */
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

  // ------------------------------------------------------------------ wrapup

  /** Opening the state of the task in view. Nothing is created; the pane handles the absence. */
  function openWrapup(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = 'wrapup'
  }

  function openDescription(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = 'description'
  }

  /** D, like W: the key that took you to the description takes you back to the note. */
  function toggleDescription(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = paneFocus.value === 'description' ? 'note' : 'description'
  }

  /** What the keyboard does: the key that took you to the wrapup takes you back to the note. */
  function toggleWrapup(): void {
    if (selectedTaskId.value === null) return
    paneFocus.value = paneFocus.value === 'wrapup' ? 'note' : 'wrapup'
  }

  /**
   * Writes the whole text, and says it was you.
   *
   * The author is not sent: the endpoint stamps HAND on anything that arrives over HTTP and
   * CLAUDE on anything that arrives over MCP, because a client that could claim to be the other
   * one would make the field worthless.
   */
  async function saveWrapupBody(taskId: TaskId, bodyMarkdown: string): Promise<void> {
    saveState.value = 'saving'
    try {
      const saved = await apiSaveWrapup(taskId, bodyMarkdown)
      const known = wrapups.value.some((wrapup) => wrapup.id === saved.id)
      wrapups.value = known
        ? wrapups.value.map((wrapup) => (wrapup.id === saved.id ? saved : wrapup))
        : [saved, ...wrapups.value]
      saveState.value = 'saved'
      // `hasWrapup` on the task row is now wrong until the task list is read again.
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

  // ------------------------------------------------------------------ time tracking

  function upsertTimeEntry(entry: TimeEntry): void {
    const known = timeEntries.value.some((candidate) => candidate.id === entry.id)
    timeEntries.value = known
      ? timeEntries.value.map((candidate) => (candidate.id === entry.id ? entry : candidate))
      : [entry, ...timeEntries.value]
  }

  /** Opens a session on this task. Whatever is running on other tasks keeps running. */
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

  // ------------------------------------------------------------------ refreshing

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
    wrapupIsBehind,
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
    toggleDescription,
    toggleWrapup,
    saveWrapupBody,
    removeWrapup,
    startTimer,
    pauseTimer,
    editTimer,
    deleteTimer
  }
})
