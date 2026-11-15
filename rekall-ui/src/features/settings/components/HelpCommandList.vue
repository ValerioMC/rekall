<script setup lang="ts">
import type { HelpCommand } from '@/features/settings/model/help-context'
import HelpCopyButton from '@/features/settings/components/HelpCopyButton.vue'

defineProps<{ commands: readonly HelpCommand[] }>()
</script>

<template>
  <section class="flex flex-col gap-2" aria-labelledby="help-commands" data-testid="help-commands">
    <h3 id="help-commands" class="text-title font-semibold text-text">In a session</h3>
    <ul class="divide-y divide-border overflow-hidden rounded-card border border-border bg-surface">
      <li v-for="command in commands" :key="command.line" class="flex items-start gap-3 px-4 py-2.5" data-testid="help-command">
        <div class="min-w-0 flex-1">
          <code class="block break-words font-mono text-small text-anchor">{{ command.line }}</code>
          <p class="mt-0.5 text-small text-text-muted">{{ command.does }}</p>
        </div>
        <HelpCopyButton :text="command.line" :label="`Copy ${command.line}`" />
      </li>
    </ul>
  </section>
</template>
