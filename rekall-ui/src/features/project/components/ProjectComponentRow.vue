<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import { folderName } from '@/features/project/lib/component-folder'
import type { ProjectComponent } from '@/features/project/model/project-component'

/**
 * A component as a leaf of the project's folder tree, on one line: its name, its folder when the
 * name does not already say it, and the start of its description. Opening it puts its editor
 * beside the tree.
 */
const props = defineProps<{ component: ProjectComponent; path: string; open: boolean }>()

defineEmits<{ open: [] }>()

const hue = computed(() => identityHue(props.component.id))
const shownPath = computed(() => (folderName(props.path) === props.component.name ? null : props.path))
</script>

<template>
  <button
    type="button"
    class="focus-ring group flex h-[34px] w-full cursor-pointer items-center gap-2.5 rounded-control pl-2 pr-2.5 text-left transition-colors"
    :class="open ? 'selected-row' : 'hover:bg-surface-raised'"
    :style="{ '--select-tint': hue.base, '--select-tint-soft': hue.soft }"
    :aria-current="open || undefined"
    :aria-label="`Edit ${component.name}`"
    :title="component.description || undefined"
    :data-testid="`component-row-${component.id}`"
    @click="$emit('open')"
  >
    <span class="size-1.5 shrink-0 rounded-full" :style="{ background: hue.base }" aria-hidden="true" />
    <span class="max-w-[38%] shrink-0 truncate text-small font-semibold text-text" data-testid="component-row-name">{{ component.name }}</span>
    <span v-if="shownPath" class="max-w-[30%] shrink-0 truncate font-mono text-micro text-text-subtle" dir="rtl">
      <bdo dir="ltr">{{ shownPath }}</bdo>
    </span>
    <span class="hidden min-w-0 flex-1 truncate text-caption text-text-muted sm:block">{{ component.description }}</span>
    <svg
      class="ml-auto size-3 shrink-0 text-text-subtle opacity-0 transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100"
      viewBox="0 0 14 14"
      fill="none"
      aria-hidden="true"
    >
      <path d="M2.5 11.5 3 8.8l5.8-5.8a1 1 0 0 1 1.4 0l.8.8a1 1 0 0 1 0 1.4L5.2 11 2.5 11.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
    </svg>
  </button>
</template>
