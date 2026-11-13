<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import WorkProjectGroup from '@/features/work/components/WorkProjectGroup.vue'
import WorkSortKey from '@/features/work/components/WorkSortKey.vue'
import { useNow } from '@/composables/useNow'
import { useConsoleStore } from '@/stores/console.store'
import { useWorkStore } from '@/features/work/store/work.store'
import type { TaskId } from '@/model/branded'

const MINUTE_MS = 60_000

const store = useConsoleStore()
const work = useWorkStore()
const router = useRouter()
const { projects } = storeToRefs(store)
const { view, density, sort, groups, unfoldedFiled } = storeToRefs(work)
const now = useNow(MINUTE_MS)

const nothingWaits = computed(() => view.value === 'needs' && work.query.trim() === '')
const noProject = computed(() => projects.value.length === 0)

function openTask(taskId: TaskId): void {
  store.selectTask(taskId)
  void router.push({ name: 'console' })
}
</script>

<template>
  <div data-testid="work-ledger">
    <div
      class="ledger-columns sticky top-0 z-(--z-sticky) h-9 border-b border-border bg-canvas text-caption text-text-subtle"
      role="row"
    >
      <span />
      <WorkSortKey label="Task" :active="sort === 'task'" @sort="sort = 'task'" />
      <span>Status</span>
      <span>Plan</span>
      <WorkSortKey label="Needs you" :active="sort === 'needs'" @sort="sort = 'needs'" />
      <span class="text-right max-[1180px]:hidden">Notes</span>
      <WorkSortKey class="max-[1180px]:hidden" label="Tokens" align-end :active="sort === 'tokens'" @sort="sort = 'tokens'" />
      <WorkSortKey label="Updated" align-end :active="sort === 'updated'" @sort="sort = 'updated'" />
    </div>

    <WorkProjectGroup
      v-for="group in groups"
      :key="group.projectId"
      :group="group"
      :density="density"
      :now="now"
      :filed-unfolded="unfoldedFiled.has(group.projectId)"
      @open="openTask"
      @toggle-filed="work.toggleFiled(group.projectId)"
    />

    <div
      v-if="groups.length === 0 && noProject"
      class="px-4 py-16 text-center text-body text-text-subtle"
      data-testid="work-no-project"
    >
      No project here yet. Add one from the scope picker, then a task can live in it.
    </div>
    <div
      v-else-if="groups.length === 0 && nothingWaits"
      class="grid justify-items-center gap-3 py-16 text-center"
      data-testid="work-nothing-waits"
    >
      <span class="size-14"><StepSeal state="DONE" complete arriving /></span>
      <p class="text-heading font-medium text-text">Nothing waits for you.</p>
      <p class="text-body text-text-muted">Every claimed step is accepted. Resume a session to get more.</p>
    </div>
    <div v-else-if="groups.length === 0" class="px-4 py-16 text-center text-body text-text-subtle" data-testid="work-no-match">
      <b class="block text-title font-semibold text-text">No task matches.</b>
      Clear the filter or change the scope.
    </div>
  </div>
</template>
