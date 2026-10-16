<script setup lang="ts">
import KindGlyph from '@/features/diagram/components/canvas/KindGlyph.vue'
import { KIND_INFO, OFFERED_KINDS } from '@/features/diagram/lib/kinds'
import type { DiagramKind } from '@/features/diagram/model/diagram'

/**
 * The diagram types to draw, any number of them: each a toggle that says what it is for, and
 * whether its boxes will open the code (the types the console draws itself).
 */
const props = defineProps<{ modelValue: readonly DiagramKind[] }>()

const emit = defineEmits<{ 'update:modelValue': [kinds: DiagramKind[]] }>()

function toggle(kind: DiagramKind): void {
  const chosen = props.modelValue.includes(kind) ? props.modelValue.filter((known) => known !== kind) : [...props.modelValue, kind]
  emit('update:modelValue', OFFERED_KINDS.filter((offered) => chosen.includes(offered)))
}
</script>

<template>
  <div class="grid grid-cols-2 gap-1.5 sm:grid-cols-3" role="group" aria-label="Diagram types">
    <button
      v-for="kind in OFFERED_KINDS"
      :key="kind"
      type="button"
      class="focus-ring flex min-w-0 items-start gap-2 rounded-control border px-2.5 py-2 text-left transition-colors"
      :class="modelValue.includes(kind) ? 'border-accent/60 bg-accent-soft text-text' : 'border-border bg-canvas text-text-muted hover:border-border-strong hover:text-text'"
      :aria-pressed="modelValue.includes(kind)"
      :data-testid="`kind-${kind}`"
      @click="toggle(kind)"
    >
      <KindGlyph :kind="kind" :size="16" class="mt-px shrink-0" :class="modelValue.includes(kind) ? 'text-accent' : ''" />
      <span class="min-w-0">
        <span class="flex items-center gap-1 text-small font-medium">
          {{ KIND_INFO[kind].label }}
          <span v-if="KIND_INFO[kind].drawnHere" class="font-mono text-micro text-kind-action" title="Its boxes open the code">&lt;/&gt;</span>
        </span>
        <span class="block truncate text-micro text-text-subtle">{{ KIND_INFO[kind].purpose }}</span>
      </span>
    </button>
  </div>
</template>
