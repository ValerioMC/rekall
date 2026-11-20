<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { relativeTime } from '@/common/format/relative-time'
import GitlabNamespaceGlyph from '@/features/connection/components/GitlabNamespaceGlyph.vue'
import type { PathVerdict } from '@/features/connection/composables/useGitlabPath'
import type { GitlabNamespace } from '@/features/connection/model/connection'

/**
 * What GitLab said about the path under the field, as one answer: found (a project or a group), not
 * found, outside the connection's root path, no credential to ask with, or not answering. Only
 * a found path is good news; the rest say what to do next.
 */
const props = defineProps<{ verdict: PathVerdict; host: string; connectionTitle: string; rootPath: string | null }>()
const emit = defineEmits<{ useNearest: [path: string]; retry: [] }>()

type Tone = 'ok' | 'bad' | 'warn' | 'wait'

const TONE_CLASSES: Readonly<Record<Tone, { readonly card: string; readonly tile: string; readonly pill: string }>> = {
  ok: { card: 'border-border', tile: 'bg-safe-soft text-safe', pill: 'bg-safe-soft text-safe' },
  bad: { card: 'border-danger/30', tile: 'bg-danger-soft text-danger', pill: 'bg-danger-soft text-danger' },
  warn: { card: 'border-border', tile: 'bg-warn-soft text-warn', pill: 'bg-warn-soft text-warn' },
  wait: { card: 'border-border', tile: 'bg-surface-raised text-text-muted', pill: 'bg-surface-raised text-text-muted' }
}

const check = computed(() => (props.verdict.state === 'answered' ? props.verdict.check : null))
const found = computed(() => (check.value?.status === 'FOUND' ? check.value.found : null))
const nearest = computed(() => (check.value?.status === 'NOT_FOUND' ? check.value.nearest : null))
const isRefusal = computed(() => check.value?.status === 'NOT_FOUND' || check.value?.status === 'OUTSIDE_ROOT')
const tone = computed<Tone>(() => {
  if (!check.value) return 'wait'
  if (check.value.status === 'FOUND') return 'ok'
  return isRefusal.value ? 'bad' : 'warn'
})
const classes = computed(() => TONE_CLASSES[tone.value])
const typedPath = computed(() => (props.verdict.state === 'blank' ? '' : props.verdict.path))
const statusName = computed(() => check.value?.status ?? (props.verdict.state === 'blank' ? 'BLANK' : 'CHECKING'))

function counts(namespace: GitlabNamespace): string[] {
  const parts: string[] = []
  if (namespace.subgroupCount !== null) parts.push(`${namespace.subgroupCount} ${namespace.subgroupCount === 1 ? 'subgroup' : 'subgroups'}`)
  if (namespace.projectCount !== null) parts.push(`${namespace.projectCount} ${namespace.projectCount === 1 ? 'project' : 'projects'}`)
  return parts
}
</script>

