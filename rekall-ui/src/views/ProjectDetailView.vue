<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import DocumentPanel from '@/components/shared/DocumentPanel.vue'
import {
  createTask,
  deleteProject,
  fetchProject,
  fetchTasks,
  updateProject
} from '@/api/catalog.api'
import {
  createDocument,
  deleteDocument,
  fetchDocuments,
  updateDocument
} from '@/api/documents.api'
import { useCatalogStore } from '@/stores/catalog.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { PROJECT_STATUSES } from '@/model/catalog'
import type { Project, RekallDocument, Task } from '@/model/catalog'
import type { DocumentId, ProjectId } from '@/model/branded'

const props = defineProps<{ id: string }>()

const router = useRouter()
const catalog = useCatalogStore()
const { run, isRunning } = useAsyncAction()

const STATUS_OPTIONS = PROJECT_STATUSES.map((status) => ({ value: status, label: status }))

const projectId = props.id as ProjectId
const project = ref<Project | null>(null)
/** The record is immutable; what the form binds to is a draft of it. */
const form = ref({ name: '', status: 'ACTIVE' as Project['status'], description: '' })
const tasks = ref<Task[]>([])
const documents = ref<RekallDocument[]>([])
const isAddingTask = ref(false)
const taskName = ref('')

onMounted(load)

async function load(): Promise<void> {
  const [loadedProject, loadedTasks, loadedDocuments] = await Promise.all([
    run(() => fetchProject(projectId)),
    run(() => fetchTasks(projectId)),
    run(() => fetchDocuments({ projectId }))
  ])
  if (loadedProject) {
    project.value = loadedProject
    form.value = {
      name: loadedProject.name,
      status: loadedProject.status,
      description: loadedProject.description ?? ''
    }
  }
  tasks.value = loadedTasks ?? []
  documents.value = loadedDocuments ?? []
}

async function save(): Promise<void> {
  if (!project.value) return
  await run(
    () =>
      updateProject(projectId, {
        name: form.value.name,
        status: form.value.status,
        description: form.value.description || null
      }),
    'Project saved.'
  )
  await catalog.load()
}

async function remove(): Promise<void> {
  const done = await run(() => deleteProject(projectId), 'Project deleted.')
  if (done === undefined) return
  await catalog.load()
  await router.push('/projects')
}

async function addTask(): Promise<void> {
  const created = await run(
    () =>
      createTask({
        name: taskName.value,
        status: 'TODO',
        description: null,
        projectId,
        environmentId: null
      }),
    'Task created.'
  )
  if (!created) return
  isAddingTask.value = false
  taskName.value = ''
  await load()
}

async function reloadDocuments(): Promise<void> {
  const loaded = await run(() => fetchDocuments({ projectId }))
  documents.value = loaded ?? []
}
</script>

<template>
  <AppSkeleton v-if="!project" :lines="5" class="m-8" />

  <template v-else>
    <AppPageHeader :title="project.name" :subtitle="`project:${project.name}`">
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
        </div>
        <AppField label="Description" class="mt-4">
          <AppTextarea v-model="form.description" :rows="3" />
        </AppField>
      </AppCard>

      <AppCard>
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-[13px] font-semibold uppercase tracking-[0.06em] text-text-subtle">
            Tasks
          </h2>
          <AppButton size="sm" @click="isAddingTask = !isAddingTask">
            {{ isAddingTask ? 'Cancel' : 'Add task' }}
          </AppButton>
        </div>

        <div v-if="isAddingTask" class="mb-4 flex gap-2">
          <AppInput v-model="taskName" placeholder="code-validator-main-workflow" class="flex-1" />
          <AppButton variant="primary" :disabled="!taskName" @click="addTask">Create</AppButton>
        </div>

        <AppEmptyState
          v-if="!tasks.length"
          title="No tasks"
          description="A task is what a working session is opened around."
        />
        <div v-else class="grid gap-2">
          <RouterLink v-for="task in tasks" :key="task.id" :to="`/tasks/${task.id}`">
            <div
              class="flex items-center gap-3 rounded-[var(--radius-control)] border border-border px-3 py-2 transition-colors hover:border-border-strong"
            >
              <span class="min-w-0 flex-1 truncate text-[13px] text-text">{{ task.name }}</span>
              <AppBadge>{{ task.status }}</AppBadge>
              <code v-if="task.environmentLabel" class="font-mono text-[11px] text-text-subtle">
                {{ task.environmentLabel }}
              </code>
            </div>
          </RouterLink>
        </div>
      </AppCard>

      <DocumentPanel
        :documents="documents"
        @create="
          async (input) => {
            await createDocument({ projectId }, input)
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
