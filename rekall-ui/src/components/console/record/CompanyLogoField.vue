<script setup lang="ts">
import { computed, ref } from 'vue'
import ImageUploadField from '@/components/console/record/ImageUploadField.vue'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import { useConsoleStore } from '@/stores/console.store'
import type { CompanyId } from '@/model/branded'

/** A company's logo, kept here until the company is saved; without one it wears its initial. */
const props = defineProps<{
  /** The company being edited; null while it is still being created. */
  companyId: CompanyId | null
  /** The name as typed, so the fallback initial follows it. */
  name: string
}>()

const store = useConsoleStore()
const imageField = ref<InstanceType<typeof ImageUploadField> | null>(null)

const storedVersion = computed(
  () => store.companies.find((candidate) => candidate.id === props.companyId)?.logoImageVersion ?? null
)

defineExpose({
  /** Uploads the picked logo, or takes the stored one off, once the company has its id. */
  async apply(companyId: CompanyId): Promise<void> {
    await imageField.value?.apply((file) => store.setCompanyLogoImage(companyId, file))
  }
})
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Logo</p>
  <ImageUploadField
    ref="imageField"
    :stored-version="storedVersion"
    hint="PNG, JPEG, WebP or GIF, cropped to a square. Shown in the scope picker; the initial stands in without one."
    testid-prefix="record-logo"
  >
    <template #mark="{ version, previewSrc }">
      <CompanyMark
        :company-id="companyId ?? 'new-company'"
        :name="name"
        :image-version="version"
        :preview-src="previewSrc"
        :size="48"
      />
    </template>
  </ImageUploadField>
</template>
