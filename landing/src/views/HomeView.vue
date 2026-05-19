<script setup lang="ts">
import FeatureShowcase from "@/components/sections/FeatureShowcase.vue";
import AiSection from "@/components/sections/AiSection.vue";
import CustomizeSection from "@/components/sections/CustomizeSection.vue";
import DashboardWidgets from "@/components/sections/DashboardWidgets.vue";
import DownloadSection from "@/components/sections/DownloadSection.vue";
import DeviceGallerySection from "@/components/sections/DeviceGallerySection.vue";
import FaqSection from "@/components/sections/FaqSection.vue";
import HeroSection from "@/components/sections/HeroSection.vue";
import PlatformsStrip from "@/components/sections/PlatformsStrip.vue";
import PrivacySection from "@/components/sections/PrivacySection.vue";
import SyncSection from "@/components/sections/SyncSection.vue";
import VideoSection from "@/components/sections/VideoSection.vue";
import OpenValuesSection from "@/components/sections/OpenValuesSection.vue";
import WhySection from "@/components/sections/WhySection.vue";
import { featureSections } from "@/content/site.en";
</script>

<template>
  <HeroSection />
  <PlatformsStrip />
  <VideoSection />
  <WhySection />
  <OpenValuesSection />
  <FeatureShowcase
    v-for="(section, index) in featureSections"
    :key="section.id"
    :id="section.id"
    :eyebrow="section.eyebrow"
    :icon="section.icon"
    :title="section.title"
    :body="section.body"
    :bullets="section.bullets"
    :screenshot="section.screenshot"
    :alt="section.alt"
    :reverse="index % 2 === 1"
    :muted="index % 2 === 0"
  >
    <template v-if="section.id === 'dashboard'" #extra>
      <DashboardWidgets />
    </template>
  </FeatureShowcase>
  <AiSection />
  <SyncSection />
  <PrivacySection />
  <CustomizeSection />
  <DeviceGallerySection />
  <FaqSection />
  <DownloadSection />
</template>
