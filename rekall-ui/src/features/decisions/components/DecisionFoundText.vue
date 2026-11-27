<script setup lang="ts">
import { computed } from 'vue'
import { matchParts } from '@/common/format/match-parts'

/** A text of a decision with every occurrence of the searched string underlined. */
const props = defineProps<{ text: string; query: string }>()

const parts = computed(() => matchParts(props.text, props.query))
</script>

<template>
  <template v-for="(part, index) in parts" :key="index"><mark v-if="part.hit" class="bg-transparent text-inherit underline decoration-accent decoration-2 underline-offset-3 [text-decoration-skip-ink:none]">{{ part.text }}</mark><template v-else>{{ part.text }}</template></template>
</template>
