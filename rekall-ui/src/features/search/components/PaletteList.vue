<script setup lang="ts">
import PaletteRow from '@/features/search/components/PaletteRow.vue'
import type { PaletteEntry, PaletteGroup, PaletteScope, PaletteSection } from '@/features/search/model/palette'

defineProps<{
  sections: readonly PaletteSection[]
  scope: PaletteScope
  terms: readonly string[]
  activeKey: string | null
}>()
const emit = defineEmits<{ pick: [entry: PaletteEntry]; hover: [entry: PaletteEntry]; widen: [group: PaletteGroup] }>()
</script>

<template>
  <div id="palette-list" role="listbox" aria-label="Results" class="max-h-[min(440px,60vh)] overflow-y-auto pb-2">
    <div v-for="section in sections" :key="section.label" role="group" :aria-label="section.label">
      <p class="flex h-8 items-center gap-2 px-4 pt-2 text-micro font-semibold uppercase tracking-[0.06em] text-text-subtle" aria-hidden="true">
        {{ section.label }}
        <span class="font-normal normal-case tracking-normal tabular-nums">{{ section.total }}</span>
        <button
          v-if="scope === 'all' && section.total > section.entries.length"
          type="button"
          tabindex="-1"
          class="ml-auto h-[22px] cursor-pointer rounded-key px-2 text-caption font-medium normal-case tracking-normal text-text-muted transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
          data-testid="palette-widen"
          @mousedown.prevent="emit('widen', section.group)"
        >
          Show all {{ section.total }}
        </button>
      </p>
      <PaletteRow
        v-for="entry in section.entries"
        :key="entry.key"
        :entry="entry"
        :terms="terms"
        :active="entry.key === activeKey"
        @pick="emit('pick', $event)"
        @hover="emit('hover', $event)"
      />
    </div>
  </div>
</template>
