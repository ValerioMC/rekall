<script setup lang="ts">
import { ref, useId } from 'vue'
import PictureAlignmentGlyph from '@/features/picture/components/PictureAlignmentGlyph.vue'
import { PICTURE_ALIGNMENTS, PICTURE_ALIGNMENT_LABEL } from '@/features/picture/lib/picture-alignment'
import type { PictureAlignment } from '@/features/picture/model/picture'

/**
 * Where the chosen picture will sit on the page. Native radios, so ← and → change it and ↵ is
 * left to whoever holds the control. The ring follows any focus: WebKit draws none for `focus()`.
 */
const alignment = defineModel<PictureAlignment>({ required: true })

const name = useId()
const group = ref<HTMLElement | null>(null)

defineExpose({ focus: () => group.value?.querySelector<HTMLInputElement>('input:checked')?.focus() })
</script>

<template>
  <fieldset ref="group" class="flex shrink-0 items-center gap-2" data-testid="picture-alignment">
    <legend class="sr-only">Align the picture</legend>
    <span class="text-micro text-text-subtle" aria-hidden="true">Align</span>
    <div class="flex h-7 gap-0.5 rounded-control border border-border bg-surface p-0.5">
      <label
        v-for="option in PICTURE_ALIGNMENTS"
        :key="option"
        class="flex cursor-pointer items-center gap-1.5 rounded-key px-2 text-small transition-colors duration-(--motion-quick) has-[:focus]:outline-2 has-[:focus]:outline-offset-1 has-[:focus]:outline-accent"
        :class="alignment === option ? 'bg-surface-raised text-text shadow-key' : 'text-text-muted hover:text-text'"
      >
        <input v-model="alignment" type="radio" class="sr-only" :name="name" :value="option" />
        <PictureAlignmentGlyph :alignment="option" />
        {{ PICTURE_ALIGNMENT_LABEL[option] }}
      </label>
    </div>
  </fieldset>
</template>
