<script setup lang="ts">
import { computed, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'

/**
 * An uploaded picture held until its record is saved: picked, previewed, or marked for removal.
 * The mark it decorates comes in through the slot, which is told what to show.
 */
const props = defineProps<{
  /** The stored picture's version; null when the record has none. */
  storedVersion: string | null
  hint: string
  /** Prefix of the test ids, so two fields on one page stay apart. */
  testidPrefix: string
}>()

defineSlots<{
  mark(props: { version: string | null; previewSrc: string | null }): unknown
}>()

const MAX_BYTES = 10 * 1024 * 1024
const ACCEPTED_TYPES = 'image/png,image/jpeg,image/webp,image/gif'

const fileField = ref<HTMLInputElement | null>(null)
const pendingImage = ref<File | null>(null)
const pendingPreview = ref<string | null>(null)
const isRemoving = ref(false)
const error = ref<string | null>(null)

const shownVersion = computed(() => (isRemoving.value ? null : props.storedVersion))
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
  isRemoving.value = props.storedVersion !== null
}

onUnmounted(releasePreview)

defineExpose({
  /** Hands `write` the picked file, or null to take the stored one off; nothing when untouched. */
  async apply(write: (file: File | null) => Promise<void>): Promise<void> {
    if (pendingImage.value) await write(pendingImage.value)
    else if (isRemoving.value) await write(null)
  }
})
</script>

<template>
  <div class="flex items-center gap-3" :data-testid="`${testidPrefix}-image`">
    <slot name="mark" :version="shownVersion" :preview-src="pendingPreview" />
    <div class="min-w-0">
      <div class="flex items-center gap-2">
        <AppButton size="sm" :data-testid="`${testidPrefix}-upload`" @click="fileField?.click()">
          {{ hasImage ? 'Replace image' : 'Upload image' }}
        </AppButton>
        <AppButton v-if="hasImage" variant="ghost" size="sm" :data-testid="`${testidPrefix}-remove`" @click="clear">
          Remove
        </AppButton>
      </div>
      <p class="mt-1.5 text-caption" :class="error ? 'text-danger' : 'text-text-subtle'" :role="error ? 'alert' : undefined">
        {{ error ?? hint }}
      </p>
    </div>
    <input ref="fileField" type="file" class="hidden" :accept="ACCEPTED_TYPES" :data-testid="`${testidPrefix}-file`" @change="pick" />
  </div>
</template>
