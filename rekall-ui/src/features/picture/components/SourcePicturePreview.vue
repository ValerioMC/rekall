<script setup lang="ts">
import { computed } from 'vue'
import { APP_LOCALE } from '@/common/format/app-locale'
import { sourceThumbnailUrl } from '@/features/picture/api/pictures.api'
import PictureThumbnail from '@/features/picture/components/PictureThumbnail.vue'
import type { DirectoryEntry } from '@/model/filesystem'

/** The highlighted entry of the image picker, large enough to recognise before choosing it. */
const props = defineProps<{ entry: DirectoryEntry | null }>()

const picture = computed(() => (props.entry && !props.entry.directory ? props.entry : null))

function inKilobytes(bytes: number): string {
  return `${Math.max(1, Math.round(bytes / 1024)).toLocaleString(APP_LOCALE)} KB`
}
</script>

<template>
  <figure
    class="flex h-40 shrink-0 gap-3 border-t border-border p-2.5 @xl:h-auto @xl:w-80 @xl:flex-col @xl:gap-2 @xl:border-l @xl:border-t-0"
    data-testid="picture-preview"
  >
    <template v-if="picture">
      <PictureThumbnail :key="picture.path" :src="sourceThumbnailUrl(picture.path)" :alt="picture.name" class="min-h-0 min-w-0 flex-1" />
      <figcaption class="w-40 min-w-0 self-end @xl:w-auto @xl:self-auto">
        <p class="truncate font-mono text-small text-text" :title="picture.name">{{ picture.name }}</p>
        <p v-if="picture.size !== null" class="text-caption tabular-nums text-text-subtle" data-testid="picture-preview-size">
          {{ inKilobytes(picture.size) }}
        </p>
      </figcaption>
    </template>
    <p v-else class="m-auto text-center text-caption text-text-subtle">
      {{ entry ? 'Open the folder to see its pictures.' : 'No picture highlighted.' }}
    </p>
  </figure>
</template>
