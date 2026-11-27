<script setup lang="ts">
import { computed, nextTick, onMounted, ref, useId, watch } from 'vue'
import { preferredProvider } from '@/common/config/session-launch'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import type { Task } from '@/features/catalog/model/catalog'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { useSessionProviderOptions } from '@/features/claude/composables/useSessionProviderOptions'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useCodexStore } from '@/features/codex/store/codex.store'
import {
  queueEffortOptions,
  queueLaunchDefaults,
  queueLaunchRequest,
  queueModelOptions,
  queueSessionBlocker
} from '@/features/queue/model/queue-launch'
import { isQueued, type QueueTaskRequest } from '@/features/queue/model/runQueue'
import { useRunQueueStore } from '@/features/queue/store/runQueue.store'

const props = defineProps<{
  task: Pick<Task, 'id' | 'anchor' | 'title'>
  mode: 'add' | 'run'
  submit: (launch: QueueTaskRequest) => Promise<void>
}>()
const emit = defineEmits<{ close: []; complete: []; setup: [] }>()

const codex = useCodexStore()
const queue = useRunQueueStore()
const terminals = useTerminalStore()
const { options, missing } = useSessionProviderOptions()
const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null
const title = computed(() => props.mode === 'add' ? 'Add to run queue' : 'Run in queue')
const confirmLabel = computed(() => {
  if (props.mode === 'add') return 'Add to queue'
  if (queue.queue.state === 'IDLE') return 'Add and start queue'
  return queue.queue.state === 'RUNNING' ? 'Add to running queue' : 'Add to queue'
})

const draft = ref(queueLaunchDefaults(preferredProvider()))
const saving = ref(false)
const error = ref<string | null>(null)
const catalog = ref<'idle' | 'loading' | 'ready' | 'failed'>('idle')
const providerMissing = computed(() => missing(draft.value.provider))
const sessionBlocker = computed(() => queueSessionBlocker(terminals.terminalForTask(props.task.id)))
const alreadyQueued = computed(() => isQueued(queue.queue, props.task.id))
const blocked = computed(() => sessionBlocker.value || alreadyQueued.value || providerMissing.value)
const needsCatalog = computed(() => draft.value.provider === 'CODEX' && !blocked.value)
const catalogPending = computed(() => needsCatalog.value && catalog.value !== 'ready')
const choicesUnavailable = computed(() => Boolean(blocked.value) || catalogPending.value)
const fieldsDisabled = computed(() => choicesUnavailable.value || saving.value)
const modelOptions = computed(() => queueModelOptions(draft.value.provider, codex.models))
const effortOptions = computed(() => queueEffortOptions(draft.value, codex.models))
const modelId = useId()
const effortId = useId()
const providerName = useId()
const fieldset = ref<HTMLFieldSetElement | null>(null)
const blockedMessage = ref<HTMLElement | null>(null)
const refusal = ref<HTMLElement | null>(null)

function chooseModel(model: string): void {
  draft.value = { ...draft.value, model }
  if (!effortOptions.value.some(({ value }) => value === draft.value.effort)) {
    draft.value = { ...draft.value, effort: 'default' }
  }
}

function chooseProvider(provider: SessionProvider): void {
  draft.value = queueLaunchDefaults(provider)
  error.value = null
  if (provider === 'CODEX' && catalog.value === 'ready') normalizeModel()
}

function normalizeModel(): void {
  const current = draft.value.model
  chooseModel(modelOptions.value.some(({ value }) => value === current) ? current : 'default')
}

async function loadCatalog(retry = false): Promise<void> {
  if (catalog.value === 'loading') return
  catalog.value = 'loading'
  try {
    await codex.loadModels()
    catalog.value = 'ready'
    if (draft.value.provider !== 'CODEX') return
    normalizeModel()
    if (retry) {
      await nextTick()
      document.getElementById(modelId)?.focus()
    }
  } catch {
    catalog.value = 'failed'
  }
}

watch(needsCatalog, (needed) => {
  if (needed && catalog.value === 'idle') void loadCatalog()
}, { immediate: true })

function dismiss(): void {
  if (saving.value) return
  emit('close')
  void nextTick(() => opener?.focus({ preventScroll: true }))
}

async function confirm(): Promise<void> {
  if (fieldsDisabled.value) return
  saving.value = true
  error.value = null
  try {
    await props.submit(queueLaunchRequest(props.task.id, draft.value))
    emit('complete')
    if (props.mode === 'add') await nextTick(() => opener?.focus({ preventScroll: true }))
  } catch (failure: unknown) {
    error.value = failure instanceof Error ? failure.message : 'Could not add the task to the queue.'
    await nextTick()
    refusal.value?.focus()
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  if (sessionBlocker.value || alreadyQueued.value) {
    blockedMessage.value?.focus()
    return
  }
  const radio = fieldset.value?.querySelector<HTMLInputElement>('input:checked:not(:disabled)')
    ?? fieldset.value?.querySelector<HTMLInputElement>('input:not(:disabled)')
  const target = radio ?? blockedMessage.value
  target?.focus()
})
</script>

