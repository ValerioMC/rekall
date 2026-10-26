<script setup lang="ts">
import { computed } from 'vue'
import { CONNECTION_KIND_PATH } from '@/features/connection/lib/connection-kind-glyph'
import { CONNECTION_KIND_LABEL, type ConnectionKind } from '@/features/connection/model/connection'

/** A connection kind as its system's logo, in the brand colour. */
const props = withDefaults(defineProps<{ kind: ConnectionKind; size?: number }>(), { size: 16 })

const BRAND: Readonly<Record<ConnectionKind, string>> = {
  GITLAB: 'text-brand-gitlab',
  KUBERNETES: 'text-brand-kubernetes',
  JIRA: 'text-brand-jira'
}

const path = computed(() => CONNECTION_KIND_PATH[props.kind])
</script>

<template>
  <svg
    class="shrink-0"
    :class="BRAND[kind]"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="currentColor"
    role="img"
    :data-kind="kind"
  >
    <title>{{ CONNECTION_KIND_LABEL[kind] }}</title>
    <path :d="path" />
  </svg>
</template>
