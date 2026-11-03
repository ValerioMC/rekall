import { defineStore } from 'pinia'
import { ref } from 'vue'
import { fetchProjectAdherence, fetchTaskAdherence } from '@/features/adherence/api/adherence.api'
import type { PlanAdherence, ProjectAdherence } from '@/features/adherence/model/adherence'
import type { ProjectId, TaskId } from '@/model/branded'

/** The adherence of the tasks and projects the console has shown, read again whenever a panel asks. */
export const useAdherenceStore = defineStore('adherence', () => {
  const byTask = ref<Readonly<Record<string, PlanAdherence>>>({})
  const byProject = ref<Readonly<Record<string, ProjectAdherence>>>({})

  function ofTask(taskId: TaskId): PlanAdherence | null {
    return byTask.value[taskId] ?? null
  }

  function ofProject(projectId: ProjectId): ProjectAdherence | null {
    return byProject.value[projectId] ?? null
  }

  async function loadTask(taskId: TaskId): Promise<void> {
    byTask.value = { ...byTask.value, [taskId]: await fetchTaskAdherence(taskId) }
  }

  async function loadProject(projectId: ProjectId): Promise<void> {
    byProject.value = { ...byProject.value, [projectId]: await fetchProjectAdherence(projectId) }
  }

  return { ofTask, ofProject, loadTask, loadProject }
})
