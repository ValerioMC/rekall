<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import EntityCard from '@/components/shared/EntityCard.vue'
import { useSchemaStore } from '@/stores/schema.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { splitList, toPhysicalName } from '@/model/mappers'
import type { EntityId } from '@/model/branded'

const schema = useSchemaStore()
const { entities, isLoading } = storeToRefs(schema)
const { isRunning, run } = useAsyncAction()

const isCreating = ref(false)
const draft = ref({ label: '', labelPlural: '', physicalName: '', description: '', aliases: '' })

const canSubmit = computed(
  () => draft.value.physicalName.length > 0 && draft.value.description.trim().length > 0
)

function onLabelInput(value: string): void {
  draft.value.label = value
  draft.value.physicalName = toPhysicalName(value)
  if (!draft.value.labelPlural) draft.value.labelPlural = value ? `${value}s` : ''
}

function relationsFor(id: EntityId) {
  return schema.outgoingRelations(id)
}

async function create(): Promise<void> {
  const created = await run(
    () =>
      schema.createEntity({
        physicalName: draft.value.physicalName,
        label: draft.value.label,
        labelPlural: draft.value.labelPlural || draft.value.label,
        description: draft.value.description,
        aliases: splitList(draft.value.aliases)
      }),
    'Entity defined. Review the plan to create its table.'
  )
  if (!created) return
  isCreating.value = false
  draft.value = { label: '', labelPlural: '', physicalName: '', description: '', aliases: '' }
}
</script>

<template>
  <AppPageHeader title="Schema">
    <template #actions>
      <AppButton variant="primary" @click="isCreating = !isCreating">
        {{ isCreating ? 'Cancel' : 'New entity' }}
      </AppButton>
    </template>
  </AppPageHeader>

  <div class="mx-auto w-full max-w-[1240px] space-y-5 px-8 pb-20 pt-6">
    <p class="max-w-[74ch] text-[13.5px] leading-relaxed text-text-muted">
      Entities you define here become real PostgreSQL tables, created only when you apply a plan.
      Descriptions are not documentation: they are what Claude reads to decide which entity a question
      is about, so write them the way you would explain the entity out loud.
    </p>

    <AppCard v-if="isCreating">
      <h2 class="mb-4 text-[15px] font-semibold text-text">New entity</h2>
      <div class="grid gap-x-5 sm:grid-cols-2">
        <AppField v-slot="{ fieldId, describedBy }" label="Name" required>
          <AppInput
        :id="fieldId"
        :described-by="describedBy"
            :model-value="draft.label"
            data-testid="entity-label"
            placeholder="Project"
            @update:model-value="onLabelInput"
          />
        </AppField>
        <AppField v-slot="{ fieldId, describedBy }" label="Plural">
          <AppInput
        :id="fieldId"
        v-model="draft.labelPlural" :described-by="describedBy" data-testid="entity-plural" placeholder="Projects" />
        </AppField>
        <AppField v-slot="{ fieldId, describedBy }" label="Table name" required hint="Lower case letters, digits and underscores. Cannot change later.">
          <AppInput
        :id="fieldId"
        v-model="draft.physicalName" :described-by="describedBy" data-testid="entity-physical-name" mono placeholder="project" />
        </AppField>
        <AppField v-slot="{ fieldId, describedBy }" label="Also known as" hint="Comma separated. Helps Claude match the words you actually use.">
          <AppInput
        :id="fieldId"
        v-model="draft.aliases" :described-by="describedBy" placeholder="progetti, commesse" />
        </AppField>
      </div>
      <AppField v-slot="{ fieldId, describedBy }" label="Description" required>
        <AppInput
        :id="fieldId"
        v-model="draft.description"
          :described-by="describedBy"
          data-testid="entity-description"
          placeholder="Tutti i progetti su cui si sta lavorando"
        />
      </AppField>
      <AppButton variant="primary" :disabled="!canSubmit" :loading="isRunning" @click="create">
        Define entity
      </AppButton>
    </AppCard>

    <AppSkeleton v-if="isLoading && !entities.length" variant="cards" />

    <AppEmptyState
      v-else-if="!entities.length && !isCreating"
      title="No entities yet"
      description="Define the first one to start describing what you work on."
    >
      <AppButton variant="primary" @click="isCreating = true">New entity</AppButton>
    </AppEmptyState>

    <div v-if="entities.length" class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      <EntityCard
        v-for="entity in entities"
        :key="entity.id"
        :entity="entity"
        :relations="relationsFor(entity.id)"
      />
    </div>
  </div>
</template>
