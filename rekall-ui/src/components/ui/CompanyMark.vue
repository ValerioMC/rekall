<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import { companyInitial, companyLogoImageSrcset, companyLogoImageUrl } from '@/common/company/logo-image-url'

/** Squarer than a project's icon corner, so a company and its project read apart when stacked. */
const CORNER = '16%'

const props = withDefaults(
  defineProps<{
    companyId: string
    name: string
    imageVersion?: string | null
    /** An image not stored yet (a file just picked), shown in place of the stored one. */
    previewSrc?: string | null
    size?: number
  }>(),
  { imageVersion: null, previewSrc: null, size: 24 }
)

const hue = computed(() => identityHue(props.companyId))
const src = computed(() => {
  if (props.previewSrc) return props.previewSrc
  return props.imageVersion ? companyLogoImageUrl(props.companyId, props.imageVersion, props.size) : null
})
const srcset = computed(() =>
  !props.previewSrc && props.imageVersion
    ? companyLogoImageSrcset(props.companyId, props.imageVersion, props.size)
    : undefined
)
const initialSize = computed(() => `${Math.round(props.size * 0.5)}px`)
</script>

<template>
  <span
    class="company-mark grid shrink-0 place-items-center select-none"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      borderRadius: CORNER,
      ...(src ? {} : { '--mark-tint': hue.base, backgroundColor: hue.soft, color: hue.base })
    }"
    aria-hidden="true"
    :data-testid="src ? 'company-mark-image' : 'company-mark-initial'"
  >
    <img v-if="src" class="size-full object-cover" :src="src" :srcset="srcset" alt="" draggable="false" />
    <span v-else class="font-semibold leading-none" :style="{ fontSize: initialSize }">{{ companyInitial(name) }}</span>
  </span>
</template>
