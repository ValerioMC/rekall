<script setup lang="ts">
import { computed, toRef } from 'vue'
import StepSeal from '@/components/console/StepSeal.vue'
import SkillMark from '@/components/skills/SkillMark.vue'
import { useStepEditor } from '@/composables/useStepEditor'
import { useStepSkills } from '@/composables/useStepSkills'
import type { Task, TaskStep } from '@/model/catalog'
import StepDetailEditor from './StepDetailEditor.vue'
import StepRowControls from './StepRowControls.vue'
import StepSkillsField from './StepSkillsField.vue'

/** One draft on the shelf below the checklist: still being worded, promoted when it is ready. */
const props = defineProps<{
  step: TaskStep
  task: Task
  isFirst: boolean
  isLast: boolean
  promoting: boolean
}>()

defineEmits<{ promote: []; move: [by: number]; remove: [] }>()

const { expandedId, toggle } = useStepEditor()
const expanded = computed(() => expandedId.value === props.step.id)
const { effective: skills } = useStepSkills(toRef(props, 'step'), toRef(props, 'task'))
</script>

<template>
  <li class="group/draft relative min-w-0" data-testid="draft-row" data-seal-host :data-step-id="step.id">
    <div
      class="min-w-0 rounded-card border border-dashed px-3 py-2 transition-colors"
      :class="expanded ? 'border-border-strong bg-surface-raised' : 'border-border-strong/70 hover:border-border-strong'"
    >
      <div class="flex items-start gap-2">
        <span class="mt-[1px] size-[22px] shrink-0">
          <StepSeal state="DRAFT" />
        </span>

        <button
          class="focus-ring min-w-0 flex-1 rounded-key text-left"
          :aria-expanded="expanded"
          data-testid="draft-title"
          @click="toggle(step)"
        >
          <span class="block text-body leading-snug text-text">{{ step.title }}</span>
          <span class="mt-1 flex flex-wrap items-center gap-2 text-micro">
            <span v-if="step.bodyMarkdown?.trim()" class="flex items-center gap-1 text-text-subtle" data-testid="draft-has-detail">
              <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M1.6 3h8.8M1.6 6h8.8M1.6 9h5.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
              </svg>
              detail
            </span>
            <span v-else class="text-text-subtle">no detail yet</span>
            <SkillMark :skills="skills" />
          </span>
        </button>

        <div class="flex shrink-0 items-center gap-0.5">
          <!-- Quiet until it is the thing under the pointer: a shelf of drafts each wearing a
               filled amber button is a shelf of competing calls to action. -->
          <button
            class="key-quiet rounded-key focus-ring group/promote inline-flex h-7 shrink-0 items-center gap-1.5 px-2.5 text-caption font-medium disabled:cursor-not-allowed disabled:opacity-40"
            :class="promoting && 'settle'"
            :disabled="promoting"
            :title="`Put ${step.title} on the checklist`"
            data-testid="draft-promote"
            @click="$emit('promote')"
          >
            <svg
              class="size-3 transition-transform duration-(--motion-move) group-hover/promote:-translate-y-px"
              viewBox="0 0 12 12"
              fill="none"
              aria-hidden="true"
            >
              <path
                d="M6 9.8V4M3.4 6.4 6 3.8l2.6 2.6M2.4 1.8h7.2"
                stroke="currentColor"
                stroke-width="1.3"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
            </svg>
            Promote
          </button>
          <div class="flex items-center gap-0.5 opacity-0 transition-opacity focus-within:opacity-100 group-hover/draft:opacity-100">
            <StepRowControls
              :title="step.title"
              :can-move-up="!isFirst"
              :can-move-down="!isLast"
              testid-prefix="draft"
              @up="$emit('move', -1)"
              @down="$emit('move', 1)"
              @remove="$emit('remove')"
            />
          </div>
        </div>
      </div>

      <Transition name="detail">
        <div v-if="expanded" class="mt-2.5">
          <StepSkillsField :step="step" :task="task" />
          <StepDetailEditor
            data-testid="draft-detail"
            :step-title="step.title"
            testid-prefix="draft"
            label="Detail"
            editor-height="260px"
            placeholder="What this step has to do, once it is promoted. Markdown."
            empty-text="Nothing written under this draft yet."
            write-prompt="Write the detail"
            :path-root="task.projectRepoFolder"
          />
        </div>
      </Transition>
    </div>
  </li>
</template>
