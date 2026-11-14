<script setup lang="ts">
import { computed } from 'vue'
import WorkBoardCard from '@/features/work/components/WorkBoardCard.vue'
import type { TaskId } from '@/model/branded'
import type { WorkLane } from '@/features/work/model/work-view'

const props = defineProps<{ lane: WorkLane; now: number }>()

defineEmits<{ open: [taskId: TaskId] }>()

const SHOWN = 40

const listed = computed(() => props.lane.rows.slice(0, SHOWN))
const hidden = computed(() => Math.max(0, props.lane.rows.length - SHOWN))
</script>

<template>
  <section
    class="flex min-h-0 flex-col rounded-card border bg-surface"
    :class="lane.id === 'review' ? 'border-accent/35 bg-linear-to-b from-accent-soft to-surface' : 'border-border'"
    :aria-label="lane.label"
    :data-testid="`board-lane-${lane.id}`"
  >
    <header class="flex h-11 items-center gap-2 pl-3.5 pr-3 font-medium text-text">
      {{ lane.label }}
      <span
        class="font-mono text-caption tabular-nums"
        :class="lane.id === 'review' && lane.rows.length > 0 ? 'rounded-key bg-accent px-1.5 leading-4 text-accent-ink' : 'text-text-subtle'"
        data-testid="board-lane-count"
      >
        {{ lane.rows.length }}
      </span>
    </header>
    <div class="flex min-h-0 flex-col gap-2 overflow-y-auto px-2.5 pb-3 pt-0.5">
      <WorkBoardCard
        v-for="row in listed"
        :key="row.task.id"
        :row="row"
        :now="now"
        @open="$emit('open', row.task.id)"
      />
      <p v-if="lane.rows.length === 0" class="rounded-card border border-dashed border-border px-3 py-3 text-small text-text-subtle">
        Empty
      </p>
      <p v-if="hidden > 0" class="px-1 pt-1 text-caption text-text-subtle" data-testid="board-lane-more">
        and {{ hidden }} more, newest first
      </p>
    </div>
  </section>
</template>
