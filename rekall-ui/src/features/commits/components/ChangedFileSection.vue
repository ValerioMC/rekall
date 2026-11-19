<script setup lang="ts">
import { computed } from 'vue'
import DiffHunkRows from '@/features/commits/components/DiffHunkRows.vue'
import HunkOrigin from '@/features/commits/components/HunkOrigin.vue'
import { languageOf } from '@/features/commits/lib/code-tokens'
import type { OtherChange, ReviewFile } from '@/features/commits/lib/change-sources'
import type { ChangeScope, DiffLayout } from '@/features/commits/model/changeScope'
import type { TaskStepId } from '@/model/branded'

/**
 * One file of the review: a sticky header with the reader's Viewed mark, then its hunks. In the net
 * view and a step each hunk names the commit that made it; in a single change the file says which
 * other commits touched it.
 */
const props = defineProps<{
  file: ReviewFile
  layout: DiffLayout
  scope: ChangeScope
  viewed: boolean
  shut: boolean
  current: boolean
  otherChanges: readonly OtherChange[]
  stepNumbers: ReadonlyMap<TaskStepId, number>
}>()

const emit = defineEmits<{ toggleViewed: []; toggleShut: []; openSource: [sourceId: string] }>()

const language = computed(() => languageOf(props.file.path))
const directory = computed(() => (props.file.path.includes('/') ? props.file.path.slice(0, props.file.path.lastIndexOf('/') + 1) : ''))
const name = computed(() => props.file.path.slice(directory.value.length))
const namesOrigin = computed(() => props.scope.kind === 'all' || props.scope.kind === 'step')
const KIND_LETTER = { ADDED: 'A', MODIFIED: 'M', DELETED: 'D', RENAMED: 'R' } as const
const KIND_TITLE = { ADDED: 'Added', MODIFIED: 'Modified', DELETED: 'Deleted', RENAMED: 'Renamed' } as const

function stepNumberOf(stepId: TaskStepId | null): number | null {
  return stepId === null ? null : (props.stepNumbers.get(stepId) ?? null)
}
</script>

<template>
  <section
    class="mx-6 mt-5 rounded-card border border-border bg-canvas last:mb-10"
    :aria-label="file.path"
    :data-file-id="file.id"
    :data-current="current"
    data-testid="changed-file"
  >
    <header
      class="sticky top-0 z-10 flex min-h-10 items-center gap-2.5 border-b border-border bg-surface pl-2 pr-3"
      :class="shut ? 'rounded-card border-b-0' : 'rounded-t-card'"
    >
      <button
        type="button"
        class="focus-ring grid size-6 shrink-0 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
        :aria-label="shut ? 'Open the file' : 'Fold the file'"
        :aria-expanded="!shut"
        data-testid="file-fold"
        @click="emit('toggleShut')"
      >
        <svg class="size-3 transition-transform duration-(--motion-quick)" :class="shut ? '' : 'rotate-90'" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <span class="w-3.5 shrink-0 text-center font-mono text-micro text-text-subtle" :title="KIND_TITLE[file.kind]">{{ KIND_LETTER[file.kind] }}</span>
      <span class="min-w-0 flex-1 truncate font-mono text-small text-text">
        <span class="text-text-subtle">{{ directory }}</span
        >{{ name }}
        <span v-if="file.oldPath" class="text-text-subtle"> ← {{ file.oldPath }}</span>
      </span>
      <span class="shrink-0 font-mono text-caption tabular-nums"><span class="text-safe">+{{ file.additions }}</span> <span class="text-danger">−{{ file.deletions }}</span></span>
      <button
        type="button"
        class="focus-ring inline-flex h-6.5 shrink-0 cursor-pointer items-center gap-2 rounded-control border border-border-strong bg-surface-raised px-2.5 text-caption font-medium shadow-key transition-colors duration-(--motion-quick) hover:bg-surface-hover"
        :class="viewed ? 'text-text' : 'text-text-muted'"
        :aria-pressed="viewed"
        title="Your reading marker. It does not accept the work."
        data-testid="file-viewed"
        @click="emit('toggleViewed')"
      >
        <span class="grid size-3.5 place-items-center rounded-key border" :class="viewed ? 'border-text-muted bg-surface-hover text-text' : 'border-border-field text-transparent'" aria-hidden="true">
          <svg class="size-2.5" viewBox="0 0 10 10" fill="none"><path d="m2 5.2 2 2L8 3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </span>
        Viewed
      </button>
    </header>

    <p v-if="viewed && shut" class="px-4 py-2.5 text-caption text-text-subtle" data-testid="file-viewed-note">
      Viewed. {{ file.additions + file.deletions }} changed lines are folded; open the file to read them again.
    </p>

    <template v-if="!shut">
      <p
        v-for="other in otherChanges"
        :key="other.source.id"
        class="flex items-center gap-2 border-b border-border bg-session/10 px-3.5 py-2 text-caption text-text-muted"
        data-testid="file-other-change"
      >
        This file is also changed {{ other.when }} by
        <span class="rounded-control border border-border-strong bg-canvas px-1.5 font-mono text-micro text-text-muted">{{ other.source.commitHash?.slice(0, 7) ?? 'the working tree' }}</span
        >.
        <button type="button" class="focus-ring cursor-pointer rounded-key font-medium text-session-strong underline underline-offset-2" @click="emit('openSource', other.source.id)">Read that change</button>
      </p>
      <p v-if="file.binary" class="px-4 py-3 text-small text-text-subtle" data-testid="file-binary">Binary file: there is no text to compare.</p>
      <p v-else-if="file.hunks.length === 0" class="px-4 py-3 text-small text-text-subtle" data-testid="file-no-hunks">Renamed or re-moded with no change to its lines.</p>
      <div v-for="(attributed, index) in file.hunks" :key="index" data-testid="file-hunk">
        <div class="flex min-h-8 items-center gap-2.5 border-y border-border bg-surface pl-3.5 pr-3" :class="index === 0 ? 'border-t-0' : ''">
          <span class="min-w-0 flex-1 truncate font-mono text-caption text-text-subtle">
            @@ -{{ attributed.hunk.oldStart }},{{ attributed.hunk.oldLines }} +{{ attributed.hunk.newStart }},{{ attributed.hunk.newLines }} @@ {{ attributed.hunk.context }}
          </span>
          <HunkOrigin v-if="namesOrigin" :source="attributed.source" :step-number="stepNumberOf(attributed.source.stepId)" @open="emit('openSource', attributed.source.id)" />
        </div>
        <DiffHunkRows :hunk="attributed.hunk" :layout="layout" :language="language" />
      </div>
    </template>
  </section>
</template>
