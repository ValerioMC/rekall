<script setup lang="ts">
import { computed } from 'vue'
import DiffCode from '@/features/commits/components/DiffCode.vue'
import { splitRows, unifiedRows } from '@/features/commits/lib/diff-rows'
import type { CodeLanguage } from '@/features/commits/lib/code-tokens'
import type { DiffLayout } from '@/features/commits/model/changeScope'
import type { DiffHunk, DiffLineKind } from '@/features/commits/model/taskChanges'

/** The lines of one hunk, old file beside new (`split`) or one column (`unified`), each with its real line numbers. */
const props = defineProps<{ hunk: DiffHunk; layout: DiffLayout; language: CodeLanguage }>()

const unified = computed(() => unifiedRows(props.hunk, props.language))
const split = computed(() => splitRows(props.hunk, props.language))

const ROW_TINT: Readonly<Record<DiffLineKind | 'EMPTY', string>> = {
  CONTEXT: 'text-text-muted',
  ADDED: 'bg-safe/10 text-text',
  REMOVED: 'bg-danger/10 text-text',
  EMPTY: ''
}
const NUMBER_TINT: Readonly<Record<DiffLineKind | 'EMPTY', string>> = {
  CONTEXT: 'text-text-subtle',
  ADDED: 'bg-safe/15 text-safe',
  REMOVED: 'bg-danger/15 text-danger',
  EMPTY: ''
}
const SIGN: Readonly<Record<DiffLineKind, string>> = { CONTEXT: '', ADDED: '+', REMOVED: '−' }

function sideOf(kind: DiffLineKind | 'EMPTY'): 'added' | 'removed' | null {
  if (kind === 'ADDED') return 'added'
  return kind === 'REMOVED' ? 'removed' : null
}
</script>

<template>
  <div class="font-mono text-caption leading-relaxed" data-testid="diff-hunk-rows" :data-layout="layout">
    <template v-if="layout === 'unified'">
      <div
        v-for="(row, index) in unified"
        :key="index"
        class="grid grid-cols-[3rem_3rem_1.25rem_minmax(0,1fr)]"
        :class="ROW_TINT[row.kind]"
        data-testid="diff-row"
      >
        <span class="select-none pr-2.5 text-right tabular-nums" :class="NUMBER_TINT[row.kind]">{{ row.oldNumber }}</span>
        <span class="select-none pr-2.5 text-right tabular-nums" :class="NUMBER_TINT[row.kind]">{{ row.newNumber }}</span>
        <span class="select-none text-center" :class="NUMBER_TINT[row.kind]">{{ SIGN[row.kind] }}</span>
        <span class="whitespace-pre-wrap px-3 [overflow-wrap:anywhere]"><DiffCode :parts="row.parts" :side="sideOf(row.kind)" /></span>
      </div>
    </template>
    <template v-else>
      <div v-for="(row, index) in split" :key="index" class="grid grid-cols-[3rem_minmax(0,1fr)_3rem_minmax(0,1fr)]" data-testid="diff-row">
        <span class="select-none pr-2.5 text-right tabular-nums" :class="NUMBER_TINT[row.left.kind]">{{ row.left.number }}</span>
        <span class="whitespace-pre-wrap border-r border-border px-3 [overflow-wrap:anywhere]" :class="ROW_TINT[row.left.kind]">
          <DiffCode :parts="row.left.parts" :side="sideOf(row.left.kind)" />
        </span>
        <span class="select-none pr-2.5 text-right tabular-nums" :class="NUMBER_TINT[row.right.kind]">{{ row.right.number }}</span>
        <span class="whitespace-pre-wrap px-3 [overflow-wrap:anywhere]" :class="ROW_TINT[row.right.kind]">
          <DiffCode :parts="row.right.parts" :side="sideOf(row.right.kind)" />
        </span>
      </div>
    </template>
  </div>
</template>
