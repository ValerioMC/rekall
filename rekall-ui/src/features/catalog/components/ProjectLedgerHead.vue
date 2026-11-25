<script setup lang="ts">
interface LedgerColumn {
  readonly label: string
  readonly alignEnd: boolean
}

/** The column head of a ledger of project rows; it shares their grid, so each label sits over its column. */
const COLUMNS: readonly LedgerColumn[] = [
  { label: 'Project', alignEnd: false },
  { label: 'Work', alignEnd: false },
  { label: 'Tokens', alignEnd: true },
  { label: 'Status', alignEnd: true },
  { label: 'Updated', alignEnd: true },
  { label: '', alignEnd: false }
]
</script>

<template>
  <div
    class="project-row-columns h-8 border-b border-border text-caption font-medium text-text-subtle @max-[52rem]:hidden"
    aria-hidden="true"
  >
    <span v-for="column in COLUMNS" :key="column.label" :class="column.alignEnd ? 'justify-self-end' : ''">
      {{ column.label }}
    </span>
  </div>
</template>
