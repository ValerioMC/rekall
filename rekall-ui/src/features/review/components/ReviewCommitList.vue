<script setup lang="ts">
import { relativeTime } from '@/common/format/relative-time'
import type { CommitReference } from '@/features/commits/model/commitReference'

const SHORT_HASH_LENGTH = 7

/** The commits logged against a claim: hash, subject and when it was logged. */
defineProps<{ references: readonly CommitReference[] }>()
</script>

<template>
  <ul class="flex flex-col" data-testid="review-commits">
    <li v-for="reference in references" :key="reference.id" class="grid min-h-7.5 grid-cols-[4.5rem_minmax(0,1fr)_auto] items-center gap-2.5 text-small text-text-muted">
      <span class="font-mono text-caption text-text-subtle">{{ reference.commitHash.slice(0, SHORT_HASH_LENGTH) }}</span>
      <span class="truncate" :title="reference.comment">{{ reference.comment }}</span>
      <span class="text-text-subtle">{{ relativeTime(reference.createdAt) }}</span>
    </li>
  </ul>
</template>
