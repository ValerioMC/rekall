<script setup lang="ts">
import { computed, ref } from 'vue'
import SubNav, { type SubNavItem } from '@/components/ui/SubNav.vue'
import HelpCommandList from '@/features/settings/components/HelpCommandList.vue'
import HelpExampleList from '@/features/settings/components/HelpExampleList.vue'
import HelpToolList from '@/features/settings/components/HelpToolList.vue'
import {
  DEFAULT_HELP_CONTEXT,
  HELP_CONTEXTS,
  helpContextOf,
  type HelpContextId
} from '@/features/settings/model/help-context'

/** What a session can do in Rekall, one context at a time: its `/rk` commands, its MCP tools and requests to paste. */
const contextId = ref<HelpContextId>(DEFAULT_HELP_CONTEXT)

const items: readonly SubNavItem<HelpContextId>[] = HELP_CONTEXTS.map(({ id, label }) => ({ id, label }))
const context = computed(() => helpContextOf(contextId.value))
</script>

<template>
  <section class="flex flex-col gap-4" aria-label="Help" data-testid="help-section">
    <h2 class="text-heading font-medium text-text">Help</h2>
    <p class="text-small text-text-muted">
      Pick what you are doing. Commands and requests are templates: put your own labels and task codes in place of
      <code class="font-mono text-anchor">vega</code> and <code class="font-mono text-anchor">RK-12</code>.
    </p>

    <div class="overflow-x-auto" data-testid="help-contexts">
      <SubNav v-model="contextId" :items="items" label="Help context" class="min-w-max" />
    </div>

    <div class="flex flex-col gap-6 pt-1" role="tabpanel" :aria-label="context.label" data-testid="help-panel">
      <div class="flex max-w-[68ch] flex-col gap-2">
        <p class="text-body text-text-muted" data-testid="help-summary">{{ context.summary }}</p>
        <p v-if="context.id === 'outside'" class="text-small text-text-muted">
          Registration, and its repair, is in
          <RouterLink to="/settings/sessions" class="focus-ring rounded-key text-text underline underline-offset-2 hover:text-accent">Sessions and run queue</RouterLink>.
        </p>
      </div>
      <HelpCommandList v-if="context.commands.length" :commands="context.commands" />
      <HelpToolList v-if="context.tools.length" :names="context.tools" />
      <HelpExampleList v-if="context.examples.length" :examples="context.examples" />
    </div>
  </section>
</template>
