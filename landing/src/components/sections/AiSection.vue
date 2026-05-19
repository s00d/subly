<script setup lang="ts">
import SectionShell from "@/components/layout/SectionShell.vue";
import RevealOnScroll from "@/components/ui/RevealOnScroll.vue";
import ScreenshotFrame from "@/components/ui/ScreenshotFrame.vue";
import UiIcon from "@/components/ui/UiIcon.vue";
import { useSiteContent } from "@/composables/useSiteContent";
import { screenshotUrl } from "@/lib/assets";
import type { MdiIconName } from "@/mdi-icons";

const { content } = useSiteContent();
</script>

<template>
  <SectionShell
    id="ai"
    :eyebrow="content.aiSection.eyebrow"
    :eyebrow-icon="content.aiSection.icon"
    :title="content.aiSection.title"
    muted
  >
    <RevealOnScroll tag="div" class="max-w-3xl">
      <p class="text-lg leading-relaxed text-muted md:text-xl">
        {{ content.aiSection.lead }}
      </p>
    </RevealOnScroll>

    <div class="mt-10 grid gap-4 sm:grid-cols-3">
      <RevealOnScroll
        v-for="(item, i) in content.aiSection.principles"
        :key="item.title"
        tag="div"
        :stagger-index="i"
        class="rounded-xl border border-border bg-surface/60 px-4 py-4"
      >
        <UiIcon :name="item.icon as MdiIconName" size="md" class="text-accent" />
        <h3 class="mt-2 font-semibold text-text">{{ item.title }}</h3>
        <p class="mt-1 text-sm text-muted">{{ item.body }}</p>
      </RevealOnScroll>
    </div>

    <div class="mt-16 grid items-start gap-10 lg:grid-cols-2 lg:gap-14">
      <div>
        <h3 class="text-sm font-semibold uppercase tracking-widest text-accent">
          {{ content.aiSection.capabilitiesTitle }}
        </h3>
        <p class="mt-2 text-muted">
          {{ content.aiSection.capabilitiesIntro }}
        </p>
        <div class="mt-6 grid gap-4 sm:grid-cols-2">
          <RevealOnScroll
            v-for="(feat, i) in content.aiSection.features"
            :key="feat.title"
            tag="article"
            :stagger-index="i"
            class="flex flex-col rounded-xl border border-border bg-surface p-5 transition-colors hover:border-accent/40"
          >
            <div class="flex items-center gap-2">
              <span
                class="flex size-9 shrink-0 items-center justify-center rounded-lg border border-border bg-bg"
              >
                <UiIcon :name="feat.icon as MdiIconName" size="md" class="text-accent" />
              </span>
              <h4 class="font-semibold text-text">{{ feat.title }}</h4>
            </div>
            <p class="mt-3 flex-1 text-sm leading-relaxed text-muted">{{ feat.body }}</p>
            <p class="mt-3 border-t border-border pt-3 text-xs italic text-muted/90">
              {{ feat.example }}
            </p>
          </RevealOnScroll>
        </div>
      </div>

      <RevealOnScroll tag="div" :stagger-index="2">
        <ScreenshotFrame
          :src="screenshotUrl(content.aiSection.screenshot)"
          :alt="content.aiSection.alt"
        />
        <ul class="mt-6 space-y-2">
          <li
            v-for="(item, i) in content.aiSection.settingsBullets"
            :key="i"
            class="flex gap-2 text-sm text-muted"
          >
            <UiIcon name="check-circle-outline" size="sm" class="mt-0.5 shrink-0 text-accent" />
            {{ item }}
          </li>
        </ul>
      </RevealOnScroll>
    </div>

    <RevealOnScroll tag="div" :stagger-index="3" class="mt-16">
      <h3 class="text-center text-sm font-semibold uppercase tracking-widest text-muted">
        {{ content.aiSection.inputMethodsTitle }}
      </h3>
      <div class="mt-6 grid gap-4 sm:grid-cols-3">
        <div
          v-for="method in content.aiSection.inputMethods"
          :key="method.label"
          class="flex flex-col items-center rounded-xl border border-border bg-bg px-4 py-5 text-center"
        >
          <UiIcon :name="method.icon as MdiIconName" size="lg" class="text-accent" />
          <span class="mt-3 font-medium text-text">{{ method.label }}</span>
          <span class="mt-1 text-xs text-muted">{{ method.detail }}</span>
        </div>
      </div>
    </RevealOnScroll>

    <RevealOnScroll tag="div" :stagger-index="4" class="mt-14">
      <h3 class="mb-4 text-sm font-semibold uppercase tracking-widest text-muted">
        {{ content.aiSection.providersTitle }}
      </h3>
      <ul class="flex flex-wrap gap-2">
        <li
          v-for="name in content.aiSection.providers"
          :key="name"
          class="rounded-lg border border-border bg-surface px-3 py-2 text-sm font-medium text-text"
        >
          {{ name }}
        </li>
      </ul>
    </RevealOnScroll>

    <RevealOnScroll
      tag="div"
      :stagger-index="5"
      class="mt-10 flex gap-4 rounded-xl border border-accent/30 bg-accent/5 p-5 md:p-6"
    >
      <UiIcon name="shield-lock-outline" size="lg" class="shrink-0 text-accent" />
      <p class="text-sm leading-relaxed text-muted md:text-base">
        {{ content.aiSection.privacyNote }}
      </p>
    </RevealOnScroll>
  </SectionShell>
</template>
