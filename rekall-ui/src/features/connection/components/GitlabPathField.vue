<script setup lang="ts">
import { computed, useId, watch } from 'vue'
import AppField from '@/components/ui/AppField.vue'
import GitlabPathList from '@/features/connection/components/GitlabPathList.vue'
import TargetVerdict from '@/features/connection/components/TargetVerdict.vue'
import { useGitlabPath } from '@/features/connection/composables/useGitlabPath'
import type { ConnectionId } from '@/model/branded'

/**
 * The GitLab path a project is narrowed to, typed or chosen from what the token can read, and
 * checked against the instance before it can be bound. Says through `resolve` whether the path may
 * be bound and which one it is: blank binds the whole connection, a found path binds that path. A
 * path the binding already has is checked on arrival without opening the list.
 */
const props = defineProps<{
  connectionId: ConnectionId
  connectionTitle: string
  baseUrl: string | null
  rootPath: string | null
  initialPath?: string | null
}>()
const emit = defineEmits<{ resolve: [binding: { readonly ready: boolean; readonly path: string | null }] }>()

const listId = useId()
const path = useGitlabPath(() => props.connectionId)
if (props.initialPath) path.seed(props.initialPath)
const host = computed(() => (props.baseUrl ? new URL(props.baseUrl).host : 'GitLab'))
const verdictStatus = computed(() => (path.verdict.value.state === 'answered' ? path.verdict.value.check.status : null))
const isRefused = computed(() => verdictStatus.value === 'NOT_FOUND' || verdictStatus.value === 'OUTSIDE_ROOT')
const activeOptionId = computed(() => (path.isOpen.value && path.rows.value.length ? `${listId}-option-${path.activeIndex.value}` : undefined))

watch(
  [path.isBindable, path.verified],
  () => emit('resolve', { ready: path.isBindable.value, path: path.verified.value?.path ?? null }),
  { immediate: true }
)

function onInput(event: Event): void {
  path.type((event.target as HTMLInputElement).value)
}

function onEnter(event: KeyboardEvent): void {
  if (path.isOpen.value && path.chooseActive()) event.preventDefault()
}

function onEscape(event: KeyboardEvent): void {
  if (!path.isOpen.value) return
  event.stopPropagation()
  path.close()
}
</script>

<template>
  <AppField v-slot="{ fieldId, describedBy }" label="GitLab project path" hint="Optional. Leave it empty to give the project the whole connection; type a path or pick one to narrow it.">
    <div class="field flex h-(--spacing-control) items-center rounded-control" :aria-invalid="isRefused || undefined">
      <span class="select-none pl-2.5 font-mono text-small text-text-subtle">{{ host }}/</span>
      <input
        :id="fieldId"
        :value="path.value.value"
        role="combobox"
        aria-autocomplete="list"
        :aria-expanded="path.isOpen.value"
        :aria-controls="listId"
        :aria-activedescendant="activeOptionId"
        :aria-describedby="describedBy"
        spellcheck="false"
        autocomplete="off"
        placeholder="platform/backend/payments-api"
        class="h-full min-w-0 flex-1 bg-transparent pr-2.5 pl-0.5 font-mono text-small text-text outline-none placeholder:text-text-subtle"
        data-testid="binder-target"
        @input="onInput"
        @focus="path.open()"
        @blur="path.close()"
        @keydown.down.prevent="path.move(1)"
        @keydown.up.prevent="path.move(-1)"
        @keydown.enter="onEnter"
        @keydown.esc="onEscape"
      />
      <span class="grid w-8 shrink-0 place-items-center" aria-hidden="true">
        <span v-if="path.verdict.value.state === 'checking'" class="size-3.5 animate-spin rounded-full border-2 border-border-strong border-t-text-muted motion-reduce:animate-none" />
        <svg v-else-if="verdictStatus === 'FOUND'" class="size-4 text-safe" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="m3.5 8.5 3 3 6-7" /></svg>
        <svg v-else-if="isRefused" class="size-4 text-danger" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="m4 4 8 8M12 4l-8 8" /></svg>
        <svg v-else-if="verdictStatus" class="size-4 text-warn" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M8 2.5 14 13H2z" /><path d="M8 7v3M8 11.6v.1" /></svg>
      </span>
    </div>

    <GitlabPathList
      v-if="path.isOpen.value"
      :id="listId"
      :cursor="path.cursor.value"
      :root-path="rootPath"
      :rows="path.rows.value"
      :active-index="path.activeIndex.value"
      :unavailable="path.listingUnavailable.value"
      @choose="path.choose"
      @jump="path.jumpTo"
    />
    <TargetVerdict
      v-else
      :verdict="path.verdict.value"
      :host="host"
      :connection-title="connectionTitle"
      :root-path="rootPath"
      @use-nearest="path.type($event)"
      @retry="path.retry()"
    />
  </AppField>
</template>
