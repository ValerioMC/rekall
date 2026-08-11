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
import type { DocumentMatch } from '@/model/records'

const { run, isRunning } = useAsyncAction()

const query = ref('')
const results = ref<readonly DocumentMatch[] | null>(null)

async function search(): Promise<void> {
  if (!query.value.trim()) return
  const found = await run(() => searchDocuments(query.value))
  if (found) results.value = found
}
</script>

<template>
  <AppPageHeader title="Search">
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
    <p class="max-w-[74ch] text-[13.5px] leading-relaxed text-text-muted">
      Full text across every document. Quoted phrases and <code class="text-accent">or</code> work as in
      a search box. URLs and paths are indexed whole, so
      <code class="text-accent">/api/v1/pipelines</code> is found by the full path rather than by
      <code class="text-accent">pipelines</code> alone.
    </p>

    <AppEmptyState
      v-if="results && !results.length"
      title="No matches"
      description="Nothing stored mentions that."
    />

    <AppCard v-for="match in results ?? []" :key="match.documentId" data-testid="search-result">
      <div class="flex flex-wrap items-center gap-2.5">
        <strong class="text-[14px] font-medium text-text">{{ match.title }}</strong>
        <AppBadge>{{ match.kind }}</AppBadge>
        <div class="flex-1" />
        <RouterLink
          :to="`/data/${match.entityName}/${match.recordId}`"
          class="focus-ring text-[12.5px] text-accent underline-offset-4 hover:underline"
        >
          open in {{ match.entityName }} &rarr;
        </RouterLink>
      </div>
      <!-- Server-generated ts_headline fragment: contains <b> around the hits and nothing else. -->
      <!-- eslint-disable-next-line vue/no-v-html -->
      <p class="mt-2.5 text-[12.5px] leading-relaxed text-text-muted [&_b]:bg-accent-soft [&_b]:px-0.5 [&_b]:text-accent" v-html="match.excerpt" />
    </AppCard>
  </div>
</template>
