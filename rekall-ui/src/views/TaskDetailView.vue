<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import DocumentPanel from '@/components/shared/DocumentPanel.vue'
import { deleteTask, fetchTask, updateTask } from '@/api/catalog.api'
import {
  createDocument,
  deleteDocument,
  fetchDocuments,
  updateDocument
} from '@/api/documents.api'
import { useCatalogStore } from '@/stores/catalog.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { TASK_STATUSES } from '@/model/catalog'
import type { RekallDocument, Task } from '@/model/catalog'
import type { DocumentId, EnvironmentId, TaskId } from '@/model/branded'

const props = defineProps<{ id: string }>()

const router = useRouter()
const catalog = useCatalogStore()
const { environments } = storeToRefs(catalog)
const { run, isRunning } = useAsyncAction()

const STATUS_OPTIONS = TASK_STATUSES.map((status) => ({ value: status, label: status }))

const taskId = props.id as TaskId
const task = ref<Task | null>(null)
/** The record is immutable; what the form binds to is a draft of it. */
const form = ref({ name: '', status: 'TODO' as Task['status'], description: '' })
const documents = ref<RekallDocument[]>([])
const environmentId = ref<string>('')

const environmentOptions = computed(() => [
  { value: '', label: 'none' },
  ...environments.value.map((environment) => ({
    value: environment.id as string,
    label: environment.label
  }))
])

onMounted(async () => {
  await catalog.load()
  await load()
})

async function load(): Promise<void> {
  const [loadedTask, loadedDocuments] = await Promise.all([
    run(() => fetchTask(taskId)),
    run(() => fetchDocuments({ taskId }))
  ])
  if (loadedTask) {
    task.value = loadedTask
    environmentId.value = loadedTask.environmentId ?? ''
    form.value = {
      name: loadedTask.name,
      status: loadedTask.status,
      description: loadedTask.description ?? ''
    }
  }
  documents.value = loadedDocuments ?? []
}

async function save(): Promise<void> {
  if (!task.value) return
  const saved = await run(
    () =>
      updateTask(taskId, {
        name: form.value.name,
        status: form.value.status,
        description: form.value.description || null,
        projectId: task.value!.projectId,
        environmentId: environmentId.value ? (environmentId.value as EnvironmentId) : null
      }),
    'Task saved.'
  )
  if (saved) task.value = saved
}

async function remove(): Promise<void> {
  if (!task.value) return
  const projectId = task.value.projectId
  const done = await run(() => deleteTask(taskId), 'Task deleted.')
  if (done === undefined) return
  await router.push(`/projects/${projectId}`)
}

async function reloadDocuments(): Promise<void> {
  const loaded = await run(() => fetchDocuments({ taskId }))
  documents.value = loaded ?? []
}
</script>

<template>
  <AppSkeleton v-if="!task" :lines="5" class="m-8" />

  <template v-else>
    <AppPageHeader
      :title="task.name"
      :subtitle="`/rk project:${task.projectName} task:${task.name}`"
    >
      <template #actions>
        <AppButton :loading="isRunning" @click="save">Save</AppButton>
        <AppButton variant="danger" @click="remove">Delete</AppButton>
      </template>
    </AppPageHeader>

    <div class="mx-auto w-full max-w-[1240px] space-y-5 px-8 pb-20 pt-6">
      <AppCard>
        <div class="grid gap-4 sm:grid-cols-2">
          <AppField label="Name">
            <AppInput v-model="form.name" />
          </AppField>
          <AppField label="Status">
            <AppSelect v-model="form.status" :options="STATUS_OPTIONS" />
          </AppField>
          <AppField label="Project">
            <RouterLink
              :to="`/projects/${task.projectId}`"
              class="focus-ring inline-block text-[13px] text-accent hover:underline"
            >
              {{ task.projectName }}
            </RouterLink>
          </AppField>
          <AppField
            label="Environment"
            hint="Its notes arrive with this task when Claude loads the context"
          >
            <AppSelect v-model="environmentId" :options="environmentOptions" />
          </AppField>
        </div>
        <AppField label="Description" class="mt-4">
          <AppTextarea v-model="form.description" :rows="3" />
        </AppField>
      </AppCard>

      <DocumentPanel
        :documents="documents"
        @create="
          async (input) => {
            await createDocument({ taskId }, input)
            await reloadDocuments()
          }
        "
        @save="
          async (docId: DocumentId, input) => {
            await updateDocument(docId, input)
            await reloadDocuments()
          }
        "
        @remove="
          async (docId: DocumentId) => {
            await deleteDocument(docId)
            await reloadDocuments()
          }
        "
      />
    </div>
  </template>
</template>
