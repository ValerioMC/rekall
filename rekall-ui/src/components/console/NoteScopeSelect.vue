<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { identityHue } from '@/common/identity'
import { GLOBAL_SCOPE, scopeAdmits, scopeKey, type NoteScope } from '@/model/note-scope'
import type { RekallDocument } from '@/model/catalog'

/**
 * Where an open note lives, as a compact control in its toolbar. Every scope is offered, but one
 * that would leave out a task the note is on is disabled: narrowing goes through taking the
 * note off those tasks first, so a scope change never unlinks anything by itself.
 */
const props = defineProps<{ note: RekallDocument }>()
const emit = defineEmits<{ change: [scope: NoteScope] }>()

const store = useConsoleStore()
const { projects, companies, tasks } = storeToRefs(store)

interface ScopeOption {
  readonly key: string
  readonly scope: NoteScope
  readonly label: string
  readonly fits: boolean
}

const placedProjects = computed(() =>
  props.note.tasks
    .map((ref) => tasks.value.find((task) => task.id === ref.id)?.projectId)
    .map((projectId) => projects.value.find((project) => project.id === projectId))
    .filter((project) => project !== undefined)
)

function option(scope: NoteScope, label: string): ScopeOption {
  return {
    key: scopeKey(scope),
    scope,
    label,
    fits: placedProjects.value.every((project) => scopeAdmits(scope, project))
  }
}

const companyOptions = computed(() =>
  companies.value.map((company) => option({ kind: 'COMPANY', id: company.id }, company.name))
)
const projectOptions = computed(() =>
  projects.value.map((project) => option({ kind: 'PROJECT', id: project.id }, `${project.title} · ${project.companyName}`))
)
const everyOption = computed(() => [option(GLOBAL_SCOPE, 'Global'), ...companyOptions.value, ...projectOptions.value])

const current = computed(() => scopeKey(props.note.scope))

const dotColor = computed(() =>
  props.note.scope.kind === 'PROJECT' ? identityHue(props.note.scope.id).base : 'var(--color-text-subtle)'
)

function choose(key: string): void {
  const picked = everyOption.value.find((candidate) => candidate.key === key)
  if (picked && picked.fits && picked.key !== current.value) emit('change', picked.scope)
}
</script>

<template>
  <label class="relative inline-flex items-center gap-1.5 text-[11px] text-text-muted" data-testid="note-scope">
    <span class="eyebrow text-[10.5px]">Lives in</span>
    <span class="relative inline-flex items-center">
      <span
        class="pointer-events-none absolute left-2 size-1.5 rounded-full"
        :style="{ backgroundColor: dotColor }"
        aria-hidden="true"
      />
      <select
        class="focus-ring h-6 max-w-[220px] cursor-pointer appearance-none truncate rounded-full border border-border-strong bg-surface-raised pl-5 pr-6 text-[11px] text-text transition-colors hover:border-text-subtle"
        :value="current"
        aria-label="Where this note lives"
        data-testid="note-scope-select"
        @change="choose(($event.target as HTMLSelectElement).value)"
      >
        <option :value="everyOption[0]!.key">Global</option>
        <optgroup v-if="companyOptions.length" label="Company">
          <option v-for="entry in companyOptions" :key="entry.key" :value="entry.key" :disabled="!entry.fits">
            {{ entry.label }}
          </option>
        </optgroup>
        <optgroup v-if="projectOptions.length" label="Project">
          <option v-for="entry in projectOptions" :key="entry.key" :value="entry.key" :disabled="!entry.fits">
            {{ entry.label }}
          </option>
        </optgroup>
      </select>
      <svg
        class="pointer-events-none absolute right-2 size-2.5 text-text-subtle"
        viewBox="0 0 10 10"
        fill="none"
        aria-hidden="true"
      >
        <path d="m2.5 3.8 2.5 2.5 2.5-2.5" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </span>
  </label>
</template>
