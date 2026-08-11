<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import DynamicRecordForm from '@/components/shared/DynamicRecordForm.vue'
import RecordTable from '@/components/shared/RecordTable.vue'
import { createRecord, fetchRecords } from '@/api/data.api'
import { useSchemaStore } from '@/stores/schema.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { RecordPage, RecordValue } from '@/model/records'
import type { EntityName } from '@/model/branded'

const props = defineProps<{ entity?: string }>()

const schema = useSchemaStore()
const { run, isRunning } = useAsyncAction()

const page = ref<RecordPage | null>(null)
const search = ref('')
const isCreating = ref(false)
const draft = ref<Record<string, RecordValue>>({})

const entityName = computed(() => (props.entity ?? '') as EntityName)
const entity = computed(() => (props.entity ? schema.byName(entityName.value) : null))

async function load(): Promise<void> {
  if (!props.entity) return
  const result = await run(() => fetchRecords(entityName.value, search.value))
  if (result) page.value = result
}

watch(() => [props.entity, schema.entities.length], load, { immediate: true })

async function create(): Promise<void> {
  const created = await run(() => createRecord(entityName.value, draft.value), 'Record created.')
  if (!created) return
  isCreating.value = false
  draft.value = {}
  await load()
}
</script>

<template>
  <AppPageHeader :title="entity?.labelPlural ?? 'Data'">
    <template #actions>
      <div class="w-[220px]">
        <AppInput v-model="search" type="search" placeholder="Filter by name" @keyup.enter="load" />
      </div>
      <AppButton size="md" :loading="isRunning" @click="load">Search</AppButton>
      <AppButton v-if="entity" variant="primary" @click="isCreating = !isCreating">
        {{ isCreating ? 'Cancel' : 'New record' }}
      </AppButton>
    </template>
  </AppPageHeader>

  <div class="mx-auto w-full max-w-[1240px] space-y-5 px-8 pb-20 pt-6">
    <AppEmptyState
      v-if="!props.entity"
      title="Pick an entity"
      description="Choose one from the sidebar to browse its records."
    />

    <template v-else-if="entity">
      <p class="max-w-[74ch] text-[13.5px] leading-relaxed text-text-muted">{{ entity.description }}</p>

      <AppCard v-if="isCreating">
        <h2 class="mb-4 text-[15px] font-semibold text-text">New {{ entity.label.toLowerCase() }}</h2>
        <DynamicRecordForm v-model="draft" :entity="entity" />
        <AppButton variant="primary" :loading="isRunning" @click="create">Create</AppButton>
      </AppCard>

      <AppSkeleton v-if="isRunning && !page" variant="table" />
      <RecordTable v-else-if="page?.records.length" :entity="entity" :records="page.records" />
      <AppEmptyState
        v-else-if="page"
        title="No records"
        :description="`Nothing stored under ${entity.labelPlural.toLowerCase()} yet.`"
      />

      <p v-if="page" class="text-[12px] text-text-subtle">
        {{ page.records.length }} of {{ page.total }}
      </p>
    </template>

    <AppEmptyState
      v-else
      title="Not applied yet"
      description="This entity is defined but its table does not exist. Review the plan and apply it."
    />
  </div>
</template>
