<script setup lang="ts">
import { onUnmounted, ref } from 'vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import { rkCommand } from '@/common/format/rk-command'

/** Copies `/rk <anchor>` to the clipboard; the command is in the tooltip, so the header need not print it. */
const props = defineProps<{ anchor: string }>()

const FLASH_MS = 1400

const copied = ref(false)
let timer: ReturnType<typeof setTimeout> | null = null

async function copy(): Promise<void> {
  await navigator.clipboard?.writeText(rkCommand(props.anchor))
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
    class="focus-ring grid size-6 shrink-0 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-anchor"
    :class="copied && 'flash'"
    :title="copied ? 'Copied' : `Copy ${rkCommand(anchor)}`"
    :aria-label="`Copy ${rkCommand(anchor)}`"
    @click="copy"
  >
    <CopyGlyph :copied="copied" />
  </button>
</template>
