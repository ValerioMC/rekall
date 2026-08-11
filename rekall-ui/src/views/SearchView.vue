<script setup lang="ts">
import { ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import AppPageHeader from '@/components/ui/AppPageHeader.vue'
import { searchDocuments } from '@/api/documents.api'
import { useAsyncAction } from '@/composables/useAsyncAction'
import type { RekallDocument } from '@/model/catalog'

const { run, isRunning } = useAsyncAction()

const query = ref('')
const results = ref<readonly RekallDocument[] | null>(null)

async function search(): Promise<void> {
  if (!query.value.trim()) return
  const found = await run(() => searchDocuments(query.value))
  if (found) results.value = found
}

/** A few lines around the match, so a hit is readable without opening the note. */
function excerpt(document: RekallDocument): string {
  const index = document.bodyMarkdown.toLowerCase().indexOf(query.value.trim().toLowerCase())
  if (index < 0) return document.bodyMarkdown.slice(0, 240)
  const start = Math.max(0, index - 80)
  return (start > 0 ? '…' : '') + document.bodyMarkdown.slice(start, start + 240) + '…'
}
</script>

<template>
  <AppPageHeader title="Search" subtitle="Substring match over every note">
    <template #actions>
      <div class="w-[340px]">
        <AppInput
          v-model="query"
          type="search"
          placeholder="cluster name, endpoint, anything you wrote down"
          @keyup.enter="search"
        />
      </div>
      <AppButton variant="primary" :loading="isRunning" @click="search">Search</AppButton>
    </template>
  </AppPageHeader>

  <div class="mx-auto w-full max-w-[1240px] space-y-4 px-8 pb-20 pt-6">
    <AppEmptyState
      v-if="results === null"
      title="Nothing searched yet"
      description="Unlike a stemmed index this also finds the middle of a path like /api/v1/pipelines."
    />
    <AppEmptyState
      v-else-if="!results.length"
      title="No matches"
      :description="`Nothing contains ${query}.`"
    />
    <AppCard v-for="document in results ?? []" :key="document.id">
      <div class="flex items-center gap-2">
        <span class="text-[14px] font-semibold text-text">{{ document.title }}</span>
        <AppBadge>{{ document.kind }}</AppBadge>
        <code class="ml-auto font-mono text-[11px] text-text-subtle">{{ document.owner }}</code>
      </div>
      <p class="mt-2 whitespace-pre-wrap text-[13px] leading-relaxed text-text-muted">
        {{ excerpt(document) }}
      </p>
    </AppCard>
  </div>
</template>
