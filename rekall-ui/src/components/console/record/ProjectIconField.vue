<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ProjectIcon from '@/components/ui/ProjectIcon.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { useConsoleStore } from '@/stores/console.store'
import { PROJECT_ICON_KEYS, PROJECT_ICON_LABEL } from '@/model/project-icon'
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

const MAX_BYTES = 10 * 1024 * 1024
const ACCEPTED_TYPES = 'image/png,image/jpeg,image/webp,image/gif'

const store = useConsoleStore()
const fileField = ref<HTMLInputElement | null>(null)
const pendingImage = ref<File | null>(null)
const pendingPreview = ref<string | null>(null)
const isRemoving = ref(false)
const error = ref<string | null>(null)

const storedVersion = computed(
  () => store.projects.find((candidate) => candidate.id === props.projectId)?.iconImageVersion ?? null
)
const shownVersion = computed(() => (isRemoving.value ? null : storedVersion.value))
const hasImage = computed(() => pendingImage.value !== null || shownVersion.value !== null)

function releasePreview(): void {
  if (pendingPreview.value) URL.revokeObjectURL(pendingPreview.value)
  pendingPreview.value = null
}

/** A file over 10 MB is refused and nothing is kept. */
function pick(event: Event): void {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0] ?? null
  input.value = ''
  if (!file) return
  if (file.size > MAX_BYTES) {
    error.value = 'The image is larger than 10 MB.'
    return
  }
  error.value = null
  releasePreview()
  pendingImage.value = file
  pendingPreview.value = URL.createObjectURL(file)
  isRemoving.value = false
}

function clear(): void {
  error.value = null
  releasePreview()
  pendingImage.value = null
  isRemoving.value = storedVersion.value !== null
}

onUnmounted(releasePreview)

defineExpose({
  /** Uploads the picked picture, or takes the stored one off, once the project has its id. */
  async apply(projectId: ProjectId): Promise<void> {
    if (pendingImage.value) await store.setProjectIconImage(projectId, pendingImage.value)
    else if (isRemoving.value) await store.setProjectIconImage(projectId, null)
  }
})
</script>

<template>
  <p class="mb-1.5 mt-5 eyebrow text-[11px]">Icon</p>
  <div class="mb-3 flex items-center gap-3" data-testid="record-icon-image">
    <ProjectMark
      :project-id="projectId ?? 'new-project'"
      :icon="icon"
      :image-version="shownVersion"
      :preview-src="pendingPreview"
      :size="48"
    />
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <AppButton size="sm" data-testid="record-icon-upload" @click="fileField?.click()">
          {{ hasImage ? 'Replace image' : 'Upload image' }}
        </AppButton>
        <AppButton v-if="hasImage" variant="ghost" size="sm" data-testid="record-icon-remove" @click="clear">
          Remove
        </AppButton>
      </div>
      <p class="mt-1.5 text-[11.5px]" :class="error ? 'text-danger' : 'text-text-subtle'" :role="error ? 'alert' : undefined">
        {{ error ?? 'PNG, JPEG, WebP or GIF, cropped to a square with rounded corners. The icon below shows without one.' }}
      </p>
    </div>
    <input ref="fileField" type="file" class="hidden" :accept="ACCEPTED_TYPES" data-testid="record-icon-file" @change="pick" />
  </div>
  <div class="flex flex-wrap gap-1.5" data-testid="record-icon-picker">
    <button
      v-for="option in PROJECT_ICON_KEYS"
      :key="option"
      type="button"
      class="focus-ring grid size-8 place-items-center rounded-[var(--radius-control)] border transition-colors"
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
