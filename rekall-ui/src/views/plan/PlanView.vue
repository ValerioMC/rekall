<script setup lang="ts">
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import PlanStatementCard from '@/components/shared/PlanStatementCard.vue'
import { usePlanStore } from '@/stores/plan.store'
import { useSchemaStore } from '@/stores/schema.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useToastStore } from '@/stores/toast.store'

const plan = usePlanStore()
const schema = useSchemaStore()
const { plan: current, isLoading, isApplying, backfills, confirmations } = storeToRefs(plan)
const { run } = useAsyncAction()
const toast = useToastStore()

onMounted(() => run(() => plan.refresh()))

async function reload(): Promise<void> {
  await run(() => plan.refresh())
}

function onBackfill(key: string, value: string): void {
  plan.setBackfill(key, value)
}

async function onConfirm(key: string, value: boolean): Promise<void> {
  plan.setConfirmation(key, value)
  await reload()
}

async function apply(): Promise<void> {
  const result = await run(() => plan.apply())
  if (!result) return

  // Reported explicitly. Applying is the one irreversible action in the application, and a
  // screen that simply empties out afterwards leaves you guessing whether it ran.
  toast.notify(
    result.documentsDeleted > 0
      ? `Applied ${result.statementCount} statement(s) and removed ${result.documentsDeleted} orphaned document(s).`
      : `Applied ${result.statementCount} statement(s).`
  )
  await run(() => schema.load())
}
</script>

<template>
  <AppPageHeader title="Plan">
    <template #actions>
      <AppBadge v-if="current?.blockedCount" tone="danger" dot>
        {{ current.blockedCount }} refused
      </AppBadge>
      <AppBadge v-if="current?.awaitingInputCount" tone="warn" dot>
        {{ current.awaitingInputCount }} need an answer
      </AppBadge>
      <AppButton variant="primary" :disabled="!current?.applicable" :loading="isApplying" @click="apply">
        Apply
      </AppButton>
    </template>
  </AppPageHeader>

  <div class="mx-auto w-full max-w-[1240px] space-y-4 px-8 pb-20 pt-6">
    <p class="max-w-[74ch] text-[13.5px] leading-relaxed text-text-muted">
      Every difference between your definitions and the live database, in the order it would run. The
      whole plan executes inside a single transaction: if one statement fails, none of them happened.
    </p>

    <AppSkeleton v-if="isLoading && !current" variant="list" :rows="3" />

    <AppEmptyState
      v-else-if="current && !current.statements.length"
      title="Nothing to do"
      description="The database already matches your definitions."
    />

    <PlanStatementCard
      v-for="(statement, index) in current?.statements ?? []"
      :key="`${statement.phase}-${index}`"
      :statement="statement"
      :index="index"
      :backfill="statement.inputKey ? (backfills[statement.inputKey] ?? '') : ''"
      :confirmed="statement.inputKey ? (confirmations[statement.inputKey] ?? false) : false"
      @backfill="onBackfill"
      @confirm="onConfirm"
      @reload="reload"
    />
  </div>
</template>
