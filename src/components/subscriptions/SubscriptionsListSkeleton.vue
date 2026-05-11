<script setup lang="ts">
/**
 * Плейсхолдер списка подписок на время первой загрузки (форма карточек как у основного списка).
 */
</script>

<template>
  <div class="animate-pulse motion-reduce:animate-none space-y-3" aria-hidden="true">
    <div class="flex items-center justify-between px-3 py-2.5 rounded-xl bg-surface border border-border">
      <div class="h-4 w-28 rounded-md bg-surface-hover" />
      <div class="h-5 w-24 rounded-md bg-surface-hover" />
    </div>
    <div class="flex flex-col gap-1.5 sm:gap-2">
      <div
        v-for="n in 6"
        :key="n"
        class="rounded-2xl border border-border bg-surface-secondary h-[88px] sm:h-[96px]"
      />
    </div>
  </div>
</template>
