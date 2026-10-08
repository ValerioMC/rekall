<script setup lang="ts">
/**
 * How and when the run queue runs: the start, the usage ceiling and what each session is opened
 * with. It edits the panel's draft through its models; saving and starting stay with the panel.
 */
import { computed, onMounted, ref } from 'vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CeilingGauge from '@/components/queue/CeilingGauge.vue'
import { formatResetIn } from '@/common/format/countdown'
import { fetchClaudeUsage } from '@/api/claude.api'
import {
  CLAUDE_EFFORT_CHOICES,
  CLAUDE_MODEL_CHOICES,
  claudeEffortChoiceLabel,
  claudeModelChoiceLabel,
  type ClaudeUsageLimit
} from '@/model/claude'

defineProps<{ now: number; startIso: string | null; startInPast: boolean }>()

const when = defineModel<'now' | 'later'>('when', { required: true })
const startLocal = defineModel<string>('startLocal', { required: true })
const ceilingOn = defineModel<boolean>('ceilingOn', { required: true })
const ceiling = defineModel<number>('ceiling', { required: true })
const model = defineModel<string>('model', { required: true })
const effort = defineModel<string>('effort', { required: true })
const skipPermissions = defineModel<boolean>('skipPermissions', { required: true })

// Two selects share a 300px column, so 'Account default' is shortened to fit; the heading says whose.
const shortDefault = (value: string, label: string): string => (value === 'default' ? 'Default' : label)
const MODEL_OPTIONS = CLAUDE_MODEL_CHOICES.map((value) => ({ value, label: shortDefault(value, claudeModelChoiceLabel(value)) }))
const EFFORT_OPTIONS = CLAUDE_EFFORT_CHOICES.map((value) => ({ value, label: shortDefault(value, claudeEffortChoiceLabel(value)) }))

const usageLimits = ref<readonly ClaudeUsageLimit[]>([])

/** The windows the server checks the ceiling against: session, weekly, and the model's own week. */
const watchedWindows = computed(() =>
  usageLimits.value.filter(
    (limit) =>
      limit.key === 'session' ||
      limit.key === 'weekly_all' ||
      (model.value !== 'default' && limit.key === `weekly_${model.value}`)
  )
)

/** The gauge is a guide, not a gate: without a reading it draws no windows rather than an error. */
async function loadUsage(): Promise<void> {
  try {
    const reading = await fetchClaudeUsage()
    usageLimits.value = reading.status === 'OK' ? reading.limits : []
  } catch {
    usageLimits.value = []
  }
}

onMounted(() => void loadUsage())
</script>

<template>
  <aside class="flex min-h-0 flex-col gap-6 overflow-y-auto border-t border-border px-5 py-4 md:border-l md:border-t-0">
    <section aria-labelledby="queue-when-heading">
      <h3 id="queue-when-heading" class="section-label mb-2">When</h3>
      <div class="relative flex gap-0.5 rounded-control bg-surface p-0.5" role="radiogroup" aria-label="When the queue starts">
        <button
          v-for="choice in (['now', 'later'] as const)"
          :key="choice"
          type="button"
          role="radio"
          :aria-checked="when === choice"
          class="focus-ring h-7 flex-1 rounded-control text-small transition-colors"
          :class="when === choice ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
          :data-testid="`run-queue-when-${choice}`"
          @click="when = choice"
        >
          {{ choice === 'now' ? 'Now' : 'At a time' }}
        </button>
      </div>
      <div v-if="when === 'later'" class="mt-2.5">
        <label for="queue-start-at" class="sr-only">Start time</label>
        <input
          id="queue-start-at"
          v-model="startLocal"
          type="datetime-local"
          class="field h-(--spacing-control) w-full rounded-control px-3 font-mono text-small text-text [color-scheme:dark]"
          :aria-invalid="startInPast"
          data-testid="run-queue-start-at"
        />
        <p class="mt-1.5 text-caption" :class="startInPast ? 'text-danger' : 'text-text-subtle'">
          <template v-if="startInPast">That time has passed. Pick a later one.</template>
          <template v-else>In {{ formatResetIn(startIso, now) }}. Rekall has to be open then.</template>
        </p>
      </div>
    </section>

    <section aria-labelledby="queue-ceiling-heading">
      <div class="mb-1 flex items-center justify-between gap-2">
        <h3 id="queue-ceiling-heading" class="section-label">Usage ceiling</h3>
        <AppCheckbox v-model="ceilingOn" label="On" data-testid="run-queue-ceiling-on" />
      </div>
      <p class="mb-2 text-caption leading-relaxed text-text-muted">
        <template v-if="ceilingOn">
          No new task or step starts once a window reaches the line. The queue waits for that
          window to reset, then carries on.
        </template>
        <template v-else>Off: the queue runs until it is empty, whatever the usage.</template>
      </p>
      <CeilingGauge v-model="ceiling" :windows="watchedWindows" :disabled="!ceilingOn" :now="now" />
    </section>

    <section aria-labelledby="queue-session-heading" class="flex flex-col gap-3">
      <h3 id="queue-session-heading" class="section-label">Sessions</h3>
      <div class="grid grid-cols-2 gap-2">
        <div>
          <label for="queue-model" class="mb-1 block text-caption text-text-subtle">Model</label>
          <AppSelect id="queue-model" v-model="model" :options="MODEL_OPTIONS" />
        </div>
        <div>
          <label for="queue-effort" class="mb-1 block text-caption text-text-subtle">Effort</label>
          <AppSelect id="queue-effort" v-model="effort" :options="EFFORT_OPTIONS" />
        </div>
      </div>
      <div>
        <AppCheckbox v-model="skipPermissions" label="Skip permission prompts" data-testid="run-queue-skip" />
        <p class="mt-1.5 text-caption leading-relaxed" :class="skipPermissions ? 'text-text-subtle' : 'text-warn'">
          <template v-if="skipPermissions">Sessions run with <span class="font-mono">--dangerously-skip-permissions</span>.</template>
          <template v-else>Off: an unattended session stops at its first permission prompt and waits for you.</template>
        </p>
      </div>
    </section>
  </aside>
</template>