<template>
  <div
    class="mt-2 flex gap-3 rounded-control border bg-canvas px-3 py-2.5"
    :class="classes.card"
    role="status"
    data-testid="target-verdict"
    :data-verdict="statusName"
  >
    <span class="grid size-7 shrink-0 place-items-center rounded-control" :class="classes.tile" aria-hidden="true">
      <GitlabNamespaceGlyph v-if="verdict.state === 'blank'" kind="GROUP" />
      <span v-else-if="verdict.state === 'checking'" class="size-3.5 animate-spin rounded-full border-2 border-border-strong border-t-text-muted motion-reduce:animate-none" />
      <GitlabNamespaceGlyph v-else-if="found" :kind="found.kind" />
      <svg v-else-if="isRefusal" class="size-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="m4 4 8 8M12 4l-8 8" /></svg>
      <svg v-else class="size-3.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M8 2.5 14 13H2z" /><path d="M8 7v3M8 11.6v.1" /></svg>
    </span>

    <div class="min-w-0 flex-1">
      <template v-if="verdict.state === 'blank'">
        <p class="text-body font-semibold">Whole connection</p>
        <p class="mt-0.5 text-small text-text-muted">No path: the project reaches everything {{ rootPath ? `under ${rootPath}` : `on ${host}` }}.</p>
      </template>

      <template v-else-if="verdict.state === 'checking'">
        <p class="text-body font-semibold">Asking {{ host }}</p>
        <p class="mt-0.5 truncate font-mono text-caption text-text-muted">{{ typedPath }}</p>
      </template>

      <template v-else-if="found">
        <p class="flex flex-wrap items-baseline gap-x-2">
          <span class="text-body font-semibold">{{ found.name }}</span>
          <span class="truncate font-mono text-caption text-text-muted">{{ found.path }}</span>
        </p>
        <p class="mt-0.5 flex flex-wrap gap-x-3.5 text-caption text-text-subtle">
          <span>{{ found.kind === 'GROUP' ? 'Group' : 'Project' }}</span>
          <span v-if="found.visibility">{{ found.visibility }}</span>
          <span v-if="found.defaultBranch">Default branch <span class="font-mono">{{ found.defaultBranch }}</span></span>
          <span v-if="found.lastActivityAt">Last activity {{ relativeTime(found.lastActivityAt) }}</span>
          <span v-for="count in counts(found)" :key="count">{{ count }}</span>
        </p>
        <p v-if="found.kind === 'GROUP'" class="mt-1.5 text-small text-text-muted">The project will reach every repository under this group.</p>
      </template>

      <template v-else-if="check?.status === 'NOT_FOUND'">
        <p class="text-body font-semibold">No group or project at this path</p>
        <p class="mt-0.5 truncate font-mono text-caption text-text-muted">{{ typedPath }}</p>
        <p class="mt-1.5 text-small text-text-muted">
          Checked on {{ host }} with the stored token. A path the token cannot read looks the same as one that does not exist.
        </p>
        <AppButton v-if="nearest" size="sm" class="mt-2" data-testid="use-nearest" @click="emit('useNearest', nearest)">Use {{ nearest }}</AppButton>
      </template>

      <template v-else-if="check?.status === 'OUTSIDE_ROOT'">
        <p class="text-body font-semibold">Outside this connection's root path</p>
        <p class="mt-0.5 truncate font-mono text-caption text-text-muted">{{ typedPath }}</p>
        <p class="mt-1.5 text-small text-text-muted">
          {{ connectionTitle }} covers <span class="font-mono">{{ check.root }}</span> only. Pick a path under it, or add a connection for the other group.
        </p>
      </template>

      <template v-else-if="check?.status === 'NO_CREDENTIAL'">
        <p class="text-body font-semibold">No token is stored for {{ connectionTitle }}</p>
        <p class="mt-1.5 text-small text-text-muted">Paths are checked against GitLab, which needs the token. Set the token on this connection, then come back.</p>
      </template>

      <template v-else-if="check?.status === 'UNREACHABLE'">
        <p class="text-body font-semibold">{{ host }} did not answer</p>
        <p class="mt-1.5 text-small text-text-muted">{{ check.reason }} The path can only be bound once GitLab confirms it.</p>
        <AppButton size="sm" class="mt-2" data-testid="verdict-retry" @click="emit('retry')">Check again</AppButton>
      </template>
    </div>

    <span class="inline-flex h-6 shrink-0 items-center self-start rounded-full px-2 text-caption font-medium" :class="classes.pill" data-testid="verdict-pill">
      <template v-if="verdict.state === 'blank'">Whole connection</template>
      <template v-else-if="verdict.state === 'checking'">Checking</template>
      <template v-else-if="check?.status === 'FOUND'">Found</template>
      <template v-else-if="check?.status === 'NOT_FOUND'">Not found</template>
      <template v-else-if="check?.status === 'OUTSIDE_ROOT'">Outside root</template>
      <template v-else-if="check?.status === 'NO_CREDENTIAL'">No token</template>
      <template v-else>Can't check</template>
    </span>
  </div>
</template>
