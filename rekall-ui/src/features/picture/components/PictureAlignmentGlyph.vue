<script setup lang="ts">
import type { PictureAlignment } from '@/features/picture/model/picture'

/** A picture between two lines of a page, where the alignment puts it. */
defineProps<{ alignment: PictureAlignment }>()

const PICTURE_LEFT_EDGE: Readonly<Record<PictureAlignment, number>> = { left: 1, center: 3.5, right: 6 }
</script>

<template>
  <svg class="size-3.5 shrink-0" viewBox="0 0 14 14" fill="none" aria-hidden="true">
    <path d="M1 2h12M1 12h12" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" opacity="0.55" />
    <rect :x="PICTURE_LEFT_EDGE[alignment]" y="4.5" width="7" height="5" rx="1" fill="currentColor" />
  </svg>
</template>
