<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import ProjectTaskGroup from '@/features/project/components/ProjectTaskGroup.vue'
import ProjectTaskPager from '@/features/project/components/ProjectTaskPager.vue'
import ProjectTaskSortKey from '@/features/project/components/ProjectTaskSortKey.vue'
import { useNow } from '@/composables/useNow'
import { useConsoleStore } from '@/stores/console.store'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'

const MINUTE_MS = 60_000

const store = useConsoleStore()
const projectTasks = useProjectTasksStore()
const { projects } = storeToRefs(store)
const { view, density, sort, groups, unfoldedFiled, foldedEpics, projectInView, paging } = storeToRefs(projectTasks)
const ledger = ref<HTMLElement | null>(null)
const now = useNow(MINUTE_MS)

const nothingWaits = computed(() => view.value === 'needs' && projectTasks.query.trim() === '')
const noProject = computed(() => projects.value.length === 0)

async function showFirstRows(): Promise<void> {
  await nextTick()
  ledger.value?.scrollIntoView?.({ block: 'start' })
}

watch(() => paging.value.page, showFirstRows)
</script>

<template>
  <div ref="ledger" data-testid="task-ledger">
    <div
      class="ledger-columns sticky top-0 z-(--z-sticky) h-9 border-b border-border bg-canvas text-caption text-text-subtle"
      role="row"
    >
      <span />
      <span />
      <ProjectTaskSortKey label="Task" :active="sort === 'task'" @sort="sort = 'task'" />
      <span>Title</span>
      <span>Status</span>
      <span>Plan</span>
      <ProjectTaskSortKey label="Needs you" :active="sort === 'needs'" @sort="sort = 'needs'" />
      <span class="text-right max-[1180px]:hidden">Notes</span>
      <ProjectTaskSortKey class="max-[1180px]:hidden" label="Tokens" align-end :active="sort === 'tokens'" @sort="sort = 'tokens'" />
      <ProjectTaskSortKey label="Updated" align-end :active="sort === 'updated'" @sort="sort = 'updated'" />
      <span />
    </div>

    <ProjectTaskGroup
      v-for="group in groups"
      :key="group.projectId"
      :group="group"
      :shows-header="projectInView === null"
      :density="density"
      :now="now"
      :filed-unfolded="unfoldedFiled.has(group.projectId)"
      :folded-epics="foldedEpics"
      @toggle-filed="projectTasks.toggleFiled(group.projectId)"
      @toggle-epic="projectTasks.toggleEpic"
    />

    <ProjectTaskPager :paging="paging" @go="projectTasks.goToPage" />

    <div
      v-if="groups.length === 0 && noProject"
      class="px-4 py-16 text-center text-body text-text-subtle"
      data-testid="task-no-project"
    >
      No project here yet. Add one from the scope picker, then a task can live in it.
    </div>
    <div
      v-else-if="groups.length === 0 && nothingWaits"
      class="grid justify-items-center gap-3 py-16 text-center"
      data-testid="task-nothing-waits"
    >
      <span class="size-14"><StepSeal state="DONE" complete arriving /></span>
      <p class="text-heading font-medium text-text">Nothing waits for you.</p>
      <p class="text-body text-text-muted">Every claimed step is accepted. Resume a session to get more.</p>
    </div>
    <div v-else-if="groups.length === 0" class="px-4 py-16 text-center text-body text-text-subtle" data-testid="task-no-match">
      <b class="block text-title font-semibold text-text">No task matches.</b>
      Clear the filter or change the scope.
    </div>
  </div>
</template>
