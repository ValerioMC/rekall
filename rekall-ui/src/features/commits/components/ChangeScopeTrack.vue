<script setup lang="ts">
import { computed } from 'vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { sameScope, type ChangeScope } from '@/features/commits/model/changeScope'
import type { ChangeSource, RepositoryRef, ScopeGroup } from '@/features/commits/lib/change-sources'

/**
 * What to compare, as one track: all changes, each repository when the task changed more than one,
 * then each step with the commits logged against it, then the working trees.
 */
const props = defineProps<{
  scope: ChangeScope
  repositories: readonly RepositoryRef[]
  groups: readonly ScopeGroup[]
  workingTrees: readonly ChangeSource[]
  /** Reference ids chosen for the `/rk` context: their chip carries a mark. */
  inContext: ReadonlySet<string>
}>()

const emit = defineEmits<{ select: [scope: ChangeScope] }>()

const BUTTON =
  'focus-ring inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-control px-2.5 text-small font-medium whitespace-nowrap transition-colors duration-(--motion-quick)'

const several = computed(() => props.repositories.length > 1)

function classOf(target: ChangeScope): string {
  return sameScope(props.scope, target)
    ? 'bg-surface-raised text-text shadow-key shadow-[inset_0_-2px_0_var(--color-accent)]'
    : 'text-text-muted hover:bg-surface-raised hover:text-text'
}
</script>

<template>
  <div class="flex min-w-0 items-center gap-1 overflow-x-auto rounded-card border border-border bg-canvas p-0.75" role="group" aria-label="What to compare" data-testid="change-scope-track">
    <button type="button" :class="[BUTTON, classOf({ kind: 'all' })]" :aria-pressed="scope.kind === 'all'" title="Everything the sessions changed for this task" data-testid="scope-all" @click="emit('select', { kind: 'all' })">
      All changes
    </button>
    <template v-if="several">
      <span class="mx-1 h-4.5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <button
        v-for="repository in repositories"
        :key="repository.id"
        type="button"
        :class="[BUTTON, classOf({ kind: 'repository', repositoryId: repository.id })]"
        :aria-pressed="sameScope(scope, { kind: 'repository', repositoryId: repository.id })"
        :title="`Everything changed in ${repository.name}`"
        data-testid="scope-repository"
        @click="emit('select', { kind: 'repository', repositoryId: repository.id })"
      >
        {{ repository.name }}
      </button>
    </template>
    <template v-for="group in groups" :key="group.step?.id ?? 'task'">
      <span class="mx-1 h-4.5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <button
        v-if="group.step"
        type="button"
        :class="[BUTTON, classOf({ kind: 'step', stepId: group.step.id })]"
        :aria-pressed="sameScope(scope, { kind: 'step', stepId: group.step.id })"
        :title="group.step.title"
        data-testid="scope-step"
        @click="emit('select', { kind: 'step', stepId: group.step.id })"
      >
        <span class="size-3.5" aria-hidden="true"><StepSeal :state="group.step.state" /></span>
        Step {{ group.step.number }}
      </button>
      <span v-else class="px-2 text-caption text-text-subtle">Task</span>
      <button
        v-for="source in group.sources"
        :key="source.id"
        type="button"
        :class="[BUTTON, classOf({ kind: 'commit', referenceId: source.id })]"
        :aria-pressed="sameScope(scope, { kind: 'commit', referenceId: source.id })"
        :title="source.subject"
        data-testid="scope-commit"
        @click="emit('select', { kind: 'commit', referenceId: source.id })"
      >
        <span class="font-mono text-caption">{{ source.commitHash?.slice(0, 7) }}</span>
        <span
          v-if="several"
          class="max-w-24 truncate text-caption font-normal text-text-subtle"
          data-testid="scope-commit-repository"
          >{{ source.repository.name }}</span
        >
        <span
          v-if="inContext.has(source.id)"
          class="size-1.5 rounded-full bg-anchor"
          title="In the /rk context"
          data-testid="scope-commit-in-context"
        />
      </button>
    </template>
    <template v-if="workingTrees.length > 0">
      <span class="mx-1 h-4.5 w-px shrink-0 bg-border-strong" aria-hidden="true" />
      <button
        v-for="tree in workingTrees"
        :key="tree.id"
        type="button"
        :class="[BUTTON, classOf({ kind: 'working-tree', repositoryId: tree.repository.id })]"
        :aria-pressed="sameScope(scope, { kind: 'working-tree', repositoryId: tree.repository.id })"
        :title="
          several
            ? `What a session has written in ${tree.repository.name} and not committed yet`
            : 'What a session has written and not committed yet'
        "
        data-testid="scope-working-tree"
        @click="emit('select', { kind: 'working-tree', repositoryId: tree.repository.id })"
      >
        <span class="size-1.5 rounded-full bg-session motion-safe:animate-pulse" aria-hidden="true" />
        Working tree
        <span v-if="several" class="max-w-24 truncate text-caption font-normal text-text-subtle">{{
          tree.repository.name
        }}</span>
      </button>
    </template>
  </div>
</template>
