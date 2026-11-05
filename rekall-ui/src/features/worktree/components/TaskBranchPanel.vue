<script setup lang="ts">
import { computed, watch } from 'vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { aheadText, mergeText, worktreeText } from '@/features/worktree/lib/taskBranch'
import { useTaskBranchStore } from '@/features/worktree/store/taskBranch.store'
import type { TaskId } from '@/model/branded'

/**
 * The branch a task's work lives on when the run queue gave it a worktree: what it holds and
 * whether it merges. Accepting the task merges the branch and removes the worktree, so the panel
 * offers no action. Drawn only while the branch exists. `signature` changes whenever the task's
 * work moves, so the figures are read again.
 */
const props = defineProps<{ taskId: TaskId; signature: string }>()

const store = useTaskBranchStore()
const { run } = useAsyncAction()

const branch = computed(() => store.of(props.taskId))

watch(
  () => [props.taskId, props.signature],
  () => void run(() => store.load(props.taskId)),
  { immediate: true }
)
</script>

<template>
  <section v-if="branch" class="rounded-card border border-border bg-surface px-4 py-3" aria-label="Branch" data-testid="task-branch">
    <div class="flex flex-wrap items-baseline gap-x-3">
      <span class="eyebrow">Branch</span>
      <span class="font-mono text-small text-anchor" data-testid="branch-name">{{ branch.branch }}</span>
      <span class="text-caption text-text-subtle" data-testid="branch-ahead">{{ aheadText(branch) }}</span>
    </div>

    <p
      class="mt-2 text-small"
      :class="branch.mergesCleanly === false ? 'text-danger' : 'text-text-muted'"
      data-testid="branch-merge"
    >
      <span aria-hidden="true">{{ branch.mergesCleanly === false ? '✕' : branch.mergesCleanly ? '✓' : '?' }}</span>
      {{ mergeText(branch) }}
    </p>

    <details v-if="branch.changedFiles.length" class="mt-1">
      <summary class="cursor-pointer text-caption text-text-subtle" data-testid="branch-files-toggle">Changed files</summary>
      <ul class="mt-1 font-mono text-caption text-text-muted" data-testid="branch-files">
        <li v-for="file in branch.changedFiles" :key="file">{{ file }}</li>
      </ul>
    </details>

    <p class="mt-2 text-caption text-text-subtle" data-testid="branch-worktree">{{ worktreeText(branch) }}</p>
  </section>
</template>
