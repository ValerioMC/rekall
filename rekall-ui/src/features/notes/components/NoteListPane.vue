<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import DescriptionCard from '@/features/catalog/components/DescriptionCard.vue'
import DeviationsCard from '@/features/deviations/components/DeviationsCard.vue'
import InformationList from '@/features/notes/components/InformationList.vue'
import TimeLogDialog from '@/features/time/components/TimeLogDialog.vue'
import StepsCard from '@/features/steps/components/StepsCard.vue'
import EpicDossierCard from '@/features/catalog/components/epic/EpicDossierCard.vue'
import EpicMembershipCard from '@/features/catalog/components/epic/EpicMembershipCard.vue'
import { tasksOfEpic } from '@/features/catalog/model/epic'
import { isEpic } from '@/features/catalog/model/catalog'
import TimerCard from '@/features/time/components/TimerCard.vue'
import WrapupCard from '@/features/wrapup/components/WrapupCard.vue'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'

const store = useConsoleStore()
const {
  selectedTask,
  taskDocuments,
  paneFocus,
  selectedDeviationStep,
  selectedTaskSteps,
  selectedWrapup,
  wrapupIsBehind,
  wrapupMissesSteps,
  selectedTaskEntries,
  runningEntries,
  tasks,
  isLoading
} = storeToRefs(store)
const { run } = useAsyncAction()

const { deviations } = useTaskDeviations(computed(() => selectedTask.value?.id ?? null))

const epicOfSelected = computed(() => {
  const epicId = selectedTask.value?.epicId
  return epicId ? tasks.value.find((task) => task.id === epicId) ?? null : null
})
const selectedEpicChildren = computed(() =>
  selectedTask.value && isEpic(selectedTask.value) ? tasksOfEpic(selectedTask.value.id, tasks.value) : []
)

const showTimeLog = ref(false)
const information = ref<InstanceType<typeof InformationList> | null>(null)
const isRunningHere = computed(() =>
  runningEntries.value.some((entry) => entry.taskId === selectedTask.value?.id)
)

async function startTimer(): Promise<void> {
  if (!selectedTask.value) return
  await run(() => store.startTimer(selectedTask.value!.id))
}

async function pauseTimer(): Promise<void> {
  if (!selectedTask.value) return
  await run(() => store.pauseTimer(selectedTask.value!.id))
}

defineExpose({ addInformation: () => information.value?.add() })
</script>

<template>
  <section
    class="flex min-h-0 w-(--spacing-notelist) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Information on the selected task"
  >
    <header class="flex h-(--spacing-header) shrink-0 items-center gap-2 border-b border-border px-3.5">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-body font-semibold text-text">
          {{ selectedTask?.title ?? 'Information' }}
        </span>
        <span
          v-if="selectedTask"
          class="block truncate font-mono text-micro text-anchor/80"
          :title="selectedTask.anchor"
        >
          {{ selectedTask.anchor }}
        </span>
      </span>
      <span v-if="taskDocuments.length" class="font-mono text-caption text-text-subtle">
        {{ taskDocuments.length }}
      </span>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto p-2">
      <div v-if="isLoading" class="flex flex-col gap-2" aria-hidden="true">
        <div class="skeleton h-14 rounded-control" />
        <div v-for="row in 3" :key="row" class="skeleton h-16 rounded-control" />
      </div>

      <p v-else-if="!selectedTask" class="px-2 py-2 text-small leading-relaxed text-text-subtle">
        Pick a task to see its information.
      </p>

      <template v-else>
        <TimerCard
          :entries="selectedTaskEntries"
          :is-running="isRunningHere"
          @start="startTimer"
          @pause="pauseTimer"
          @open-log="showTimeLog = true"
        />
        <EpicMembershipCard v-if="epicOfSelected" :epic="epicOfSelected" @open="store.selectTask(epicOfSelected.id)" />
        <div class="dossier" data-testid="dossier">
          <EpicDossierCard
            v-if="isEpic(selectedTask)"
            :epic="selectedTask"
            :children="selectedEpicChildren"
            :selected="paneFocus === 'description'"
            @open="store.openDescription()"
          />
          <DescriptionCard
            v-else
            :description="selectedTask.description"
            :selected="paneFocus === 'description'"
            :review-state="
              selectedTask.reviewActive && selectedTask.stepCount === 0
                ? selectedTask.reviewState
                : null
            "
            @open="store.openDescription()"
          />
          <StepsCard
            v-if="!isEpic(selectedTask)"
            class="dossier-seam"
            :steps="selectedTaskSteps"
            :selected="paneFocus === 'steps'"
            @open="store.openSteps()"
          />
          <WrapupCard
            class="dossier-seam"
            :wrapup="selectedWrapup"
            :selected="paneFocus === 'wrapup'"
            :behind="wrapupIsBehind"
            :misses-steps="wrapupMissesSteps"
            @open="store.openWrapup()"
          />

          <DeviationsCard
            :deviations="deviations"
            :open-step="paneFocus === 'deviation' ? selectedDeviationStep : null"
            @open="store.openDeviation"
          />

          <InformationList ref="information" />
        </div>
      </template>
    </div>

    <Transition name="dialog">
      <TimeLogDialog
        v-if="showTimeLog && selectedTask"
        :task="selectedTask"
        :entries="selectedTaskEntries"
        @close="showTimeLog = false"
      />
    </Transition>

  </section>
</template>
