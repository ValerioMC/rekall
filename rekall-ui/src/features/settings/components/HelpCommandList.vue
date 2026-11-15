<script setup lang="ts">
import { useId } from 'vue'
import type { HelpCommand } from '@/features/settings/model/help-context'
import { helpText, type HelpValues } from '@/features/settings/lib/help-template'
import HelpCalls from '@/features/settings/components/HelpCalls.vue'
import HelpCopyButton from '@/features/settings/components/HelpCopyButton.vue'
import HelpLine from '@/features/settings/components/HelpLine.vue'

defineProps<{ heading: string; commands: readonly HelpCommand[]; values: HelpValues }>()

const headingId = useId()
</script>

<template>
  <section class="flex flex-col gap-1" :aria-labelledby="headingId" data-testid="help-commands">
    <h4 :id="headingId" class="text-small font-semibold text-text-muted">{{ heading }}</h4>
    <ul class="divide-y divide-border border-y border-border">
      <li v-for="command in commands" :key="command.line" class="flex items-start gap-3 py-3" data-testid="help-command">
        <div class="flex min-w-0 flex-1 flex-col gap-1">
          <HelpLine :template="command.line" :values="values" />
          <p class="text-small text-text-muted">{{ command.does }}</p>
          <HelpCalls :names="command.calls" />
        </div>
        <HelpCopyButton :text="helpText(command.line, values)" :label="`Copy ${helpText(command.line, values)}`" />
      </li>
    </ul>
  </section>
</template>
