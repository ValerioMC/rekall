<script setup lang="ts">
import { ref, watch } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { relativeTime } from '@/common/format/relative-time'
import type { StepPass } from '@/model/catalog'

/**
 * The passes a step was sent back from, sealed above the feedback editor. Each holds what that
 * pass worked from and can no longer be edited, so feedback never overwrites the brief. The
 * latest stands open because it is what the feedback answers; earlier ones fold to a line.
 */
const props = defineProps<{ passes: readonly StepPass[] }>()

const openIndexes = ref<Set<number>>(new Set())

watch(
  () => props.passes.length,
  (count) => {
    openIndexes.value = new Set(count ? [count - 1] : [])
  },
  { immediate: true }
)

function toggle(index: number): void {
  const next = new Set(openIndexes.value)
  if (next.has(index)) next.delete(index)
  else next.add(index)
  openIndexes.value = next
}
</script>

<template>
  <ol class="flex flex-col gap-1.5" aria-label="Earlier passes" data-testid="step-passes">
    <li
      v-for="(pass, index) in passes"
      :key="pass.sentBackAt"
      class="sealed-pass rounded-[var(--radius-control)] border border-border bg-canvas"
      data-testid="step-pass"
    >
      <button
        type="button"
        class="focus-ring flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2.5 py-1.5 text-left"
        :aria-expanded="openIndexes.has(index)"
        @click="toggle(index)"
      >
        <svg class="size-3 shrink-0 text-text-subtle" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <rect x="2.2" y="5.2" width="7.6" height="5.3" rx="1.2" stroke="currentColor" stroke-width="1.1" />
          <path d="M4 5.2V3.9a2 2 0 0 1 4 0v1.3" stroke="currentColor" stroke-width="1.1" />
        </svg>
        <span class="text-[11.5px] font-medium text-text-muted">Pass {{ index + 1 }}</span>
        <span class="min-w-0 flex-1 truncate text-[10.5px] text-text-subtle">
          executed, sent back {{ relativeTime(pass.sentBackAt) }}
        </span>
        <svg
          class="size-2.5 shrink-0 text-text-subtle transition-transform"
          :class="openIndexes.has(index) && 'rotate-90'"
          viewBox="0 0 24 24"
          fill="none"
          aria-hidden="true"
        >
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <div v-if="openIndexes.has(index)" class="border-t border-border px-2.5 pb-2 pt-1.5">
        <AppMarkdownEditor
          v-if="pass.detailMarkdown?.trim()"
          :model-value="pass.detailMarkdown"
          readonly
          compact
          data-testid="step-pass-detail"
        />
        <p v-else class="text-[12px] text-text-subtle">Worked from the title alone.</p>
      </div>
    </li>
  </ol>
</template>

<style scoped>
/* Sealed: recessed onto the canvas and dimmed, so it reads as a record rather than a field. */
.sealed-pass {
  box-shadow: inset 0 1px 2px rgb(0 0 0 / 0.3);
}

.sealed-pass :deep(.md-editor-preview) {
  opacity: 0.85;
}
</style>
