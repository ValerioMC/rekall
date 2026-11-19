<script setup lang="ts">
import { ref } from 'vue'
import CompanyLogoField from '@/features/catalog/components/record/CompanyLogoField.vue'
import RecordDescriptionField from '@/features/catalog/components/record/RecordDescriptionField.vue'
import RecordIdentityFields from '@/features/catalog/components/record/RecordIdentityFields.vue'
import type { CompanyDraft } from '@/features/catalog/model/record-draft'
import type { CompanyId } from '@/model/branded'

/** The fields of a company: its name, its logo and its description. */
defineProps<{ titleError: string | null }>()

const draft = defineModel<CompanyDraft>({ required: true })
const logoField = ref<InstanceType<typeof CompanyLogoField> | null>(null)

defineExpose({
  async applyImage(companyId: CompanyId): Promise<void> {
    await logoField.value?.apply(companyId)
  }
})
</script>

<template>
  <RecordIdentityFields
    v-model:title="draft.name"
    kind="company"
    :is-new="draft.id === null"
    :title-error="titleError"
    autofocus
  />
  <CompanyLogoField ref="logoField" :company-id="draft.id" :name="draft.name" />
  <RecordDescriptionField v-model="draft.description" />
</template>
