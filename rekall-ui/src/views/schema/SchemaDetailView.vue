<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import FieldTable from '@/components/shared/FieldTable.vue'
import RelationList from '@/components/shared/RelationList.vue'
import { useSchemaStore } from '@/stores/schema.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { splitList, toPhysicalName, typeLabel } from '@/model/mappers'
import { FIELD_TYPES } from '@/model/schema'
import type { FieldType, OnDeleteAction } from '@/model/schema'
import type { EntityId, FieldId, RelationId } from '@/model/branded'

const props = defineProps<{ id: string }>()

const router = useRouter()
const schema = useSchemaStore()
const { run, isRunning } = useAsyncAction()

const entityId = computed(() => props.id as EntityId)
const entity = computed(() => schema.byId(entityId.value))

const outgoing = computed(() => schema.outgoingRelations(entityId.value))
const incoming = computed(() => schema.incomingRelations(entityId.value))

const referenceFields = computed(() => entity.value?.fields.filter((f) => f.type === 'REFERENCE') ?? [])

const isAddingField = ref(false)
const isAddingRelation = ref(false)

const fieldDraft = ref({
  label: '',
  columnName: '',
  description: '',
  type: 'TEXT' as FieldType,
  required: false,
  defaultValue: '',
  length: '' as string,
  precision: '' as string,
  scale: '' as string,
  enumValues: ''
})

const relationDraft = ref({
  targetTableId: '',
  sourceFieldId: '',
  onDelete: 'RESTRICT' as OnDeleteAction,
  description: ''
})

const typeOptions = FIELD_TYPES.map((type) => ({ value: type, label: typeLabel(type) }))
const entityOptions = computed(() =>
  schema.entities.map((candidate) => ({ value: candidate.id, label: candidate.label }))
)
const referenceFieldOptions = computed(() =>
  referenceFields.value.map((field) => ({ value: field.id, label: field.columnName }))
)
const onDeleteOptions = [
  { value: 'RESTRICT', label: 'Refuse the delete' },
  { value: 'CASCADE', label: 'Delete these records too' },
  { value: 'SET_NULL', label: 'Clear the reference' }
]

function onFieldLabelInput(value: string): void {
  fieldDraft.value.label = value
  fieldDraft.value.columnName = toPhysicalName(value)
}

function toNumber(value: string): number | null {
  return value.trim() === '' ? null : Number(value)
}

async function addField(): Promise<void> {
  const ok = await run(
    () =>
      schema.addField(entityId.value, {
        columnName: fieldDraft.value.columnName,
        label: fieldDraft.value.label,
        description: fieldDraft.value.description,
        type: fieldDraft.value.type,
        nullable: !fieldDraft.value.required,
        defaultValue: fieldDraft.value.defaultValue || null,
        length: toNumber(fieldDraft.value.length),
        precision: toNumber(fieldDraft.value.precision),
        scale: toNumber(fieldDraft.value.scale),
        enumValues: splitList(fieldDraft.value.enumValues)
      }),
    'Field added. It reaches the database when you apply the plan.'
  )
  if (ok === null) return
  isAddingField.value = false
  fieldDraft.value = {
    label: '',
    columnName: '',
    description: '',
    type: 'TEXT',
    required: false,
    defaultValue: '',
    length: '',
    precision: '',
    scale: '',
    enumValues: ''
  }
}

async function removeField(fieldId: FieldId): Promise<void> {
  await run(() => schema.removeField(fieldId), 'Field removed from the definition.')
}

async function setDisplayField(fieldId: FieldId): Promise<void> {
  const current = entity.value
  if (!current) return
  await run(
    () =>
      schema.updateEntity(entityId.value, {
        label: current.label,
        labelPlural: current.labelPlural,
        description: current.description,
        aliases: current.aliases,
        displayFieldId: fieldId
      }),
    'Identifying field set. Records will be shown by this value.'
  )
}

