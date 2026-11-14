<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import TokenCount from '@/components/ui/TokenCount.vue'
import { PROJECT_STATUS_LABEL, type ProjectStatus } from '@/features/catalog/model/project-status'
import type { Project } from '@/features/catalog/model/catalog'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { useConsoleStore } from '@/stores/console.store'

const props = defineProps<{ project: Project }>()

const TONES: Readonly<Record<ProjectStatus, 'accent' | 'safe' | 'neutral'>> = {
  ACTIVE: 'accent',
  PAUSED: 'neutral',
  DONE: 'safe',
  ARCHIVED: 'neutral'
}

const { tasks } = storeToRefs(useConsoleStore())
const tokenUsage = useTokenUsageStore()
const tokens = computed(() => tokenUsage.ofProject(props.project.id, tasks.value))
</script>

<template>
  <span class="anchor-chip inline-flex h-[22px] items-center px-2 text-caption" data-testid="project-anchor">/rk {{ project.anchor }}</span>
  <CopyAnchorButton :anchor="project.anchor" data-testid="copy-project-anchor" />
  <AppBadge :tone="TONES[project.status]" dot>{{ PROJECT_STATUS_LABEL[project.status] }}</AppBadge>
  <TokenCount :totals="tokens" :hue-of="project.id" />
</template>
