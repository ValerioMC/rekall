<script setup lang="ts">
import { computed } from 'vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'

const provider = defineModel<SessionProvider>({ required: true })
const entries = computed<readonly PopoverMenuEntry[]>(() =>
  SESSION_PROVIDERS.map((value) => ({
    kind: 'item',
    id: value,
    label: SESSION_PROVIDER_LABEL[value],
    current: value === provider.value
  }))
)

function choose(value: string): void {
  const choice = SESSION_PROVIDERS.find((candidate) => candidate === value)
  if (choice) provider.value = choice
}
</script>

<template>
  <section class="space-y-3" aria-labelledby="session-provider-label">
    <div>
      <label id="session-provider-label" for="session-provider" class="text-body font-semibold text-text">
        Provider
      </label>
      <p id="session-provider-description" class="mt-0.5 text-small text-text-subtle">
        Choose the provider for new interactive sessions. Each provider keeps its own launch settings.
      </p>
    </div>
    <PopoverMenu class="w-full max-w-xs" label="Provider" :entries="entries" @pick="choose">
      <template #trigger="{ open, toggle }">
        <button
          id="session-provider"
          type="button"
          class="field flex h-(--spacing-control) w-full cursor-pointer items-center justify-between gap-2 rounded-control px-3 text-left text-body text-text"
          aria-haspopup="menu"
          :aria-expanded="open"
          aria-labelledby="session-provider-label session-provider-value"
          aria-describedby="session-provider-description"
          data-popover-trigger
          @click="toggle"
          @keydown.down.prevent="!open && toggle()"
        >
          <span id="session-provider-value">{{ SESSION_PROVIDER_LABEL[provider] }}</span>
          <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="m4 6 4 4 4-4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </template>
    </PopoverMenu>
  </section>
</template>
