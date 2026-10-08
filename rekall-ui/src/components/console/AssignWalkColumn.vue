<script lang="ts">
export interface WalkRow<RowId extends string> {
  readonly id: RowId
  readonly title: string
  /** The note's tasks under this record: shown in the accent, and only when there are some. */
  readonly attached: number
  readonly taskCount: number
  /** The project's identity dot; a company carries none. */
  readonly hue?: string
}
</script>

<script setup lang="ts" generic="Id extends string">
/**
 * One column of the note-assignment walk, company or project: the records to step into, how many
 * of the note's tasks already sit under each, and a marker on the one picked.
 */
defineProps<{
  heading: string
  regionLabel: string
  rows: readonly WalkRow<Id>[]
  picked: Id | null
  emptyText: string
  testid: string
}>()

defineEmits<{ pick: [id: Id] }>()
</script>

<template>
  <section
    class="flex min-h-0 min-w-0 flex-1 basis-0 flex-col border-b border-border sm:border-b-0 sm:border-r"
    :aria-label="regionLabel"
  >
    <div class="flex shrink-0 items-baseline justify-between px-3.5 pb-1.5 pt-3">
      <p class="eyebrow">{{ heading }}</p>
      <span v-if="rows.length" class="font-mono text-micro text-text-subtle">{{ rows.length }}</span>
    </div>
    <div class="relative min-h-0 flex-1">
      <div v-if="rows.length" class="h-full space-y-0.5 overflow-y-auto overscroll-contain px-1.5 py-1">
        <button
          v-for="row in rows"
          :key="row.id"
          :data-testid="testid"
          class="focus-ring flex w-full items-center gap-2 rounded-control px-2.5 py-2 text-left transition-colors"
          :class="row.id === picked ? 'walk-picked bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
          :aria-pressed="row.id === picked"
          @click="$emit('pick', row.id)"
        >
          <span v-if="row.hue" class="size-1.5 shrink-0 rounded-full" :style="{ backgroundColor: row.hue }" aria-hidden="true" />
          <span class="min-w-0 flex-1 truncate text-small" :title="row.title">{{ row.title }}</span>
          <span
            v-if="row.attached"
            class="shrink-0 rounded-full bg-accent-soft px-1.5 font-mono text-micro text-accent"
            :title="`${row.attached} of this note's tasks are here`"
          >
            {{ row.attached }}
          </span>
          <span class="shrink-0 font-mono text-micro text-text-subtle" :title="`${row.taskCount} tasks`">
            {{ row.taskCount }}
          </span>
        </button>
      </div>
      <div v-else class="grid h-full place-items-center px-5">
        <p class="text-center text-small leading-relaxed text-text-subtle">{{ emptyText }}</p>
      </div>
      <div
        v-if="rows.length"
        class="pointer-events-none absolute inset-x-1.5 bottom-0 h-5 bg-gradient-to-t from-surface to-transparent"
        aria-hidden="true"
      />
    </div>
  </section>
</template>

<style scoped>
.walk-picked {
  position: relative;
}

.walk-picked::before {
  content: '';
  position: absolute;
  left: 3px;
  top: 7px;
  bottom: 7px;
  width: 3px;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--color-accent-strong), var(--color-accent-deep));
}
</style>
