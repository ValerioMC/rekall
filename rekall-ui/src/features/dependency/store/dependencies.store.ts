import { defineStore } from 'pinia'
import { ref } from 'vue'
import { addDependency, fetchDependencies, removeDependency } from '@/features/dependency/api/dependencies.api'
import type { TaskDependency } from '@/features/dependency/model/dependency'
import type { TaskId } from '@/model/branded'

/** What each task the console has shown waits for, read when its description opens and after each change. */
export const useDependenciesStore = defineStore('dependencies', () => {
  const byTask = ref<Readonly<Record<string, readonly TaskDependency[]>>>({})

  function of(taskId: TaskId): readonly TaskDependency[] {
    return byTask.value[taskId] ?? []
  }

  function put(taskId: TaskId, dependencies: readonly TaskDependency[]): void {
    byTask.value = { ...byTask.value, [taskId]: dependencies }
  }

  async function load(taskId: TaskId): Promise<void> {
    put(taskId, await fetchDependencies(taskId))
  }

  async function add(taskId: TaskId, dependsOnTaskId: TaskId): Promise<void> {
    put(taskId, await addDependency(taskId, dependsOnTaskId))
  }

  async function remove(taskId: TaskId, dependsOnTaskId: TaskId): Promise<void> {
    put(taskId, await removeDependency(taskId, dependsOnTaskId))
  }

  return { of, load, add, remove }
})
