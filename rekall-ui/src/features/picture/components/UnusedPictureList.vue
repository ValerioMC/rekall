<script setup lang="ts">
import { formatFileSize } from '@/common/format/file-size'
import { keptPictureUrl } from '@/features/picture/api/picture-cleanup.api'
import PictureThumbnail from '@/features/picture/components/PictureThumbnail.vue'
import { pictureName } from '@/features/picture/lib/picture-cleanup-wording'
import type { UnusedPicture } from '@/features/picture/model/picture-cleanup'

/** The pictures a deletion would remove, each drawn with where it is and what it weighs. */
defineProps<{ pictures: readonly UnusedPicture[] }>()
</script>

<template>
  <!-- Focusable so a long list scrolls from the keyboard: nothing inside a row takes focus. -->
  <ul class="focus-ring max-h-[min(52vh,560px)] divide-y divide-border overflow-y-auto" tabindex="0" aria-label="Unused images">
    <li v-for="picture in pictures" :key="picture.url" class="flex items-center gap-4 px-5 py-2.5" data-testid="unused-picture">
      <PictureThumbnail :src="keptPictureUrl(picture.url)" class="h-12 w-16 shrink-0 min-[720px]:h-16 min-[720px]:w-24" />
      <span class="min-w-0 flex-1">
        <span class="block break-all text-body text-text">{{ pictureName(picture.url) }}</span>
        <span class="mt-0.5 block break-all font-mono text-caption text-text-subtle">{{ picture.folder }}</span>
      </span>
      <span class="shrink-0 text-small tabular-nums text-text-muted">{{ formatFileSize(picture.sizeBytes) }}</span>
    </li>
  </ul>
</template>