<template>
  <Teleport to="body">
    <AppDialog
      :label="title"
      :dismissible="!saving"
      :scrim="mode === 'add' ? 'light' : 'heavy'"
      panel-class="max-w-lg max-h-[calc(100dvh-2.5rem)] overflow-y-auto"
      panel-testid="queue-launch-dialog"
      @dismiss="dismiss"
    >
      <form :aria-busy="saving" @submit.prevent="confirm">
        <header class="flex items-start justify-between gap-3 border-b border-border px-4 py-3">
          <div class="min-w-0">
            <h2 class="text-title font-semibold text-text">{{ title }}</h2>
            <p class="mt-1 flex flex-wrap items-baseline gap-x-2 gap-y-1 text-small text-text-muted">
              <code class="anchor-chip px-1.5 py-0.5">{{ task.anchor }}</code>
              <span class="min-w-0 break-words">{{ task.title }}</span>
            </p>
          </div>
          <AppButton variant="ghost" size="icon" aria-label="Close queue configuration" :disabled="saving" @click="dismiss">
            <CloseGlyph />
          </AppButton>
        </header>

        <div class="grid gap-3.5 p-4">
          <fieldset ref="fieldset" :disabled="saving" class="min-w-0">
            <legend class="mb-1 text-small font-medium text-text">Provider</legend>
            <div class="grid grid-cols-2 gap-2.5">
              <label
                v-for="option in options()"
                :key="option.provider"
                class="grid min-h-9 cursor-pointer grid-cols-[1rem_minmax(0,1fr)] items-center gap-x-2 rounded-control border border-border-strong bg-surface-raised px-2 py-1.5 text-small text-text has-checked:border-border-field has-checked:bg-surface-hover has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-accent has-disabled:cursor-default has-disabled:text-text-muted"
              >
                <input
                  type="radio"
                  :name="providerName"
                  :value="option.provider"
                  :checked="draft.provider === option.provider"
                  :disabled="Boolean(option.unavailable)"
                  class="size-3.5 accent-(--color-accent) focus-visible:outline-none"
                  @change="chooseProvider(option.provider)"
                />
                <span class="flex items-center gap-1.5">
                  <SessionProviderMark :provider="option.provider" />
                  {{ SESSION_PROVIDER_LABEL[option.provider] }}
                </span>
                <span v-if="option.unavailable" class="col-start-2 text-caption">CLI is not installed</span>
              </label>
            </div>
          </fieldset>

          <div class="grid grid-cols-2 gap-2.5">
            <div class="min-w-0">
              <label :for="modelId" class="mb-1 block text-small font-medium text-text">Model</label>
              <AppSelect
                :id="modelId"
                :model-value="choicesUnavailable ? null : draft.model"
                :options="choicesUnavailable ? [] : modelOptions"
                :placeholder="choicesUnavailable ? '—' : undefined"
                :disabled="fieldsDisabled"
                data-testid="queue-launch-model"
                @update:model-value="chooseModel"
              />
            </div>
            <div class="min-w-0">
              <label :for="effortId" class="mb-1 block text-small font-medium text-text">Effort</label>
              <AppSelect
                :id="effortId"
                :model-value="choicesUnavailable ? null : draft.effort"
                :options="choicesUnavailable ? [] : effortOptions"
                :placeholder="choicesUnavailable ? '—' : undefined"
                :disabled="fieldsDisabled || effortOptions.length < 2"
                data-testid="queue-launch-effort"
                @update:model-value="draft.effort = $event"
              />
            </div>
          </div>

          <div
            v-if="choicesUnavailable"
            ref="blockedMessage"
            role="status"
            tabindex="-1"
            class="flex min-h-9 flex-wrap items-center justify-between gap-2 rounded-control border border-border bg-canvas px-2.5 py-1.5 text-small text-text-muted"
          >
            <template v-if="sessionBlocker">{{ sessionBlocker }}</template>
            <template v-else-if="alreadyQueued">This task is already in the run queue.</template>
            <template v-else-if="providerMissing">
              <span>{{ providerMissing }}</span>
              <RouterLink
                to="/settings/sessions"
                class="focus-ring rounded-key text-accent-strong hover:underline"
                @click="emit('setup')"
              >
                Agent setup in Settings
              </RouterLink>
            </template>
            <template v-else-if="catalog === 'failed'">
              <span>Could not load Codex models.</span>
              <AppButton size="sm" @click="loadCatalog(true)">Retry</AppButton>
            </template>
            <template v-else>Loading Codex models…</template>
          </div>

          <p v-if="error" ref="refusal" class="text-small text-danger" role="alert" tabindex="-1">{{ error }}</p>
          <p class="text-small text-text-muted">
            These settings are saved for this task.
            {{ mode === 'add' ? 'The queue starts separately.' : 'Confirming starts the queue if it is idle.' }}
          </p>
        </div>

        <footer class="flex justify-end gap-2 border-t border-border px-4 py-3">
          <AppButton :disabled="saving" @click="dismiss">Cancel</AppButton>
          <AppButton
            type="submit"
            variant="primary"
            :disabled="fieldsDisabled"
            :loading="saving"
            data-testid="queue-launch-confirm"
          >
            {{ confirmLabel }}
          </AppButton>
        </footer>
      </form>
    </AppDialog>
  </Teleport>
</template>
