<script setup lang="ts">
defineProps<{
  id: string;
  eyebrow?: string;
  title: string;
  subtitle?: string;
  muted?: boolean;
}>();
</script>

<template>
  <section
    :id="id"
    class="scroll-mt-24 px-5 py-20 md:py-28"
    :class="muted ? 'bg-surface/40' : ''"
  >
    <div class="motion-safe mx-auto max-w-6xl">
      <header v-if="eyebrow || subtitle" class="mb-10 max-w-2xl">
        <p v-if="eyebrow" class="text-xs font-semibold uppercase tracking-widest text-accent">
          {{ eyebrow }}
        </p>
        <h2 class="mt-2 text-3xl font-bold tracking-tight text-text md:text-4xl">
          {{ title }}
        </h2>
        <p v-if="subtitle" class="mt-4 text-lg text-muted">{{ subtitle }}</p>
      </header>
      <h2
        v-else
        class="mb-10 text-3xl font-bold tracking-tight text-text md:text-4xl"
      >
        {{ title }}
      </h2>
      <slot />
    </div>
  </section>
</template>