async function addRelation(): Promise<void> {
  const ok = await run(
    () =>
      schema.createRelation({
        sourceTableId: entityId.value,
        targetTableId: relationDraft.value.targetTableId as EntityId,
        kind: 'MANY_TO_ONE',
        sourceFieldId: relationDraft.value.sourceFieldId as FieldId,
        joinTableName: null,
        onDelete: relationDraft.value.onDelete,
        description: relationDraft.value.description
      }),
    'Relation defined.'
  )
  if (ok === null) return
  isAddingRelation.value = false
  relationDraft.value = { targetTableId: '', sourceFieldId: '', onDelete: 'RESTRICT', description: '' }
}

async function removeRelation(relationId: RelationId): Promise<void> {
  await run(() => schema.removeRelation(relationId), 'Relation removed.')
}

async function removeEntity(): Promise<void> {
  const ok = await run(() => schema.removeEntity(entityId.value), 'Entity definition deleted.')
  if (ok !== null) await router.push('/schema')
}
</script>

<template>
  <template v-if="entity">
    <AppPageHeader :title="entity.label">
      <template #back>
        <RouterLink to="/schema">
          <AppButton variant="ghost" size="sm">&larr; Schema</AppButton>
        </RouterLink>
      </template>
      <template #title-suffix>
        <code class="text-[13px] text-text-subtle">{{ entity.physicalName }}</code>
        <AppBadge :tone="entity.status === 'APPLIED' ? 'safe' : 'warn'" dot>
          {{ entity.status.toLowerCase() }}
        </AppBadge>
      </template>
      <template #actions>
        <AppButton variant="danger" size="sm" @click="removeEntity">Delete entity</AppButton>
      </template>
    </AppPageHeader>

    <div class="mx-auto w-full max-w-[1240px] space-y-8 px-8 pb-20 pt-6">
      <p class="max-w-[74ch] text-[13.5px] leading-relaxed text-text-muted">{{ entity.description }}</p>

      <section class="space-y-3.5">
        <div class="flex items-center gap-3">
          <h2 class="text-[16px] font-semibold text-text">Fields</h2>
          <div class="flex-1" />
          <AppButton size="sm" @click="isAddingField = !isAddingField">
            {{ isAddingField ? 'Cancel' : 'Add field' }}
          </AppButton>
        </div>

        <AppCard v-if="isAddingField">
          <div class="grid gap-x-5 sm:grid-cols-2">
            <AppField v-slot="{ fieldId, describedBy }" label="Name" required>
              <AppInput
        :id="fieldId"
        :described-by="describedBy"
                :model-value="fieldDraft.label"
                data-testid="field-label"
                placeholder="Status"
                @update:model-value="onFieldLabelInput"
              />
            </AppField>
            <AppField v-slot="{ fieldId, describedBy }" label="Column" required>
              <AppInput
        :id="fieldId"
        v-model="fieldDraft.columnName" :described-by="describedBy" data-testid="field-column" mono placeholder="status" />
            </AppField>
            <AppField v-slot="{ fieldId, describedBy }" label="Type">
              <AppSelect
        :id="fieldId"
        v-model="fieldDraft.type" :described-by="describedBy" :options="typeOptions" />
            </AppField>
            <AppField v-slot="{ fieldId, describedBy }" label="Default value">
              <AppInput
        :id="fieldId"
        v-model="fieldDraft.defaultValue" :described-by="describedBy" placeholder="optional" />
            </AppField>
            <AppField
              v-if="fieldDraft.type === 'TEXT' || fieldDraft.type === 'ENUM'"
              v-slot="{ fieldId, describedBy }"
             label="Maximum length">
              <AppInput
        :id="fieldId"
        v-model="fieldDraft.length" :described-by="describedBy" type="number" placeholder="255" />
            </AppField>
            <AppField v-if="fieldDraft.type === 'ENUM'" v-slot="{ fieldId, describedBy }" label="Allowed values" required>
              <AppInput
        :id="fieldId"
        v-model="fieldDraft.enumValues" :described-by="describedBy" placeholder="active, paused, done" />
            </AppField>
            <AppField v-if="fieldDraft.type === 'DECIMAL'" v-slot="{ fieldId, describedBy }" label="Precision">
              <AppInput
        :id="fieldId"
        v-model="fieldDraft.precision" :described-by="describedBy" type="number" placeholder="10" />
            </AppField>
            <AppField v-if="fieldDraft.type === 'DECIMAL'" v-slot="{ fieldId, describedBy }" label="Scale">
              <AppInput
        :id="fieldId"
        v-model="fieldDraft.scale" :described-by="describedBy" type="number" placeholder="2" />
            </AppField>
          </div>
          <AppField v-slot="{ fieldId, describedBy }" label="Description" required hint="Claude reads this to understand what the field means.">
            <AppInput
        :id="fieldId"
        v-model="fieldDraft.description"
              :described-by="describedBy"
              data-testid="field-description"
              placeholder="Stato corrente del progetto"
            />
          </AppField>
          <div class="flex items-center gap-4">
            <AppCheckbox id="field-required" v-model="fieldDraft.required" label="Required" />
            <div class="flex-1" />
            <AppButton
              variant="primary"
              :loading="isRunning"
              :disabled="!fieldDraft.columnName || !fieldDraft.description"
              @click="addField"
            >
              Add field
            </AppButton>
          </div>
        </AppCard>

        <FieldTable
          v-if="entity.fields.length"
          :fields="entity.fields"
          :display-field-id="entity.displayFieldId"
          @remove="removeField"
          @set-display="setDisplayField"
        />
        <AppEmptyState
          v-else
          title="No fields yet"
          description="An entity without fields still gets an id and timestamps, but nothing to say about itself."
        />

        <p v-if="entity.fields.length && !entity.displayFieldId" class="flex items-center gap-2 text-[13px]">
          <AppBadge tone="warn" dot>no identifier</AppBadge>
          <span class="text-text-subtle">
            Set one so records show a readable name instead of a uuid, in the UI and in Claude's answers.
          </span>
        </p>
      </section>

      <section class="space-y-3.5">
        <div class="flex items-center gap-3">
          <h2 class="text-[16px] font-semibold text-text">Relations</h2>
          <div class="flex-1" />
          <AppButton
            size="sm"
            :disabled="!referenceFields.length"
            @click="isAddingRelation = !isAddingRelation"
          >
            {{ isAddingRelation ? 'Cancel' : 'Link to an entity' }}
          </AppButton>
        </div>

        <p v-if="!referenceFields.length" class="text-[13px] text-text-subtle">
          Add a field of type <AppBadge>Reference</AppBadge> first: it is the column that carries the
          foreign key.
        </p>

        <AppCard v-if="isAddingRelation">
          <div class="grid gap-x-5 sm:grid-cols-2">
            <AppField v-slot="{ fieldId, describedBy }" label="Points at" required>
              <AppSelect
        :id="fieldId"
        v-model="relationDraft.targetTableId"
                :described-by="describedBy"
                :options="entityOptions"
                placeholder="Choose an entity"
              />
            </AppField>
            <AppField v-slot="{ fieldId, describedBy }" label="Through field" required>
              <AppSelect
        :id="fieldId"
        v-model="relationDraft.sourceFieldId"
                :described-by="describedBy"
                :options="referenceFieldOptions"
                placeholder="Choose a reference field"
              />
            </AppField>
            <AppField v-slot="{ fieldId, describedBy }" label="When the target is deleted">
              <AppSelect
        :id="fieldId"
        v-model="relationDraft.onDelete" :described-by="describedBy" :options="onDeleteOptions" />
            </AppField>
            <AppField v-slot="{ fieldId, describedBy }" label="Description" required>
              <AppInput
        :id="fieldId"
        v-model="relationDraft.description" :described-by="describedBy" placeholder="L'ambiente su cui gira" />
            </AppField>
          </div>
          <AppButton
            variant="primary"
            :loading="isRunning"
            :disabled="
              !relationDraft.targetTableId || !relationDraft.sourceFieldId || !relationDraft.description
            "
            @click="addRelation"
          >
            Create relation
          </AppButton>
        </AppCard>

        <RelationList :outgoing="outgoing" :incoming="incoming" @remove="removeRelation" />
      </section>
    </div>
  </template>

  <div v-else class="p-8">
    <AppEmptyState title="Entity not found" description="It may have been deleted." />
  </div>
</template>
