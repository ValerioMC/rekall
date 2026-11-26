<script setup lang="ts">
/**
 * One provider's column. Its header and five body blocks are subgrid rows, so two panels side by
 * side keep connection, model, effort, permissions and run queue on the same lines.
 */
import { useId } from 'vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'

defineProps<{ provider: SessionProvider }>()

const titleId = useId()
</script>

<template>
  <section
    class="row-span-6 grid min-w-0 grid-rows-subgrid gap-y-0 rounded-card border border-border bg-surface shadow-[var(--shadow-raised)]"
    :aria-labelledby="titleId"
  >
    <header class="flex flex-wrap items-center gap-x-2.5 gap-y-2 border-b border-border px-5 py-3">
      <SessionProviderMark :provider="provider" :size="16" />
      <h2 :id="titleId" class="text-title font-semibold tracking-[-0.01em] text-text">
        {{ SESSION_PROVIDER_LABEL[provider] }}
      </h2>
      <slot name="status" />
      <div class="ml-auto flex min-h-7 items-center">
        <slot name="action" />
      </div>
    </header>
    <slot />
  </section>
</template>
