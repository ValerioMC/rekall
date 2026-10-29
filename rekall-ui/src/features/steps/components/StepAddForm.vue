<script setup lang="ts">
import { ref } from 'vue'
import SkillChip from '@/features/skills/components/SkillChip.vue'
import SkillGlyph from '@/features/skills/components/SkillGlyph.vue'
import SkillPicker from '@/features/skills/components/SkillPicker.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

/**
 * The line at the foot of the checklist that names a new step, which starts as a draft. The
 * spark beside it picks the skills the step starts with; they show as chips under the line. The
 * session toggle makes the step open a fresh Claude session instead of joining the running one.
 */
const title = defineModel<string>({ required: true })
const skills = defineModel<readonly string[]>('skills', { default: () => [] })
const newSession = defineModel<boolean>('newSession', { default: false })

defineEmits<{ submit: [] }>()

const picking = ref(false)
const pickButton = ref<HTMLElement | null>(null)

function remove(name: string): void {
  skills.value = skills.value.filter((candidate) => candidate !== name)
}
</script>

<template>
  <form
    class="dock-lane-safe shrink-0 border-t border-border bg-surface py-3 pl-5"
    @submit.prevent="$emit('submit')"
  >
    <div class="flex items-center gap-2">
      <span
        class="grid size-[22px] shrink-0 place-items-center rounded-full border border-dashed border-border-strong text-text-subtle"
        aria-hidden="true"
      >
        <svg class="size-3" viewBox="0 0 12 12" fill="none">
          <path d="M6 2.4v7.2M2.4 6h7.2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </span>
      <AppInput
        id="new-step"
        v-model="title"
        placeholder="Name the next piece of this task. It starts as a draft."
        data-testid="new-step"
      />
      <button
        ref="pickButton"
        type="button"
        class="focus-ring inline-flex h-8 shrink-0 items-center gap-1 rounded-key border px-2 text-caption transition-colors"
        :class="
          skills.length
            ? 'border-anchor-line bg-anchor-soft text-anchor'
            : 'border-border-strong text-text-muted hover:border-anchor hover:text-anchor'
        "
        :aria-expanded="picking"
        aria-haspopup="dialog"
        aria-label="Choose skills for the new step"
        data-testid="new-step-skills"
        @click="picking = !picking"
      >
        <SkillGlyph :size="12" />
        <span v-if="skills.length" class="tabular-nums">{{ skills.length }}</span>
      </button>
      <button
        type="button"
        class="focus-ring inline-flex h-8 shrink-0 items-center rounded-key border px-2 text-caption transition-colors"
        :class="
          newSession
            ? 'border-anchor-line bg-anchor-soft text-anchor'
            : 'border-border-strong text-text-muted hover:border-anchor hover:text-anchor'
        "
        :aria-pressed="newSession"
        aria-label="Open a new session on the new step"
        data-testid="new-step-session"
        @click="newSession = !newSession"
      >
        New session
      </button>
      <AppButton variant="primary" type="submit" :disabled="!title.trim()" data-testid="add-step">
        Add draft
      </AppButton>
    </div>
    <div
      v-if="skills.length"
      class="mt-2 flex flex-wrap gap-1.5 pl-[30px]"
      data-testid="new-step-skill-chips"
    >
      <SkillChip v-for="name in skills" :key="name" :name="name" origin="step" @remove="remove(name)" />
    </div>
    <SkillPicker
      v-if="picking && pickButton"
      :model-value="skills"
      :anchor="pickButton"
      title="Skills for the new step"
      @update:model-value="skills = $event"
      @close="picking = false"
    />
  </form>
</template>
