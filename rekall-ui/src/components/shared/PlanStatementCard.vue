<script setup lang="ts">
import { computed } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppSqlBlock from '@/components/ui/AppSqlBlock.vue'
import { isDestructive } from '@/model/plan'
import type { ChangeClass, PlanStatement } from '@/model/plan'

const props = defineProps<{
  statement: PlanStatement
  index: number
  backfill: string
  confirmed: boolean
}>()

const emit = defineEmits<{
  backfill: [key: string, value: string]
  confirm: [key: string, value: boolean]
  reload: []
}>()

const TONES = { SAFE: 'safe', NEEDS_INPUT: 'warn', BLOCKED: 'danger' } as const

const tone = computed(() => TONES[props.statement.changeClass satisfies ChangeClass])

const destructive = computed(() => isDestructive(props.statement))
</script>

<template>
  <AppCard
    data-testid="plan-statement"
    class="border-l-2"
    :class="{
      'border-l-safe': statement.changeClass === 'SAFE',
      'border-l-warn': statement.changeClass === 'NEEDS_INPUT',
      'border-l-danger': statement.changeClass === 'BLOCKED'
    }"
  >
    <div class="flex flex-wrap items-center gap-2.5">
      <span class="font-mono text-[11px] text-text-subtle">{{ String(index + 1).padStart(2, '0') }}</span>
      <AppBadge :tone="tone" dot>
        {{ statement.changeClass.toLowerCase().replace('_', ' ') }}
      </AppBadge>
      <strong class="text-[13.5px] font-medium text-text">{{ statement.description }}</strong>
      <div class="flex-1" />
      <AppBadge mono>{{ statement.phase.toLowerCase().replace('_', ' ') }}</AppBadge>
    </div>

    <p v-if="statement.warning" class="mt-3 text-[12.5px] leading-relaxed text-warn">
      {{ statement.warning }}
    </p>

    <div v-if="statement.inputKey && !destructive" class="mt-3.5 flex items-end gap-2">
      <div class="flex-1">
        <label class="mb-1.5 block text-xs font-medium text-text-muted">
          Value to write into existing rows
        </label>
        <AppInput
          :model-value="backfill"
          placeholder="e.g. UNKNOWN"
          @update:model-value="emit('backfill', statement.inputKey!, $event)"
        />
      </div>
      <AppButton size="md" :disabled="!backfill" @click="emit('reload')">Use this</AppButton>
    </div>

    <div v-else-if="statement.inputKey" class="mt-3.5">
      <AppCheckbox
        :id="`confirm-${statement.inputKey}`"
        :model-value="confirmed"
        label="Yes, destroy this data"
        @update:model-value="emit('confirm', statement.inputKey!, $event)"
      />
    </div>

    <AppSqlBlock v-if="statement.sql" :sql="statement.sql" class="mt-3.5" />
  </AppCard>
</template>
