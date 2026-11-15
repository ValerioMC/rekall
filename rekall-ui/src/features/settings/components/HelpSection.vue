<script setup lang="ts">
import { computed, ref } from 'vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppInput from '@/components/ui/AppInput.vue'
import HelpContextPanel from '@/features/settings/components/HelpContextPanel.vue'
import { findInHelp, HELP_DEFAULTS, type HelpValues } from '@/features/settings/lib/help-template'
import { helpContextOf } from '@/features/settings/model/help-context'

/** What a session can do in Rekall: the context the rail names, or what the search finds across all of them. */
const props = defineProps<{ contextId: string | undefined }>()

const project = ref('')
const task = ref('')
const query = ref('')

const values = computed<HelpValues>(() => ({ project: project.value, task: task.value }))
const context = computed(() => helpContextOf(props.contextId))
const matches = computed(() => (query.value.trim() ? findInHelp(query.value) : null))
</script>

<template>
  <section class="flex flex-col gap-6" aria-label="Help" data-testid="help-section">
    <div class="flex flex-col gap-1">
      <h2 class="text-heading font-medium text-text">Help</h2>
      <p class="text-small text-text-muted">What a session can do in Rekall. What you type in Project and Task goes into every command and request, copied ready to paste.</p>
    </div>

    <div class="grid gap-3 min-[640px]:grid-cols-[1fr_1fr_1.6fr]">
      <label class="flex flex-col gap-1.5 text-small font-medium text-text-muted">
        Project label
        <AppInput v-model="project" :placeholder="HELP_DEFAULTS.project" mono />
      </label>
      <label class="flex flex-col gap-1.5 text-small font-medium text-text-muted">
        Task code
        <AppInput v-model="task" :placeholder="HELP_DEFAULTS.task" mono />
      </label>
      <label class="flex flex-col gap-1.5 text-small font-medium text-text-muted">
        Find a command, tool or request
        <AppInput v-model="query" type="search" placeholder="create task, rekall_note, wrapup" />
      </label>
    </div>

    <template v-if="matches">
      <AppEmptyState
        v-if="!matches.length"
        title="Nothing matches"
        description="Try what you want to do (create, note, merge), a tool name or part of a command."
      />
      <HelpContextPanel v-for="found in matches" :key="found.id" :context="found" :values="values" :show-summary="false" />
    </template>
    <HelpContextPanel v-else :context="context" :values="values" :show-summary="true" />
  </section>
</template>
