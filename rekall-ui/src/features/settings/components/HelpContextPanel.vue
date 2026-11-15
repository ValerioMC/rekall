<script setup lang="ts">
import type { HelpContext } from '@/features/settings/model/help-context'
import type { HelpValues } from '@/features/settings/lib/help-template'
import HelpCommandList from '@/features/settings/components/HelpCommandList.vue'
import HelpExampleList from '@/features/settings/components/HelpExampleList.vue'
import HelpToolList from '@/features/settings/components/HelpToolList.vue'

defineProps<{ context: HelpContext; values: HelpValues; showSummary: boolean }>()
</script>

<template>
  <article class="flex flex-col gap-5" :aria-label="context.label" data-testid="help-panel">
    <header class="flex max-w-[68ch] flex-col gap-1">
      <h3 class="text-title font-semibold text-text">{{ context.label }}</h3>
      <p v-if="showSummary" class="text-body text-text-muted" data-testid="help-summary">{{ context.summary }}</p>
    </header>
    <p v-if="context.id === 'outside'" class="-mt-3 text-small text-text-muted">
      Registration, and its repair, is in
      <RouterLink to="/settings/sessions" class="focus-ring rounded-key text-text underline underline-offset-2 hover:text-accent">Sessions and run queue</RouterLink>.
    </p>
    <HelpCommandList v-if="context.commands.length" :heading="context.commandsHeading" :commands="context.commands" :values="values" />
    <HelpExampleList v-if="context.examples.length" :examples="context.examples" :values="values" />
    <HelpToolList v-if="context.directTools.length" :names="context.directTools" />
  </article>
</template>
