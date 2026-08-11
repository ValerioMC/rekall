<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import AppTextarea from '@/components/ui/AppTextarea.vue'
import type { SelectOption } from '@/components/ui/AppSelect.vue'
import { fetchRecords } from '@/api/data.api'
import { splitList } from '@/model/mappers'
import { useToastStore } from '@/stores/toast.store'
import { useSchemaStore } from '@/stores/schema.store'
import type { Entity, EntityField } from '@/model/schema'
import type { RecordValue } from '@/model/records'

/**
 * A form built from the meta-model.
 *
 * The entities are not known at build time, so the form cannot be either: each field renders
 * the control its declared type calls for, and a reference renders a picker of the records it
 * is allowed to point at.
 */
const props = defineProps<{ entity: Entity }>()

/**
 * A two-way model rather than a mutated prop. The parent owns the draft, and every edit
 * replaces the object instead of writing into it, so the change is visible to Vue and the
 * ownership of the data stays in one place.
 */
const model = defineModel<Record<string, RecordValue>>({ required: true })

const schema = useSchemaStore()
const toast = useToastStore()

const referenceOptions = ref<Record<string, readonly SelectOption[]>>({})

onMounted(async () => {
  try {
    for (const relation of schema.outgoingRelations(props.entity.id)) {
      if (!relation.sourceFieldId) continue
      const field = props.entity.fields.find((candidate) => candidate.id === relation.sourceFieldId)
      if (!field) continue
      const page = await fetchRecords(relation.targetTableName, '', 200)
      referenceOptions.value = {
        ...referenceOptions.value,
        [field.columnName]: page.records.map((record) => ({ value: record.id, label: record.label }))
      }
    }
  } catch (error) {
    toast.notifyError(error)
  }
})

function asText(field: EntityField): string {
  const value = model.value[field.columnName]
  return value === null || value === undefined ? '' : String(value)
}

function asTags(field: EntityField): string {
  const value = model.value[field.columnName]
  return Array.isArray(value) ? value.join(', ') : String(value ?? '')
}

function set(field: EntityField, value: RecordValue): void {
  model.value = { ...model.value, [field.columnName]: value }
}

function enumOptions(field: EntityField): SelectOption[] {
  return field.enumValues.map((value) => ({ value, label: value }))
}
</script>

<template>
  <div>
    <AppField
      v-for="field in entity.fields"
      :key="field.id"
      v-slot="{ fieldId, describedBy }"
      :label="field.label"
      :required="!field.nullable"
     :hint="field.type === 'BOOLEAN' ? undefined : field.description">
      <AppSelect
        v-if="field.type === 'REFERENCE'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asText(field)"
        :options="referenceOptions[field.columnName] ?? []"
        placeholder="Not set"
        @update:model-value="set(field, $event || null)"
      />

      <AppSelect
        v-else-if="field.type === 'ENUM'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asText(field)"
        :options="enumOptions(field)"
        placeholder="Not set"
        @update:model-value="set(field, $event || null)"
      />

      <AppCheckbox
        v-else-if="field.type === 'BOOLEAN'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="model[field.columnName] === true"
        :label="field.description"
        @update:model-value="set(field, $event)"
      />

      <AppTextarea
        v-else-if="field.type === 'MARKDOWN' || field.type === 'LONG_TEXT'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asText(field)"
        :placeholder="field.description"
        @update:model-value="set(field, $event)"
      />

      <AppInput
        v-else-if="field.type === 'INTEGER' || field.type === 'DECIMAL'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asText(field)"
        type="number"
        @update:model-value="set(field, $event === '' ? null : Number($event))"
      />

      <AppInput
        v-else-if="field.type === 'DATE'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asText(field)"
        type="date"
        @update:model-value="set(field, $event || null)"
      />

      <AppInput
        v-else-if="field.type === 'TAGS'"
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asTags(field)"
        placeholder="esa, backend"
        @update:model-value="set(field, splitList($event))"
      />

      <AppInput
        v-else
        :id="fieldId"
        :described-by="describedBy"
        :model-value="asText(field)"
        :placeholder="field.description"
        @update:model-value="set(field, $event || null)"
      />
    </AppField>
  </div>
</template>
