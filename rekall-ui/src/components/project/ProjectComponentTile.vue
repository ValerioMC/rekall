<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import type { ProjectComponent } from '@/model/project-component'

/** A component at rest: name, folder and the first lines of its description. Opening it swaps in the editor. */
const props = defineProps<{ component: ProjectComponent; position: number }>()

defineEmits<{ open: [] }>()

const hue = computed(() => identityHue(props.component.id))
</script>

<template>
  <button
    type="button"
    class="tile focus-ring group relative flex h-full w-full flex-col gap-1.5 rounded-card border border-border bg-gradient-to-b from-surface-raised to-surface p-3.5 pl-4 text-left shadow-[var(--shadow-raised)]"
    :style="{ '--component-hue': hue.base, '--component-hue-line': hue.line }"
    :aria-label="`Edit ${component.name}`"
    :data-testid="`component-tile-${component.id}`"
    @click="$emit('open')"
  >
    <span class="tile-rail" aria-hidden="true" />
    <span class="flex items-baseline gap-2">
      <span class="font-mono text-micro tabular-nums tracking-[0.04em] text-[var(--component-hue)]" aria-hidden="true">
        {{ String(position + 1).padStart(2, '0') }}
      </span>
      <span class="min-w-0 flex-1 truncate text-body font-semibold tracking-[-0.01em] text-text" data-testid="component-tile-name">
        {{ component.name }}
      </span>
      <svg
        class="size-3.5 shrink-0 text-text-subtle opacity-0 transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100"
        viewBox="0 0 14 14"
        fill="none"
        aria-hidden="true"
      >
        <path d="M2.5 11.5 3 8.8l5.8-5.8a1 1 0 0 1 1.4 0l.8.8a1 1 0 0 1 0 1.4L5.2 11 2.5 11.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
      </svg>
    </span>
    <span class="truncate font-mono text-caption text-text-subtle" dir="rtl" :title="component.folder">
      <bdo dir="ltr">{{ component.folder }}</bdo>
    </span>
    <span v-if="component.description" class="line-clamp-2 text-small leading-snug text-text-muted">
      {{ component.description }}
    </span>
    <span v-else class="text-small italic text-text-subtle">No description yet</span>
  </button>
</template>

<style scoped>
/* The component's own hue runs down the tile's edge, the same mark the open editor carries. */
.tile-rail {
  position: absolute;
  inset: 12px auto 12px 0;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: var(--component-hue);
  box-shadow: 0 0 14px -2px var(--component-hue-line);
}

.tile {
  transition:
    transform var(--motion-quick) ease,
    border-color var(--motion-quick) ease,
    box-shadow var(--motion-quick) ease;
}

.tile:hover {
  transform: translateY(-2px);
  border-color: var(--color-border-strong);
  box-shadow: var(--shadow-lift);
}
</style>
