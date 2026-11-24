import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { closeTask, fetchWorkingSet, openTask, rememberPlace } from '@/features/workingset/api/working-set.api'
import { groupByProject } from '@/features/workingset/lib/group-by-project'
import type { OpenPlace, OpenTask } from '@/features/workingset/model/working-set'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * The tasks the person keeps open and the place last seen in each project, saved on the server.
 * A write shows at once and is undone, with a toast, when the server refuses it.
 */
export const useWorkingSetStore = defineStore('workingSet', () => {
  const catalog = useConsoleStore()
  const toast = useToastStore()
  const openTasks = ref<readonly OpenTask[]>([])
  const places = ref<readonly OpenPlace[]>([])
  const isLoaded = ref(false)
  const currentPlace = ref<OpenPlace | null>(null)
  const previousPlace = ref<OpenPlace | null>(null)

  const groups = computed(() => groupByProject(openTasks.value, catalog.companies, catalog.projects, catalog.tasks))

  function isOpen(taskId: TaskId): boolean {
    return openTasks.value.some((entry) => entry.taskId === taskId)
  }

  function placeOf(projectId: ProjectId): OpenPlace {
    return places.value.find((place) => place.projectId === projectId) ?? { projectId, taskId: null }
  }

  async function load(): Promise<void> {
    const read = await fetchWorkingSet()
    openTasks.value = read.tasks
    places.value = read.places
    isLoaded.value = true
  }

  async function open(taskId: TaskId, projectId: ProjectId): Promise<void> {
    if (isOpen(taskId)) return
    await undoneWhenRefused(async () => {
      openTasks.value = [...openTasks.value, { taskId, projectId }]
      await openTask(taskId)
    })
  }

  /** Closes the task and answers where its project lands; null when the server refused. */
  async function close(taskId: TaskId): Promise<OpenPlace | null> {
    let landing: OpenPlace | null = null
    await undoneWhenRefused(async () => {
      openTasks.value = openTasks.value.filter((entry) => entry.taskId !== taskId)
      landing = await closeTask(taskId)
      if (placeOf(landing.projectId).taskId === taskId) putPlace(landing)
    })
    return landing
  }

  async function remember(place: OpenPlace): Promise<void> {
    const known = placeOf(place.projectId)
    if (known.taskId === place.taskId && places.value.some((entry) => entry.projectId === place.projectId)) return
    await undoneWhenRefused(async () => {
      putPlace(place)
      await rememberPlace(place.projectId, place.taskId)
    })
  }

  /** What the person is looking at now and the place before it; kept in memory, never saved. */
  function noteVisit(place: OpenPlace): void {
    if (currentPlace.value?.projectId === place.projectId && currentPlace.value.taskId === place.taskId) return
    previousPlace.value = currentPlace.value
    currentPlace.value = place
  }

  function putPlace(place: OpenPlace): void {
    places.value = [...places.value.filter((entry) => entry.projectId !== place.projectId), place]
  }

  async function undoneWhenRefused(write: () => Promise<void>): Promise<void> {
    const keptTasks = openTasks.value
    const keptPlaces = places.value
    try {
      await write()
    } catch (failure) {
      openTasks.value = keptTasks
      places.value = keptPlaces
      toast.notifyError(failure)
    }
  }

  return { openTasks, places, isLoaded, previousPlace, groups, isOpen, placeOf, load, open, close, remember, noteVisit }
})
