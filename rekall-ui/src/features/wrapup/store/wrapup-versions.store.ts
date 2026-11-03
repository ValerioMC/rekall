import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchWrapupVersion, fetchWrapupVersions } from '@/features/wrapup/api/wrapup-versions.api'
import { useConsoleStore } from '@/stores/console.store'
import type { WrapupVersion, WrapupVersionDetail } from '@/features/wrapup/model/wrapup-version'
import type { TaskId, WrapupVersionId } from '@/model/branded'

/** The earlier texts of the wrapups the console has shown, newest first, read when a task's Versions panel opens. */
export const useWrapupVersionsStore = defineStore('wrapupVersions', () => {
  const versionsByTask = ref<Readonly<Record<string, readonly WrapupVersion[]>>>({})

  function versionsOf(taskId: TaskId): readonly WrapupVersion[] {
    return versionsByTask.value[taskId] ?? []
  }

  async function load(taskId: TaskId): Promise<void> {
    versionsByTask.value = { ...versionsByTask.value, [taskId]: await fetchWrapupVersions(taskId) }
  }

  function read(taskId: TaskId, versionId: WrapupVersionId): Promise<WrapupVersionDetail> {
    return fetchWrapupVersion(taskId, versionId)
  }

  async function restore(taskId: TaskId, versionId: WrapupVersionId): Promise<void> {
    const version = await read(taskId, versionId)
    await useConsoleStore().saveWrapupBody(taskId, version.bodyMarkdown)
    await load(taskId)
  }

  return { versionsOf, load, read, restore }
})
