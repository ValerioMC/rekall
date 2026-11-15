<script setup lang="ts">
import { HELP_CONTEXTS, HELP_GROUPS, type HelpContextId } from '@/features/settings/model/help-context'

defineProps<{ topic: HelpContextId }>()
</script>

<template>
  <div class="flex basis-full flex-row flex-wrap gap-1 py-1 min-[860px]:flex-col min-[860px]:gap-2 min-[860px]:pl-3" data-testid="settings-help-topics">
    <div v-for="group in HELP_GROUPS" :key="group.id" class="contents min-[860px]:flex min-[860px]:flex-col min-[860px]:gap-0.5">
      <p class="hidden px-3 text-micro text-text-subtle min-[860px]:block">{{ group.label }}</p>
      <RouterLink
        v-for="context in HELP_CONTEXTS.filter((candidate) => candidate.group === group.id)"
        :key="context.id"
        :to="`/settings/help/${context.id}`"
        class="focus-ring rounded-control px-3 py-1.5 text-small transition-colors duration-(--motion-quick)"
        :class="topic === context.id ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-hover hover:text-text'"
        :aria-current="topic === context.id ? 'page' : undefined"
        :data-testid="`settings-help-topic-${context.id}`"
      >
        {{ context.label }}
      </RouterLink>
    </div>
  </div>
</template>
