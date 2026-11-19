<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'
import { identityHue } from '@/common/identity'
import { slugify } from '@/features/catalog/model/catalog'

/**
 * What a record is called and what its anchor carries: the title, for a project its label (filled
 * from the title until it is typed in) and its task key, and the `/rk` line they make. A task has
 * no label: its anchor is its code, shown here and never typed. Renaming an existing project's
 * label moves its anchor, and the form says so.
 */
const props = withDefaults(
  defineProps<{
    kind: 'company' | 'project' | 'task'
    isNew: boolean
    /** The label as stored, for the rename warning; empty for a company, which has none. */
    originalLabel?: string
    /** The parent's id, which tints a task's anchor with its project's colour. */
    parentId?: string
    /** A task's code, or the one it will get; empty while the project is unknown. */
    taskCode?: string
    titleError: string | null
    labelError?: string | null
    /** Puts the cursor in the title, selected, once the dialog around it has settled. */
    autofocus?: boolean
  }>(),
  { originalLabel: '', parentId: '', taskCode: '', labelError: null, autofocus: false }
)

const title = defineModel<string>('title', { required: true })
const rawLabel = defineModel<string>('label', { default: '' })
const taskKey = defineModel<string>('taskKey', { default: '' })

const hasLabel = computed(() => props.kind === 'project')
const label = computed(() => slugify(rawLabel.value))
const labelWasTouched = ref(!props.isNew)
const anchorLive = ref(false)
const titleField = ref<HTMLInputElement | null>(null)

function onTitleInput(value: string): void {
  title.value = value
  if (hasLabel.value && !labelWasTouched.value) rawLabel.value = slugify(value)
}

function onLabelInput(value: string): void {
  labelWasTouched.value = true
  rawLabel.value = value
}

const anchorSelf = computed(() => {
  if (props.kind === 'company') return title.value.trim() ? `company:${title.value.trim()}` : ''
  if (props.kind === 'task') return props.taskCode ? `task:${props.taskCode}` : ''
  return label.value ? `project:${label.value}` : ''
})
const anchorPreview = computed(() => anchorSelf.value)

function onKeyInput(value: string): void {
  taskKey.value = value.replace(/[^a-z]/gi, '').slice(0, 4).toUpperCase()
}
const anchorIsMoving = computed(() => !props.isNew && hasLabel.value && label.value !== props.originalLabel)

onMounted(async () => {
  if (!props.autofocus) return
  await nextTick()
  titleField.value?.focus()
  titleField.value?.select()
})
</script>

<template>
  <label for="record-title" class="mb-1.5 block eyebrow text-caption">Title</label>
  <input
    id="record-title"
    ref="titleField"
    :value="title"
    data-testid="record-title"
    class="field text-text h-(--spacing-control) w-full rounded-control px-3 text-title"
    :aria-invalid="titleError ? 'true' : undefined"
    :placeholder="hasLabel ? 'Report builder, main workflow' : 'Acme S.p.A.'"
    @input="onTitleInput(($event.target as HTMLInputElement).value)"
    @focus="anchorLive = true"
    @blur="anchorLive = false"
  />
  <p v-if="titleError" class="mt-1.5 text-caption text-danger" role="alert">{{ titleError }}</p>
  <p v-else class="mt-1.5 text-caption text-text-subtle">
    What this is called. Change it whenever a better name turns up.
  </p>

  <template v-if="hasLabel">
    <label for="record-label" class="mb-1.5 mt-5 block eyebrow text-caption">Label</label>
    <input
      id="record-label"
      :value="rawLabel"
      data-testid="record-label"
      spellcheck="false"
      autocomplete="off"
      class="field field-anchor h-(--spacing-control) w-full rounded-control px-3 font-mono text-body text-anchor"
      :aria-invalid="labelError ? 'true' : undefined"
      placeholder="report-builder"
      @input="onLabelInput(($event.target as HTMLInputElement).value)"
      @focus="anchorLive = true"
      @blur="anchorLive = false"
    />
    <p v-if="labelError" class="mt-1.5 text-caption text-danger" role="alert">{{ labelError }}</p>
    <p v-else class="mt-1.5 text-caption text-text-subtle">
      What the anchor carries. Lowercase, no spaces, unique inside its parent.
    </p>

    <label for="record-key" class="mb-1.5 mt-5 block eyebrow text-caption">Task key</label>
    <input
      id="record-key"
      :value="taskKey"
      data-testid="record-key"
      spellcheck="false"
      autocomplete="off"
      maxlength="4"
      class="field field-anchor h-(--spacing-control) w-28 rounded-control px-3 font-mono text-body uppercase text-anchor"
      placeholder="RK"
      @input="onKeyInput(($event.target as HTMLInputElement).value)"
    />
    <p class="mt-1.5 text-caption text-text-subtle">
      <template v-if="isNew">Two to four letters opening every task code, like RK-12. Left empty, one is taken from the label.</template>
      <template v-else>Changing it renames every task code of this project.</template>
    </p>
  </template>

  <div
    class="mt-4 flex items-center gap-3 rounded-control border px-3.5 py-3 transition-all duration-(--motion-move)"
    :class="anchorLive ? 'border-anchor bg-anchor-soft shadow-[0_0_0_3px_var(--color-anchor-soft)]' : 'border-anchor-line bg-anchor-soft'"
    data-testid="anchor-preview"
    :title="anchorPreview"
  >
    <span class="shrink-0 font-mono text-small text-text-subtle" aria-hidden="true">/rk</span>
    <span
      v-if="kind === 'task' && parentId"
      class="size-1.5 shrink-0 rounded-full"
      :style="{ backgroundColor: identityHue(parentId).base }"
      aria-hidden="true"
    />
    <span class="min-w-0 flex-1 truncate font-mono text-body text-anchor">{{ anchorSelf || '…' }}</span>
    <span class="shrink-0 eyebrow font-normal">loads this</span>
  </div>

  <p
    v-if="anchorIsMoving"
    class="mt-2.5 rounded-control border border-warn bg-warn-soft px-3.5 py-2.5 text-small leading-relaxed text-warn"
    data-testid="rename-warning"
  >
    The anchor moves with this label. Anything still typing
    <code>{{ kind }}:{{ originalLabel }}</code> will stop finding it.
  </p>
</template>
