<script setup lang="ts">
import { computed } from 'vue'
import ProjectIcon from '@/components/ui/ProjectIcon.vue'
import { identityHue } from '@/common/identity'
import { projectIconImageSrcset, projectIconImageUrl } from '@/common/project/icon-image-url'

/** The macOS icon corner, as a share of the side, so it holds at every size. */
const CORNER = '22.37%'

const props = withDefaults(
  defineProps<{
    projectId: string
    icon: string
    imageVersion?: string | null
    /** An image not stored yet (a file just picked), shown in place of the stored one. */
    previewSrc?: string | null
    size?: number
  }>(),
  { imageVersion: null, previewSrc: null, size: 24 }
)

const hue = computed(() => identityHue(props.projectId))
const glyphSize = computed(() => Math.round(props.size * 0.58))
const src = computed(() => {
  if (props.previewSrc) return props.previewSrc
  return props.imageVersion ? projectIconImageUrl(props.projectId, props.imageVersion, props.size) : null
})
const srcset = computed(() =>
  !props.previewSrc && props.imageVersion
    ? projectIconImageSrcset(props.projectId, props.imageVersion, props.size)
    : undefined
)
</script>

<template>
  <img
    v-if="src"
    class="shrink-0 select-none object-cover"
    :src="src"
    :srcset="srcset"
    :width="size"
    :height="size"
    :style="{ borderRadius: CORNER }"
    alt=""
    draggable="false"
    data-testid="project-mark-image"
  />
  <span
    v-else
    class="grid shrink-0 place-items-center"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      borderRadius: CORNER,
      backgroundColor: hue.soft,
      color: hue.base
    }"
    aria-hidden="true"
    data-testid="project-mark-glyph"
  >
    <ProjectIcon :icon="icon" :size="glyphSize" />
  </span>
</template>
