<script setup lang="ts">
import { computed, ref } from 'vue'
import ImageUploadField from '@/features/catalog/components/record/ImageUploadField.vue'
import ProjectIcon from '@/components/ui/ProjectIcon.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { useConsoleStore } from '@/stores/console.store'
import { PROJECT_ICON_KEYS, PROJECT_ICON_LABEL } from '@/features/project/model/project-icon'
import type { ProjectId } from '@/model/branded'

/**
 * A project's icon: a picture of its own, kept here until the project is saved, or one of the
 * fixed glyphs, which shows whenever there is no picture.
 */
const props = defineProps<{
  /** The project being edited; null while it is still being created. */
  projectId: ProjectId | null
}>()

const icon = defineModel<string>({ required: true })

const store = useConsoleStore()
const imageField = ref<InstanceType<typeof ImageUploadField> | null>(null)

const storedVersion = computed(
  () => store.projects.find((candidate) => candidate.id === props.projectId)?.iconImageVersion ?? null
)

defineExpose({
  /** Uploads the picked picture, or takes the stored one off, once the project has its id. */
  async apply(projectId: ProjectId): Promise<void> {
    await imageField.value?.apply((file) => store.setProjectIconImage(projectId, file))
  }
})
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-caption">Icon</p>
  <ImageUploadField
    ref="imageField"
    class="mb-3"
    :stored-version="storedVersion"
    hint="PNG, JPEG, WebP or GIF, cropped to a square with rounded-key corners. The icon below shows without one."
    testid-prefix="record-icon"
  >
    <template #mark="{ version, previewSrc }">
      <ProjectMark
        :project-id="projectId ?? 'new-project'"
        :icon="icon"
        :image-version="version"
        :preview-src="previewSrc"
        :size="48"
      />
    </template>
  </ImageUploadField>
  <div class="flex flex-wrap gap-1.5" data-testid="record-icon-picker">
    <button
      v-for="option in PROJECT_ICON_KEYS"
      :key="option"
      type="button"
      class="focus-ring grid size-8 place-items-center rounded-control border transition-colors"
      :class="
        icon === option
          ? 'border-accent bg-accent-soft text-accent'
          : 'border-border-strong bg-canvas text-text-muted hover:border-text-subtle hover:text-text'
      "
      :aria-pressed="icon === option"
      :aria-label="PROJECT_ICON_LABEL[option]"
      :title="PROJECT_ICON_LABEL[option]"
      @click="icon = option"
    >
      <ProjectIcon :icon="option" :size="15" />
    </button>
  </div>
</template>
