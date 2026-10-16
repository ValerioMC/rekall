<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import ObsoleteCard from './ObsoleteCard.vue'
import PromotionCard from './PromotionCard.vue'
import { groupName, type MemoryGroup } from '@/features/claude-resources/model/claude-resources'
import type { MemoryAnalysis } from '@/features/claude-resources/model/memory-analysis'

/**
 * What reading the memories against each other and against the instructions found. Suggestions
 * only: nothing here writes a file, the person opens one and decides.
 */
const props = defineProps<{ analysis: MemoryAnalysis | null; analysing: boolean; groups: readonly MemoryGroup[] }>()

defineEmits<{ open: [path: string]; rerun: [] }>()

const groupNames = computed(() => new Map(props.groups.map((group) => [group.id, groupName(group)])))
const nothingFound = computed(() => props.analysis !== null && props.analysis.promotions.length === 0 && props.analysis.obsolete.length === 0)
</script>

<template>
  <section class="flex min-h-0 min-w-0 flex-1 flex-col bg-canvas" aria-label="Memory review">
    <header class="texture-dots flex shrink-0 items-start gap-4 border-b border-border px-6 py-4">
      <div class="min-w-0 flex-1">
        <h2 class="text-heading font-semibold tracking-[-0.015em] text-text">Memory review</h2>
        <p class="mt-1 max-w-prose text-small leading-[1.5] text-text-subtle">
          <template v-if="analysis">Read {{ analysis.memoriesRead }} memories against {{ analysis.documentsRead }} instructions and skills. </template>
          Matches come from shared wording, so treat them as suggestions: nothing changes until you edit a file.
        </p>
      </div>
      <AppButton size="md" variant="secondary" :loading="analysing" data-testid="review-rerun" @click="$emit('rerun')">Read again</AppButton>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5" :aria-busy="analysing">
      <AppSkeleton v-if="!analysis" variant="list" :rows="4" />

      <div v-else-if="nothingFound" class="grid place-items-center py-16" data-testid="review-empty">
        <AppEmptyState title="Nothing stands out" description="No lesson is repeated across projects, and no CLAUDE.md or skill already says what a memory does." />
      </div>

      <div v-else class="mx-auto flex max-w-3xl flex-col gap-8">
        <section aria-labelledby="review-promote">
          <div class="mb-3 flex items-baseline gap-2">
            <h3 id="review-promote" class="text-title font-semibold text-text">Worth promoting</h3>
            <span class="font-mono text-caption tabular-nums text-text-subtle">{{ analysis.promotions.length }}</span>
          </div>
          <p class="mb-3 max-w-prose text-small text-text-subtle">The same lesson written down in more than one project. Write it once where every session reads it.</p>
          <div v-if="analysis.promotions.length > 0" class="flex flex-col gap-3">
            <PromotionCard v-for="candidate in analysis.promotions" :key="candidate.memories[0]?.path" :candidate="candidate" :group-names="groupNames" @open="$emit('open', $event)" />
          </div>
          <p v-else class="rounded-control border border-dashed border-border px-3 py-4 text-small text-text-subtle">No lesson is repeated across projects.</p>
        </section>

        <section aria-labelledby="review-covered">
          <div class="mb-3 flex items-baseline gap-2">
            <h3 id="review-covered" class="text-title font-semibold text-text">Probably already covered</h3>
            <span class="font-mono text-caption tabular-nums text-text-subtle">{{ analysis.obsolete.length }}</span>
          </div>
          <p class="mb-3 max-w-prose text-small text-text-subtle">A CLAUDE.md or skill that Claude loads anyway repeats most of the memory. Check, then delete the memory.</p>
          <div v-if="analysis.obsolete.length > 0" class="flex flex-col gap-3">
            <ObsoleteCard
              v-for="finding in analysis.obsolete"
              :key="finding.memory.path"
              :finding="finding"
              :group-name="groupNames.get(finding.memory.groupId) ?? finding.memory.groupId"
              @open="$emit('open', $event)"
            />
          </div>
          <p v-else class="rounded-control border border-dashed border-border px-3 py-4 text-small text-text-subtle">No memory repeats a CLAUDE.md or skill.</p>
        </section>
      </div>
    </div>
  </section>
</template>
