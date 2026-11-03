<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CheckForm from '@/features/check/components/CheckForm.vue'
import CheckRow from '@/features/check/components/CheckRow.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useChecksStore } from '@/features/check/store/checks.store'
import type { CheckInput, DoneCheck } from '@/features/check/model/check'
import type { DoneCheckId, ProjectId, TaskStepId } from '@/model/branded'

/**
 * The checks of one scope, the project or one step, in the order they run, with the form to add
 * one. Which scope it is decides only where a new check is created.
 */
const props = defineProps<{
  projectId: ProjectId
  stepId: TaskStepId | null
  checks: readonly DoneCheck[]
  emptyText: string
}>()

const store = useChecksStore()
const { run, isRunning } = useAsyncAction()

const adding = ref(false)
const editing = ref<DoneCheckId | null>(null)

async function add(input: CheckInput): Promise<void> {
  const created = await run(() =>
    props.stepId ? store.createForStep(props.projectId, props.stepId, input) : store.createForProject(props.projectId, input)
  )
  if (created) adding.value = false
}

async function edit(id: DoneCheckId, input: CheckInput): Promise<void> {
  const saved = await run(() => store.update(id, input))
  if (saved) editing.value = null
}

function move(check: DoneCheck, direction: 'up' | 'down'): void {
  void run(() => store.move(props.projectId, props.stepId, check.id, direction))
}

function remove(check: DoneCheck): void {
  void run(() => store.remove(props.projectId, check.id), `Deleted ${check.label}`)
}
</script>

<template>
  <div data-testid="check-list">
    <p v-if="!checks.length && !adding" class="py-1 text-small text-text-subtle" data-testid="check-empty">{{ emptyText }}</p>
    <ul v-else class="-mx-2 flex flex-col">
      <template v-for="(check, index) in checks" :key="check.id">
        <li v-if="editing === check.id" class="px-2 py-1">
          <CheckForm :initial="check" :saving="isRunning" @save="edit(check.id, $event)" @cancel="editing = null" />
        </li>
        <CheckRow
          v-else
          :check="check"
          :can-move-up="index > 0"
          :can-move-down="index < checks.length - 1"
          @edit="editing = check.id"
          @up="move(check, 'up')"
          @down="move(check, 'down')"
          @remove="remove(check)"
        />
      </template>
    </ul>

    <CheckForm v-if="adding" class="mt-2" :initial="null" :saving="isRunning" @save="add" @cancel="adding = false" />
    <AppButton v-else class="mt-2" size="sm" variant="quiet" data-testid="check-add" @click="adding = true">Add a check</AppButton>
  </div>
</template>
