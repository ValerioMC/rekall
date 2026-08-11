<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import DocumentPanel from '@/components/shared/DocumentPanel.vue'
import { createEnvironment, deleteEnvironment } from '@/api/catalog.api'
import {
  createDocument,
  deleteDocument,
  fetchDocuments,
  updateDocument
} from '@/api/documents.api'
import { useCatalogStore } from '@/stores/catalog.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { RekallDocument } from '@/model/catalog'
import type { DocumentId, EnvironmentId } from '@/model/branded'

const catalog = useCatalogStore()
const { environments, isLoading } = storeToRefs(catalog)
const { run, isRunning } = useAsyncAction()

const isCreating = ref(false)
const draft = ref({ label: '', namespace: '', kubeconfigPath: '' })
const openId = ref<EnvironmentId | null>(null)
const documents = ref<RekallDocument[]>([])

onMounted(() => catalog.load())

async function create(): Promise<void> {
  const created = await run(
    () =>
      createEnvironment({
        label: draft.value.label,
        namespace: draft.value.namespace || null,
        kubeconfigPath: draft.value.kubeconfigPath || null
      }),
    'Environment created.'
  )
  if (!created) return
  isCreating.value = false
  draft.value = { label: '', namespace: '', kubeconfigPath: '' }
  await catalog.load()
}

async function remove(id: EnvironmentId): Promise<void> {
  await run(() => deleteEnvironment(id), 'Environment deleted.')
  await catalog.load()
}

/** Notes on an environment are where the cluster coordinates live, so they are editable here. */
async function openNotes(id: EnvironmentId): Promise<void> {
  openId.value = openId.value === id ? null : id
  if (!openId.value) return
  const loaded = await run(() => fetchDocuments({ environmentId: id }))
  documents.value = loaded ?? []
}

async function reloadNotes(): Promise<void> {
  const id = openId.value
  if (!id) return
  const loaded = await run(() => fetchDocuments({ environmentId: id }))
  documents.value = loaded ?? []
}
</script>

<template>
  <AppPageHeader title="Environments">
    <template #actions>
      <AppButton variant="primary" @click="isCreating = !isCreating">
        {{ isCreating ? 'Cancel' : 'New environment' }}
      </AppButton>
    </template>
  </AppPageHeader>

  <div class="mx-auto w-full max-w-[1240px] space-y-5 px-8 pb-20 pt-6">
    <AppCard v-if="isCreating">
      <div class="grid gap-4 sm:grid-cols-2">
        <AppField label="Label" hint="e.g. kmaster14 / stvv-dev">
          <AppInput v-model="draft.label" />
        </AppField>
        <AppField label="Namespace">
          <AppInput v-model="draft.namespace" placeholder="stvv-dev" />
        </AppField>
      </div>
      <AppField label="Kubeconfig path" class="mt-4">
        <AppInput v-model="draft.kubeconfigPath" placeholder="/Users/.../config.kmaster14" />
      </AppField>
      <div class="mt-4 flex justify-end">
        <AppButton variant="primary" :loading="isRunning" :disabled="!draft.label" @click="create">
          Create
        </AppButton>
      </div>
    </AppCard>

    <AppSkeleton v-if="isLoading" :lines="3" />

    <AppEmptyState
      v-else-if="!environments.length"
      title="No environments yet"
      description="An environment holds a cluster and a namespace once, so tasks stop repeating them."
    />

    <div v-else class="grid gap-3">
      <AppCard v-for="environment in environments" :key="environment.id">
        <div class="flex items-start gap-3">
          <div class="min-w-0 flex-1">
            <div class="text-[15px] font-semibold text-text">{{ environment.label }}</div>
            <dl class="mt-1 grid gap-0.5 text-[12px] text-text-muted">
              <div v-if="environment.namespace">
                <dt class="inline font-mono text-text-subtle">namespace</dt>
                <dd class="inline">&nbsp;{{ environment.namespace }}</dd>
              </div>
              <div v-if="environment.kubeconfigPath">
                <dt class="inline font-mono text-text-subtle">kubeconfig</dt>
                <dd class="inline">&nbsp;{{ environment.kubeconfigPath }}</dd>
              </div>
            </dl>
            <code class="mt-1 block font-mono text-[11px] text-text-subtle">
              environment:{{ environment.label }}
            </code>
          </div>
          <div class="flex shrink-0 gap-2">
            <AppButton size="sm" @click="openNotes(environment.id)">
              {{ openId === environment.id ? 'Hide notes' : 'Notes' }}
            </AppButton>
            <AppButton size="sm" variant="danger" @click="remove(environment.id)">Delete</AppButton>
          </div>
        </div>

        <DocumentPanel
          v-if="openId === environment.id"
          class="mt-4"
          :documents="documents"
          @create="
            async (input) => {
              await createDocument({ environmentId: environment.id }, input)
              await reloadNotes()
            }
          "
          @save="
            async (id: DocumentId, input) => {
              await updateDocument(id, input)
              await reloadNotes()
            }
          "
          @remove="
            async (id: DocumentId) => {
              await deleteDocument(id)
              await reloadNotes()
            }
          "
        />
      </AppCard>
    </div>
  </div>
</template>
