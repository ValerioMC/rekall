<script setup lang="ts">
import { computed } from 'vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import type { Task } from '@/features/catalog/model/catalog'

/** Where a task's dossier shows its steps, an epic shows the tasks it gathers and the way to its dashboard. */
const props = defineProps<{ epic: Task; children: readonly Task[] }>()

const done = computed(() => props.children.filter((task) => task.status === 'DONE').length)
</script>

<template>
  <RouterLink
    data-testid="epic-dossier-card"
    class="dossier-section focus-ring"
    :to="{ name: 'epic', params: { id: epic.id } }"
  >
    <span class="dossier-node" :class="!children.length && 'dossier-node-empty'" aria-hidden="true">
      <EpicGlyph :size="9" />
    </span>
    <span class="flex min-h-[17px] items-center gap-2">
      <span class="text-caption font-semibold tracking-[0.005em] text-text-muted">Epic tasks</span>
      <span
        class="ml-auto shrink-0 font-mono text-micro tabular-nums"
        :class="children.length && done === children.length ? 'text-safe' : 'text-text-muted'"
      >
        {{ done }}/{{ children.length }}
      </span>
    </span>
    <StatusMixBar v-if="children.length" class="mt-2" :tasks="children" size="md" />
    <span class="mt-1.5 flex items-center gap-1 text-caption text-epic">
      Open the dashboard
      <span aria-hidden="true">&#8594;</span>
    </span>
  </RouterLink>
</template>
