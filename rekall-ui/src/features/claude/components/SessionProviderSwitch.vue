<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { useRunProvider } from '@/features/claude/composables/useRunProvider'
import { useSessionProviderOptions } from '@/features/claude/composables/useSessionProviderOptions'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { useTaskProviderStore } from '@/features/claude/store/taskProvider.store'
import type { TaskId } from '@/model/branded'

/**
 * The two provider marks beside Run here: the lit one is what the key and its menu start. It takes
 * no press while `busy` or while a live session holds the task.
 */
const props = defineProps<{ taskId: TaskId; busy: boolean }>()

const chosen = useTaskProviderStore()
const { provider, isHeld } = useRunProvider(() => props.taskId)
const providerOptions = useSessionProviderOptions(() => props.taskId)
const group = ref<HTMLElement | null>(null)

const options = computed(() =>
  providerOptions.options().map((option) => {
    const checked = option.provider === provider.value
    return {
      provider: option.provider,
      checked,
      disabled: !checked && (props.busy || isHeld.value || option.unavailable !== null),
      title: option.unavailable ?? option.detail ?? SESSION_PROVIDER_LABEL[option.provider]
    }
  })
)

function choose(candidate: SessionProvider): void {
  const option = options.value.find((entry) => entry.provider === candidate)
  if (option && !option.checked && !option.disabled) chosen.choose(props.taskId, candidate)
}

async function chooseOther(event: KeyboardEvent): Promise<void> {
  event.preventDefault()
  event.stopPropagation()
  const other = SESSION_PROVIDERS.find((candidate) => candidate !== provider.value)
  if (other) choose(other)
  await nextTick()
  group.value?.querySelector<HTMLElement>('[aria-checked="true"]')?.focus()
}
</script>

<template>
  <div
    ref="group"
    role="radiogroup"
    aria-label="Provider"
    class="inline-flex h-7 shrink-0 items-center gap-0.5 rounded-control bg-surface p-0.5 ring-1 ring-inset ring-border-strong"
    data-testid="provider-switch"
    @keydown.left="chooseOther"
    @keydown.right="chooseOther"
    @keydown.up="chooseOther"
    @keydown.down="chooseOther"
  >
    <button
      v-for="option in options"
      :key="option.provider"
      type="button"
      role="radio"
      class="focus-ring grid h-6 w-7 place-items-center rounded-key transition-colors"
      :class="
        option.checked
          ? 'cursor-default bg-surface-hover shadow-key ring-1 ring-inset ring-border-field'
          : option.disabled
            ? 'cursor-default [&>svg]:opacity-25 [&>svg]:grayscale'
            : 'cursor-pointer [&>svg]:opacity-45 [&>svg]:grayscale hover:bg-surface-raised hover:[&>svg]:opacity-80'
      "
      :tabindex="option.checked ? 0 : -1"
      :aria-checked="option.checked"
      :aria-disabled="option.disabled || undefined"
      :aria-label="SESSION_PROVIDER_LABEL[option.provider]"
      :title="option.title"
      :data-testid="`provider-switch-${option.provider}`"
      @click="choose(option.provider)"
    >
      <SessionProviderMark :provider="option.provider" :size="14" />
    </button>
  </div>
</template>
