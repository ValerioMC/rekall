<script setup lang="ts">
import { computed } from 'vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { useSessionProviderOptions } from '@/features/claude/composables/useSessionProviderOptions'
import type { TaskId } from '@/model/branded'

const props = defineProps<{
  taskId: TaskId | null
  queueing?: boolean
  busy?: boolean
}>()

const emit = defineEmits<{ choose: [provider: SessionProvider] }>()

const providerOptions = useSessionProviderOptions(() => props.taskId, () => props.queueing ?? false)
const options = computed(providerOptions.options)
const initial = computed(() =>
  options.value.find((option) => option.preferred && !option.unavailable) ??
  options.value.find((option) => !option.unavailable) ??
  options.value[0]
)

function choose(provider: SessionProvider): void {
  if (!props.busy && !providerOptions.missing(provider)) emit('choose', provider)
}
</script>

<template>
  <button
    v-for="option in options"
    :key="option.provider"
    type="button"
    role="menuitem"
    tabindex="-1"
    class="focus-ring flex w-full items-start gap-2 rounded-control px-2.5 py-2 text-left hover:bg-surface"
    :class="option.unavailable || busy ? 'cursor-default' : 'cursor-pointer'"
    :aria-disabled="Boolean(option.unavailable || busy) || undefined"
    :data-menu-initial="option === initial ? '' : undefined"
    :data-testid="`session-provider-${option.provider}`"
    @click="choose(option.provider)"
  >
    <SessionProviderMark class="mt-0.5" :provider="option.provider" :size="14" />
    <span class="min-w-0 flex-1">
      <span class="flex flex-wrap items-baseline justify-between gap-x-2">
        <span class="text-small font-medium" :class="option.unavailable ? 'text-text-muted' : 'text-text'">
          {{ SESSION_PROVIDER_LABEL[option.provider] }}
        </span>
        <span v-if="option.preferred" class="text-caption text-text-subtle">Preferred</span>
      </span>
      <span v-if="option.unavailable" class="mt-0.5 block text-caption leading-snug text-text-muted">
        {{ option.unavailable }}
      </span>
      <span v-if="option.detail" class="mt-0.5 block text-caption leading-snug text-text-muted">
        {{ option.detail }}
      </span>
    </span>
  </button>
  <RouterLink
    v-if="options.some((option) => option.unavailable)"
    to="/settings/sessions"
    role="menuitem"
    tabindex="-1"
    class="focus-ring block rounded-control px-2.5 py-2 text-small text-text-muted hover:bg-surface hover:text-text"
  >
    Agent setup in Settings
  </RouterLink>
</template>
