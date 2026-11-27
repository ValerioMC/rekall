<script setup lang="ts">
import { computed, ref } from 'vue'
import { preferredProvider, setPreferredProvider } from '@/common/config/session-launch'
import { useAnchoredMenu } from '@/composables/useAnchoredMenu'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { useSessionProviderOptions } from '@/features/claude/composables/useSessionProviderOptions'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'

const trigger = ref<HTMLButtonElement | null>(null)
const { open, menu, placement, show, toggle, close, handleKeydown } = useAnchoredMenu(() => trigger.value, 224)
const { options, missing } = useSessionProviderOptions(() => null)
const provider = computed(preferredProvider)
const label = computed(() => SESSION_PROVIDER_LABEL[provider.value])
const unavailable = computed(() => missing(provider.value))
const providers = computed(options)
const needsSetup = computed(() => providers.value.some((option) => option.unavailable))

function choose(candidate: SessionProvider): void {
  if (missing(candidate)) return
  setPreferredProvider(candidate)
  close(true)
}
</script>

<template>
  <button
    ref="trigger"
    type="button"
    class="focus-ring flex h-8 shrink-0 items-center gap-1.5 rounded-control border border-border bg-surface px-2 text-small text-text-muted hover:bg-surface-raised hover:text-text"
    :class="unavailable && 'border-warn text-warn'"
    :aria-label="`Session provider · ${label}${unavailable ? ' unavailable' : ''}`"
    :title="unavailable ?? 'Session provider'"
    aria-haspopup="menu"
    :aria-expanded="open"
    data-testid="header-session-provider"
    @click="toggle"
    @keydown.down.prevent="show"
    @keydown.up.prevent="show"
  >
    <SessionProviderMark :provider="provider" />
    <span class="hidden @min-[25rem]:inline">{{ label }}</span>
    <svg class="size-3.5 shrink-0" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path d="m4 6 4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>

  <Teleport to="body">
    <div
      v-if="open"
      ref="menu"
      role="menu"
      aria-label="Session provider"
      class="fixed z-(--z-popover) overflow-y-auto rounded-card border border-border-strong bg-surface-raised p-1 shadow-lift"
      :style="placement"
      data-anchored-menu
      @keydown="handleKeydown"
    >
      <p class="px-2.5 pt-1.5 pb-1 text-caption text-text-muted">Session provider</p>
      <button
        v-for="option in providers"
        :key="option.provider"
        type="button"
        role="menuitemradio"
        tabindex="-1"
        class="focus-ring flex w-full items-center gap-2 rounded-control px-2.5 py-2 text-left text-small hover:bg-surface"
        :class="option.unavailable ? 'text-text-muted' : 'cursor-pointer text-text'"
        :aria-checked="option.preferred"
        :aria-disabled="Boolean(option.unavailable)"
        :data-menu-initial="option.preferred ? '' : undefined"
        @click="choose(option.provider)"
      >
        <SessionProviderMark :provider="option.provider" />
        <span class="flex-1">
          {{ SESSION_PROVIDER_LABEL[option.provider] }}
          <span v-if="option.unavailable" class="block text-caption">CLI is not installed</span>
        </span>
        <svg v-if="option.preferred" class="size-4 shrink-0 text-accent" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="m3.5 8 3 3 6-6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <template v-if="needsSetup">
        <div role="separator" class="mx-1 my-1 h-px bg-border" />
        <RouterLink
          to="/settings/sessions"
          role="menuitem"
          tabindex="-1"
          class="focus-ring block rounded-control px-2.5 py-2 text-small text-text-muted hover:bg-surface hover:text-text"
          @click="close(false)"
        >
          Agent setup in Settings
        </RouterLink>
      </template>
    </div>
  </Teleport>
</template>
