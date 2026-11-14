<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { relativeTime } from '@/common/format/relative-time'
import CommitContextToggle from '@/features/commits/components/CommitContextToggle.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { CommitReference } from '@/features/commits/model/commitReference'

withDefaults(
  defineProps<{
    references: readonly CommitReference[]
    /** Show which step each row belongs to. Off inside a step's own detail, where it's implied. */
    showStepTag?: boolean
    dense?: boolean
  }>(),
  { showStepTag: false, dense: false }
)

const router = useRouter()
const store = useConsoleStore()
const { run } = useAsyncAction()

const deletingReference = ref<CommitReference | null>(null)

function openInChanges(reference: CommitReference): void {
  void router.push({ name: 'task', params: { id: reference.taskId, tab: 'changes' }, query: { commit: reference.id } })
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
      v-for="(reference, index) in references"
      :key="reference.id"
      class="group/commit relative min-w-0 rounded-control border border-transparent px-1.5 py-1 transition-colors hover:border-border hover:bg-surface-raised"
      data-testid="commit-reference-row"
    >
      <span
        v-if="index > 0"
        class="pointer-events-none absolute left-[11.25px] top-[-6px] h-[19px] w-[1.5px] bg-border-strong"
        aria-hidden="true"
        data-testid="commit-rail-above"
      />
      <span
        v-if="index < references.length - 1"
        class="pointer-events-none absolute bottom-0 left-[11.25px] top-[21px] w-[1.5px] bg-border-strong"
        aria-hidden="true"
        data-testid="commit-rail-below"
      />
      <div class="flex h-[26px] min-w-0 items-center gap-1">
        <button
          type="button"
          class="focus-ring flex h-full w-full min-w-0 items-center gap-2 rounded-key text-left"
          title="Read this commit in the Changes tab"
          data-testid="commit-reference-open"
          @click="openInChanges(reference)"
        >
          <svg
            class="size-3 shrink-0 transition-colors"
            :class="reference.inContext ? 'text-anchor' : 'text-text-subtle group-hover/commit:text-accent'"
            viewBox="0 0 12 12"
            fill="none"
            aria-hidden="true"
          >
            <circle cx="6" cy="6" r="3.2" stroke="currentColor" stroke-width="1.2" />
          </svg>

          <span
            class="shrink-0 rounded-control border border-border-strong bg-surface px-1.5 py-0.5 font-mono text-micro text-accent"
            data-testid="commit-reference-hash"
          >
            {{ reference.commitHash.slice(0, 7) }}
          </span>

          <span class="min-w-0 flex-1 truncate text-small text-text-muted" :title="reference.comment">
            {{ reference.comment }}
          </span>

          <span
            v-if="showStepTag && reference.stepTitle"
            class="shrink-0 truncate rounded-key border border-border-strong px-1.5 py-px text-micro text-text-subtle"
            data-testid="commit-reference-step-tag"
          >
            {{ reference.stepTitle }}
          </span>
          <span
            v-else-if="showStepTag"
            class="shrink-0 rounded-key border border-dashed border-border-strong px-1.5 py-px text-micro text-text-subtle"
          >
            task
          </span>

          <span class="shrink-0 text-micro text-text-subtle">{{ relativeTime(reference.createdAt) }}</span>
        </button>

        <CommitContextToggle :reference="reference" />

        <button
          type="button"
          class="focus-ring hit-target relative grid size-5 shrink-0 place-items-center rounded-full text-text-subtle opacity-0 transition-colors hover:bg-danger-soft hover:text-danger group-hover/commit:opacity-100"
          aria-label="Delete this commit reference"
          data-testid="commit-reference-delete"
          @click.stop="beginDelete(reference)"
        >
          <CloseGlyph small />
        </button>
      </div>

    </li>
  </ul>

  <Teleport to="body">
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
  </Teleport>
</template>
