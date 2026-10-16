<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import type { ProjectComponent } from '@/features/project/model/project-component'

/** A component at rest, one dense row: name, the tail of its folder and the first line of its description. Opening it swaps in the editor. */
const props = defineProps<{ component: ProjectComponent; position: number }>()

defineEmits<{ open: [] }>()

const hue = computed(() => identityHue(props.component.id))
</script>

<template>
  <button
    type="button"
    class="row focus-ring group relative grid w-full cursor-pointer grid-cols-[22px_minmax(0,1fr)_14px] items-baseline gap-x-1.5 rounded-control py-1.5 pl-2.5 pr-2 text-left"
    :style="{ '--component-hue': hue.base }"
    :aria-label="`Edit ${component.name}`"
    :title="component.description || undefined"
    :data-testid="`component-row-${component.id}`"
    @click="$emit('open')"
  >
    <span class="row-rail" aria-hidden="true" />
    <span class="font-mono text-micro tabular-nums tracking-[0.04em] text-[var(--component-hue)]" aria-hidden="true">
      {{ String(position + 1).padStart(2, '0') }}
    </span>
    <span class="truncate text-small font-semibold text-text" data-testid="component-row-name">{{ component.name }}</span>
    <svg
      class="size-3 self-center text-text-subtle opacity-0 transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100"
      viewBox="0 0 14 14"
      fill="none"
      aria-hidden="true"
    >
      <path d="M2.5 11.5 3 8.8l5.8-5.8a1 1 0 0 1 1.4 0l.8.8a1 1 0 0 1 0 1.4L5.2 11 2.5 11.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
    </svg>
    <span class="col-start-2 col-end-4 truncate font-mono text-micro text-text-subtle" dir="rtl">
      <bdo dir="ltr">{{ component.folder }}</bdo>
    </span>
    <span v-if="component.description" class="col-start-2 col-end-4 truncate text-caption text-text-muted">
      {{ component.description }}
    </span>
  </button>
</template>

<style scoped>
/* The component's own hue on the row's edge: the same mark its editor carries, so the two read as one thing. */
.row-rail {
  position: absolute;
  inset: 7px auto 7px 0;
  width: 2px;
  border-radius: 0 2px 2px 0;
  background: var(--component-hue);
}

.row {
  transition: background-color var(--motion-quick) ease;
}

.row:hover {
  background: var(--color-surface-raised);
}
</style>
