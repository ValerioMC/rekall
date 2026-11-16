<script setup lang="ts">
import type { TargetSuggestion } from '@/features/connection/model/connection'

/** What a target input offers as the person types: the projects or spaces of the site, by key with their name. */
defineProps<{ id: string; suggestions: readonly TargetSuggestion[] }>()
</script>

<template>
  <datalist :id="id">
    <option v-for="suggestion in suggestions" :key="suggestion.value" :value="suggestion.value">{{ suggestion.name }}</option>
  </datalist>
</template>
