<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import DocumentPanel from '@/components/shared/DocumentPanel.vue'
import DynamicRecordForm from '@/components/shared/DynamicRecordForm.vue'
import { deleteRecord, fetchRecord, updateRecord } from '@/api/data.api'
import {
  createDocument,
  deleteDocument,
  fetchDocuments,
  updateDocument
} from '@/api/documents.api'
import { useSchemaStore } from '@/stores/schema.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { toFormModel } from '@/model/mappers'
import { isResolvedReference } from '@/model/records'
import type { EntityRecord, RecordValue, RekallDocument } from '@/model/records'
import type { DocumentId, EntityName, RecordId } from '@/model/branded'

const props = defineProps<{ entity: string; id: string }>()

const router = useRouter()
const schema = useSchemaStore()
const { run, isRunning } = useAsyncAction()

const entityName = computed(() => props.entity as EntityName)
const recordId = computed(() => props.id as RecordId)
const definition = computed(() => schema.byName(entityName.value))

const record = ref<EntityRecord | null>(null)
const documents = ref<readonly RekallDocument[]>([])
const form = ref<Record<string, RecordValue>>({})

/** Resolved references, shown as a readable summary rather than a uuid. */
const references = computed(() =>
  Object.entries(record.value?.values ?? {})
    .filter(([, value]) => isResolvedReference(value))
    .map(([column, value]) => ({ column, target: value as EntityRecord }))
)

async function load(): Promise<void> {
  const loaded = await run(() => fetchRecord(entityName.value, recordId.value))
  if (!loaded) return
  record.value = loaded
  form.value = toFormModel(loaded)
  const loadedDocuments = await run(() => fetchDocuments(entityName.value, recordId.value))
  if (loadedDocuments) documents.value = loadedDocuments
}

onMounted(load)

async function save(): Promise<void> {
  const saved = await run(() => updateRecord(entityName.value, recordId.value, form.value), 'Saved.')
  if (saved) await load()
}

async function remove(): Promise<void> {
  const removed = await run(
    () => deleteRecord(entityName.value, recordId.value),
    'Record and its documents deleted.'
  )
  if (removed !== null) await router.push(`/data/${props.entity}`)
}

async function addDocument(input: { title: string; kind: string; bodyMarkdown: string }): Promise<void> {
  const created = await run(
    () => createDocument({ entityName: entityName.value, recordId: recordId.value, ...input }),
    'Document added.'
  )
  if (created) await load()
}

async function saveDocument(
  id: DocumentId,
  input: { title: string; kind: string; bodyMarkdown: string }
): Promise<void> {
  const saved = await run(() => updateDocument(id, input), 'Document saved.')
  if (saved) await load()
}

async function removeDocument(id: DocumentId): Promise<void> {
  const removed = await run(() => deleteDocument(id), 'Document deleted.')
  if (removed !== null) await load()
}
</script>

<template>
  <template v-if="record && definition">
    <AppPageHeader :title="record.label">
      <template #back>
        <RouterLink :to="`/data/${entity}`">
          <AppButton variant="ghost" size="sm">&larr; {{ definition.labelPlural }}</AppButton>
        </RouterLink>
      </template>
      <template #actions>
        <AppButton variant="danger" size="sm" @click="remove">Delete</AppButton>
        <AppButton variant="primary" :loading="isRunning" @click="save">Save</AppButton>
      </template>
    </AppPageHeader>

    <div class="mx-auto grid w-full max-w-[1240px] gap-6 px-8 pb-20 pt-6 lg:grid-cols-[minmax(300px,380px)_1fr]">
      <AppCard class="self-start">
        <h2 class="mb-4 text-[15px] font-semibold text-text">Fields</h2>
        <DynamicRecordForm v-model="form" :entity="definition" />

        <template v-if="references.length">
          <hr class="my-4 border-border" />
          <div data-testid="record-references" class="sr-only">references</div>
          <h3 class="mb-3 text-[13px] font-semibold text-text">Points at</h3>
          <div v-for="reference in references" :key="reference.column" class="mb-3.5 last:mb-0">
            <code class="text-[11.5px] text-text-subtle">{{ reference.column }}</code>
            <RouterLink
              :to="`/data/${reference.target.entityName}/${reference.target.id}`"
              class="focus-ring mt-0.5 block text-[13.5px] font-medium text-accent underline-offset-4 hover:underline"
            >
              {{ reference.target.label }}
            </RouterLink>
            <dl class="mt-1 space-y-0.5">
              <div
                v-for="(value, key) in reference.target.values"
                :key="key"
                class="flex gap-2 text-[11.5px]"
              >
                <template v-if="value !== null && typeof value !== 'object'">
                  <dt class="font-mono text-text-subtle">{{ key }}</dt>
                  <dd class="min-w-0 flex-1 break-words text-text-muted">{{ value }}</dd>
                </template>
              </div>
            </dl>
          </div>
        </template>
      </AppCard>

      <DocumentPanel
        :documents="documents"
        @create="addDocument"
        @save="saveDocument"
        @remove="removeDocument"
      />
    </div>
  </template>

  <div v-else class="p-8">
    <AppEmptyState title="Record not found" description="It may have been deleted." />
  </div>
</template>
