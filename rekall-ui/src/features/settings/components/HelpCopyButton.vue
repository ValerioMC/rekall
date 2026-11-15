<script setup lang="ts">
import { onUnmounted, ref } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'

const props = defineProps<{ text: string; label: string }>()

const FLASH_MS = 1400

const copied = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

async function copy(): Promise<void> {
  await navigator.clipboard?.writeText(props.text)
  copied.value = true
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => (copied.value = false), FLASH_MS)
}

onUnmounted(() => {
  if (timer) clearTimeout(timer)
})
</script>

<template>
  <button
    type="button"
    class="focus-ring grid size-7 shrink-0 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-anchor"
    :title="copied ? 'Copied' : label"
    :aria-label="label"
    data-testid="help-copy"
    @click="copy"
  >
    <CopyGlyph :copied="copied" />
  </button>
</template>
