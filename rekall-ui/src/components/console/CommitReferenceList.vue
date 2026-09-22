<script setup lang="ts">
import { ref } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import { fetchCommitReferenceDiff } from '@/api/commitReference.api'
import CommitDiffView from '@/components/console/CommitDiffView.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { CommitReference } from '@/model/commitReference'

withDefaults(
  defineProps<{
    references: readonly CommitReference[]
    /** Show which step each row belongs to. Off inside a step's own detail, where it's implied. */
    showStepTag?: boolean
    dense?: boolean
  }>(),
  { showStepTag: false, dense: false }
)

const store = useConsoleStore()
const { run } = useAsyncAction()

const expandedId = ref<string | null>(null)
const diffs = ref<Record<string, string | null>>({})
const loadingId = ref<string | null>(null)
const failedId = ref<string | null>(null)
const deletingReference = ref<CommitReference | null>(null)
const togglingContextId = ref<string | null>(null)

async function toggle(reference: CommitReference): Promise<void> {
  if (expandedId.value === reference.id) {
    expandedId.value = null
    return
  }
  expandedId.value = reference.id
  if (reference.id in diffs.value) return

  failedId.value = null
  loadingId.value = reference.id
  try {
    diffs.value = { ...diffs.value, [reference.id]: await fetchCommitReferenceDiff(reference.id) }
  } catch {
    failedId.value = reference.id
  } finally {
    loadingId.value = null
  }
}

/**
 * Chooses the commit for `/rk`, or unchooses it. No toast: the pill changing colour is the
 * confirmation, and a toast per click would drown a person choosing three commits in a row.
 */
async function toggleContext(reference: CommitReference): Promise<void> {
  if (togglingContextId.value) return
  togglingContextId.value = reference.id
  try {
    await run(() => store.setCommitReferenceInContext(reference.id, !reference.inContext))
  } finally {
    togglingContextId.value = null
  }
}

function beginDelete(reference: CommitReference): void {
  deletingReference.value = reference
}

async function confirmDelete(): Promise<void> {
  const reference = deletingReference.value
  if (!reference) return
  await run(() => store.deleteCommitReference(reference.id), 'Commit reference deleted')
  deletingReference.value = null
}
</script>

<template>
  <ul
    v-if="references.length"
    class="min-w-0 space-y-1"
    :class="dense ? 'mt-2' : 'mt-1'"
    data-testid="commit-reference-list"
  >
    <li
      v-for="reference in references"
      :key="reference.id"
      class="group/commit min-w-0 rounded-[var(--radius-control)] border border-transparent px-1.5 py-1 transition-colors hover:border-border hover:bg-surface-raised"
      data-testid="commit-reference-row"
    >
      <div class="flex min-w-0 items-center gap-1">
        <button
          type="button"
          class="focus-ring flex w-full min-w-0 items-center gap-2 rounded text-left"
          :aria-expanded="expandedId === reference.id"
          data-testid="commit-reference-toggle"
          @click="toggle(reference)"
        >
          <svg
            class="size-3 shrink-0 text-text-subtle transition-colors group-hover/commit:text-accent"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
            <path d="M1 6h2.7M8.3 6H11" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
            <circle cx="6" cy="6" r="2.15" stroke="currentColor" stroke-width="1.2" />
          </svg>

          <span
            class="shrink-0 rounded-[6px] border border-border-strong bg-surface px-1.5 py-0.5 font-mono text-[10.5px] text-accent"
            data-testid="commit-reference-hash"
          >
            {{ reference.commitHash.slice(0, 7) }}
          </span>

          <span class="min-w-0 flex-1 truncate text-[12px] text-text-muted" :title="reference.comment">
            {{ reference.comment }}
          </span>

          <span
            v-if="showStepTag && reference.stepTitle"
            class="shrink-0 truncate rounded-full border border-border-strong px-1.5 py-px text-[10px] text-text-subtle"
            data-testid="commit-reference-step-tag"
          >
            {{ reference.stepTitle }}
          </span>
          <span
            v-else-if="showStepTag"
            class="shrink-0 rounded-full border border-dashed border-border-strong px-1.5 py-px text-[10px] text-text-subtle"
          >
            task
          </span>

          <span class="shrink-0 text-[10.5px] text-text-subtle">{{ relativeTime(reference.createdAt) }}</span>
        </button>

        <button
          type="button"
          class="focus-ring flex h-5 shrink-0 items-center gap-1 rounded-[6px] border px-1.5 text-[10px] leading-none transition-all duration-150"
          :class="
            reference.inContext
              ? 'border-anchor-line bg-anchor-soft text-anchor'
              : 'border-transparent text-text-subtle opacity-0 hover:border-anchor-line hover:text-anchor group-hover/commit:opacity-100 focus-visible:opacity-100'
          "
          :aria-pressed="reference.inContext"
          :disabled="togglingContextId === reference.id"
          :title="
            reference.inContext
              ? 'Stop handing this commit to /rk'
              : 'Hand this commit, with its diff, to /rk for this task'
          "
          data-testid="commit-reference-context-toggle"
          :data-in-context="reference.inContext ? 'true' : 'false'"
          @click.stop="toggleContext(reference)"
        >
          <svg class="size-2.5 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path
              d="M4.2 2.4 1.6 6l2.6 3.6M7.8 2.4 10.4 6 7.8 9.6"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          {{ reference.inContext ? 'in context' : 'add to context' }}
        </button>

        <button
          type="button"
          class="focus-ring grid size-5 shrink-0 place-items-center rounded-full text-text-subtle opacity-0 transition-colors hover:bg-danger-soft hover:text-danger group-hover/commit:opacity-100"
          aria-label="Delete this commit reference"
          data-testid="commit-reference-delete"
          @click.stop="beginDelete(reference)"
        >
          <CloseGlyph small />
        </button>
      </div>

      <div v-if="expandedId === reference.id" class="mt-1.5 min-w-0" data-testid="commit-reference-diff">
        <p v-if="loadingId === reference.id" class="px-1 py-1 text-[11px] text-text-subtle">
          Loading the diff…
        </p>
        <p v-else-if="failedId === reference.id" class="px-1 py-1 text-[11px] text-danger">
          Could not load the diff.
        </p>
        <CommitDiffView v-else :diff="diffs[reference.id] ?? null" />
      </div>
    </li>
  </ul>

  <Transition name="dialog">
    <AppConfirm
      v-if="deletingReference"
      title="Delete this commit reference?"
      body="Removes the link between this commit and the task/step it was logged against. The commit itself in git is untouched."
      :blast="`${deletingReference.commitHash.slice(0, 7)} · ${deletingReference.comment} · not recoverable`"
      confirm-label="Delete reference"
      @cancel="deletingReference = null"
      @confirm="confirmDelete"
    />
  </Transition>
</template>
