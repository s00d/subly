<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import {
  syncStatus,
  getProviders,
  getSyncConfig,
  setProviderCredentials,
  enableSync,
  disableSync,
  pullRemote,
  pushLocal,
  checkRemote,
  dismissPendingUpdate,
} from "@/services/sync";
import type { SyncProviderType } from "@/services/sync";
import {
  RefreshCw, Cloud, CloudOff, Check, AlertTriangle,
  Loader2, Save, Download, Upload, ChevronDown,
} from "lucide-vue-next";
import Tooltip from "@/components/ui/Tooltip.vue";
import { tv } from "@/lib/tv";

const { t } = useI18n();
const { toast } = useToast();

const allProviders = getProviders();
const isConnecting = ref(false);
const expandedProvider = ref<SyncProviderType | null>(null);
const showChangeProvider = ref(false);

const gdriveClientId = ref("");
const gdriveClientSecret = ref("");
const dropboxAppKey = ref("");
const dropboxAppSecret = ref("");
const onedriveClientId = ref("");
const webdavUrl = ref("");
const webdavUsername = ref("");
const webdavPassword = ref("");

onMounted(() => {
  const cfg = getSyncConfig();
  gdriveClientId.value = cfg.gdriveClientId;
  gdriveClientSecret.value = cfg.gdriveClientSecret;
  dropboxAppKey.value = cfg.dropboxAppKey;
  dropboxAppSecret.value = cfg.dropboxAppSecret;
  onedriveClientId.value = cfg.onedriveClientId;
  webdavUrl.value = cfg.webdavUrl;
  webdavUsername.value = cfg.webdavUsername;
  webdavPassword.value = cfg.webdavPassword;
});

const activeProviderInfo = computed(() =>
  syncStatus.enabled && syncStatus.provider
    ? allProviders.find((p) => p.type === syncStatus.provider) ?? null
    : null,
);

const availableProviders = computed(() =>
  allProviders.filter((p) => p.type !== syncStatus.provider),
);

const lastSyncedFormatted = computed(() => {
  if (!syncStatus.lastSynced) return t("sync_never");
  return new Date(syncStatus.lastSynced).toLocaleString();
});

const lastSyncedRemoteFormatted = computed(() => {
  if (!syncStatus.remoteUpdatedAt) return t("sync_never");
  return new Date(syncStatus.remoteUpdatedAt).toLocaleString();
});

function canConnect(type: SyncProviderType): boolean {
  if (type === "gdrive") return !!gdriveClientId.value && !!gdriveClientSecret.value;
  if (type === "dropbox") return !!dropboxAppKey.value && !!dropboxAppSecret.value;
  if (type === "onedrive") return !!onedriveClientId.value;
  if (type === "webdav") return !!webdavUrl.value && !!webdavUsername.value;
  return true;
}

function toggleExpand(type: SyncProviderType) {
  expandedProvider.value = expandedProvider.value === type ? null : type;
}

async function saveCredentials(type: SyncProviderType) {
  if (type === "gdrive") {
    await setProviderCredentials("gdrive", {
      clientId: gdriveClientId.value,
      clientSecret: gdriveClientSecret.value,
    });
  } else if (type === "dropbox") {
    await setProviderCredentials("dropbox", {
      appKey: dropboxAppKey.value,
      appSecret: dropboxAppSecret.value,
    });
  } else if (type === "onedrive") {
    await setProviderCredentials("onedrive", {
      clientId: onedriveClientId.value,
    });
  } else if (type === "webdav") {
    await setProviderCredentials("webdav", {
      serverUrl: webdavUrl.value,
      username: webdavUsername.value,
      password: webdavPassword.value,
    });
  }
  toast(t("sync_credentials_saved"));
}

async function handleConnect(type: SyncProviderType) {
  isConnecting.value = true;
  try {
    const ok = await enableSync(type);
    if (ok) {
      expandedProvider.value = null;
      showChangeProvider.value = false;
      toast(t("sync_success"));
    } else {
      toast(t("sync_not_available"), "error");
    }
  } catch {
    toast(t("sync_error"), "error");
  } finally {
    isConnecting.value = false;
  }
}

async function handleDisconnect() {
  await disableSync();
  showChangeProvider.value = false;
  toast(t("sync_disabled"));
}

async function handleCheckRemote() {
  const hasUpdate = await checkRemote();
  if (hasUpdate) {
    toast(t("sync_remote_newer"));
  } else {
    toast(t("sync_success"));
  }
}

async function handlePull() {
  const ok = await pullRemote();
  if (ok) {
    toast(t("sync_pull_success"));
  } else if (syncStatus.error) {
    toast(syncStatus.error, "error");
  }
}

async function handlePush() {
  const ok = await pushLocal();
  if (ok) {
    toast(t("sync_push_success"));
  } else if (syncStatus.error) {
    toast(syncStatus.error, "error");
  }
}

const sectionTv = tv({
  slots: {
    root: "bg-[var(--color-surface)] rounded-xl border border-[var(--color-border)] p-4 sm:p-5",
    header: "flex items-center gap-2 mb-1",
    title: "text-base sm:text-lg font-semibold text-[var(--color-text-primary)]",
    desc: "text-xs sm:text-sm text-[var(--color-text-muted)] mb-4",
    statusCard: "mb-4 p-4 rounded-xl bg-[var(--color-surface-secondary)] border border-[var(--color-border)]",
    statusTop: "flex items-center gap-3",
    statusIcon: "w-10 h-10 rounded-xl object-contain p-1.5 bg-[var(--color-surface)] border border-[var(--color-border)]",
    statusInfo: "flex-1 min-w-0",
    statusName: "text-sm font-semibold text-[var(--color-text-primary)]",
    statusMeta: "text-[10px] text-[var(--color-text-muted)] mt-0.5",
    statusActions: "flex items-center gap-3 mt-3 pt-3 border-t border-[var(--color-border)]",
    actionBtn: "flex-1 flex items-center justify-center gap-1.5 px-3 py-2 rounded-lg text-xs font-medium transition-colors disabled:opacity-50",
    dangerBtn: "text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 border border-red-200 dark:border-red-800/40",
    providerCard: "rounded-xl border border-[var(--color-border)] overflow-hidden transition-all hover:border-[var(--color-text-muted)]",
    providerRow: "flex items-center gap-3 p-3 cursor-pointer select-none",
    providerIcon: "w-8 h-8 rounded-lg object-contain",
    providerName: "text-sm font-medium text-[var(--color-text-primary)]",
    providerDesc: "text-[10px] text-[var(--color-text-muted)]",
    providerChevron: "text-[var(--color-text-muted)] transition-transform duration-200 shrink-0",
    credForm: "px-3 pb-3 pt-0",
    credFormInner: "space-y-2.5 p-3 rounded-lg bg-[var(--color-surface-secondary)]",
    credLabel: "block text-[10px] font-medium text-[var(--color-text-muted)] mb-1",
    credInput: [
      "w-full px-2.5 py-1.5 rounded-lg border border-[var(--color-border)]",
      "bg-[var(--color-surface)] text-xs text-[var(--color-text-primary)]",
      "focus:outline-none focus:ring-1 focus:ring-[var(--color-primary)]",
    ],
    credActions: "flex items-center gap-2 pt-1",
    saveBtn: [
      "flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors disabled:opacity-50",
      "bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text-primary)]",
      "hover:bg-[var(--color-surface-hover)]",
    ],
    connectBtn: [
      "flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors disabled:opacity-50",
      "bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)]",
    ],
    pendingBanner: "mb-4 p-3 rounded-xl bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800",
    errorBanner: "mb-4 p-3 rounded-xl bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800",
    changeLabel: "text-xs font-medium text-[var(--color-text-muted)] mb-2 mt-4",
  },
});

const s = sectionTv();
</script>

<template>
  <section :class="s.root()">
    <div :class="s.header()">
      <Cloud :size="18" class="text-[var(--color-primary)]" />
      <h2 :class="s.title()">{{ t('cloud_sync') }}</h2>
    </div>
    <p :class="s.desc()">{{ t('cloud_sync_desc') }}</p>

    <!-- ============ CONNECTED STATE ============ -->
    <template v-if="activeProviderInfo">
      <div :class="s.statusCard()">
        <div :class="s.statusTop()">
          <img :src="activeProviderInfo.icon" :alt="activeProviderInfo.name" :class="s.statusIcon()" />
          <div :class="s.statusInfo()">
            <div class="flex items-center gap-2">
              <span :class="s.statusName()">{{ activeProviderInfo.name }}</span>
              <span class="flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 text-[10px] font-medium">
                <Check :size="10" />
                {{ t('sync_connected') }}
              </span>
            </div>
            <div :class="s.statusMeta()">
              <span>{{ t('sync_last_synced') }}: {{ lastSyncedFormatted }}</span>
              <template v-if="syncStatus.remoteUpdatedAt">
                <span class="mx-1">·</span>
                <span>{{ t('sync_last_synced') }} ({{ t('sync_push') }}): {{ lastSyncedRemoteFormatted }}</span>
              </template>
            </div>
          </div>
          <div class="shrink-0">
            <Loader2 v-if="syncStatus.syncing" :size="16" class="text-[var(--color-primary)] animate-spin" />
            <AlertTriangle v-else-if="syncStatus.error" :size="16" class="text-amber-500" />
          </div>
        </div>

        <!-- Sync action buttons -->
        <div :class="s.statusActions()">
          <Tooltip :text="t('sync_now')">
            <button
              @click="handleCheckRemote"
              :disabled="syncStatus.syncing"
              :class="s.actionBtn()"
              class="text-[var(--color-primary)] hover:bg-[var(--color-primary-light)] border border-[var(--color-primary)]/20"
            >
              <RefreshCw :size="13" :class="{ 'animate-spin': syncStatus.syncing }" />
              {{ t('sync_now') }}
            </button>
          </Tooltip>
          <Tooltip :text="t('sync_pull')">
            <button
              @click="handlePull"
              :disabled="syncStatus.syncing"
              :class="s.actionBtn()"
              class="text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900/20 border border-blue-200 dark:border-blue-800/40"
            >
              <Download :size="13" />
              {{ t('sync_pull') }}
            </button>
          </Tooltip>
          <Tooltip :text="t('sync_push')">
            <button
              @click="handlePush"
              :disabled="syncStatus.syncing"
              :class="s.actionBtn()"
              class="text-green-600 hover:bg-green-50 dark:hover:bg-green-900/20 border border-green-200 dark:border-green-800/40"
            >
              <Upload :size="13" />
              {{ t('sync_push') }}
            </button>
          </Tooltip>
        </div>
      </div>

      <!-- Pending update banner -->
      <div v-if="syncStatus.pendingUpdate" :class="s.pendingBanner()">
        <div class="flex items-start sm:items-center gap-3 flex-col sm:flex-row">
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-blue-800 dark:text-blue-200">{{ t('sync_remote_newer') }}</p>
            <p class="text-[10px] text-blue-600 dark:text-blue-400 mt-0.5">{{ t('sync_remote_newer_desc') }}</p>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button
              @click="handlePull"
              :disabled="syncStatus.syncing"
              class="px-3 py-1.5 rounded-lg bg-blue-600 text-white text-xs font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
            >
              {{ t('sync_pull') }}
            </button>
            <button
              @click="dismissPendingUpdate()"
              class="px-3 py-1.5 rounded-lg text-blue-600 dark:text-blue-400 text-xs font-medium hover:bg-blue-100 dark:hover:bg-blue-900/40 transition-colors"
            >
              {{ t('sync_dismiss') }}
            </button>
          </div>
        </div>
      </div>

      <!-- Error -->
      <div v-if="syncStatus.error" :class="s.errorBanner()">
        <p class="text-xs text-red-600 dark:text-red-400">{{ syncStatus.error }}</p>
      </div>

      <!-- Disconnect + Change provider row -->
      <div class="flex items-center gap-2 mt-3">
        <button
          @click="handleDisconnect"
          :class="[s.actionBtn(), s.dangerBtn()]"
        >
          <CloudOff :size="13" />
          {{ t('sync_disconnect') }}
        </button>
        <button
          v-if="availableProviders.length"
          @click="showChangeProvider = !showChangeProvider"
          :class="s.actionBtn()"
          class="text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)] border border-[var(--color-border)]"
        >
          <RefreshCw :size="13" />
          {{ t('sync_change_provider') }}
          <ChevronDown :size="13" class="transition-transform duration-200" :style="{ transform: showChangeProvider ? 'rotate(180deg)' : '' }" />
        </button>
      </div>
    </template>

    <!-- ============ PROVIDER LIST ============ -->
    <Transition
      enter-active-class="transition-all duration-200 ease-out"
      enter-from-class="max-h-0 opacity-0"
      enter-to-class="max-h-[1000px] opacity-100"
      leave-active-class="transition-all duration-150 ease-in"
      leave-from-class="max-h-[1000px] opacity-100"
      leave-to-class="max-h-0 opacity-0"
    >
      <div class="space-y-2 overflow-hidden" v-if="activeProviderInfo ? showChangeProvider : true" :class="activeProviderInfo ? 'mt-3' : ''">
        <div
          v-for="provider in (activeProviderInfo ? availableProviders : allProviders)"
          :key="provider.type"
          :class="s.providerCard()"
        >
          <!-- Provider header row -->
          <div :class="s.providerRow()" @click="toggleExpand(provider.type)">
            <img :src="provider.icon" :alt="provider.name" :class="s.providerIcon()" />
            <div class="flex-1 min-w-0">
              <p :class="s.providerName()">{{ provider.name }}</p>
              <p :class="s.providerDesc()">
                {{ provider.type === 'icloud' ? 'macOS / iOS' :
                   provider.type === 'webdav' ? 'Nextcloud, ownCloud, Synology…' :
                   provider.type === 'onedrive' ? 'Microsoft' :
                   provider.type === 'gdrive' ? 'Google' :
                   provider.type === 'dropbox' ? 'Dropbox Inc.' : '' }}
              </p>
            </div>
            <!-- Quick connect for icloud (no credentials needed) -->
            <button
              v-if="provider.type === 'icloud'"
              @click.stop="handleConnect(provider.type)"
              :disabled="isConnecting"
              :class="s.connectBtn()"
            >
              {{ isConnecting ? '...' : t('sync_connect') }}
            </button>
            <ChevronDown
              v-else
              :size="16"
              :class="s.providerChevron()"
              :style="{ transform: expandedProvider === provider.type ? 'rotate(180deg)' : '' }"
            />
          </div>

          <!-- Expanded credentials -->
          <Transition
            enter-active-class="transition-all duration-200 ease-out"
            enter-from-class="max-h-0 opacity-0"
            enter-to-class="max-h-96 opacity-100"
            leave-active-class="transition-all duration-150 ease-in"
            leave-from-class="max-h-96 opacity-100"
            leave-to-class="max-h-0 opacity-0"
          >
            <div v-if="expandedProvider === provider.type && provider.type !== 'icloud'" class="overflow-hidden">
              <div :class="s.credForm()">
                <div :class="s.credFormInner()">
                  <!-- Google Drive -->
                  <template v-if="provider.type === 'gdrive'">
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_client_id') }}</label>
                      <input v-model="gdriveClientId" type="text" :class="s.credInput()" placeholder="xxxx.apps.googleusercontent.com" />
                    </div>
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_client_secret') }}</label>
                      <input v-model="gdriveClientSecret" type="password" :class="s.credInput()" />
                    </div>
                    <div :class="s.credActions()">
                      <button @click="saveCredentials('gdrive')" :disabled="!gdriveClientId || !gdriveClientSecret" :class="s.saveBtn()">
                        <Save :size="12" /> {{ t('sync_save_credentials') }}
                      </button>
                      <button @click="handleConnect('gdrive')" :disabled="isConnecting || !canConnect('gdrive')" :class="s.connectBtn()">
                        {{ isConnecting ? '...' : t('sync_connect') }}
                      </button>
                    </div>
                  </template>

                  <!-- Dropbox -->
                  <template v-if="provider.type === 'dropbox'">
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_app_key') }}</label>
                      <input v-model="dropboxAppKey" type="text" :class="s.credInput()" />
                    </div>
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_app_secret') }}</label>
                      <input v-model="dropboxAppSecret" type="password" :class="s.credInput()" />
                    </div>
                    <div :class="s.credActions()">
                      <button @click="saveCredentials('dropbox')" :disabled="!dropboxAppKey || !dropboxAppSecret" :class="s.saveBtn()">
                        <Save :size="12" /> {{ t('sync_save_credentials') }}
                      </button>
                      <button @click="handleConnect('dropbox')" :disabled="isConnecting || !canConnect('dropbox')" :class="s.connectBtn()">
                        {{ isConnecting ? '...' : t('sync_connect') }}
                      </button>
                    </div>
                  </template>

                  <!-- OneDrive -->
                  <template v-if="provider.type === 'onedrive'">
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_client_id') }}</label>
                      <input v-model="onedriveClientId" type="text" :class="s.credInput()" placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" />
                    </div>
                    <div :class="s.credActions()">
                      <button @click="saveCredentials('onedrive')" :disabled="!onedriveClientId" :class="s.saveBtn()">
                        <Save :size="12" /> {{ t('sync_save_credentials') }}
                      </button>
                      <button @click="handleConnect('onedrive')" :disabled="isConnecting || !canConnect('onedrive')" :class="s.connectBtn()">
                        {{ isConnecting ? '...' : t('sync_connect') }}
                      </button>
                    </div>
                  </template>

                  <!-- WebDAV -->
                  <template v-if="provider.type === 'webdav'">
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_webdav_url') }}</label>
                      <input v-model="webdavUrl" type="url" :class="s.credInput()" placeholder="https://cloud.example.com/remote.php/dav/files/user" />
                    </div>
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_webdav_username') }}</label>
                      <input v-model="webdavUsername" type="text" :class="s.credInput()" />
                    </div>
                    <div>
                      <label :class="s.credLabel()">{{ t('sync_webdav_password') }}</label>
                      <input v-model="webdavPassword" type="password" :class="s.credInput()" />
                    </div>
                    <div :class="s.credActions()">
                      <button @click="saveCredentials('webdav')" :disabled="!webdavUrl || !webdavUsername" :class="s.saveBtn()">
                        <Save :size="12" /> {{ t('sync_save_credentials') }}
                      </button>
                      <button @click="handleConnect('webdav')" :disabled="isConnecting || !canConnect('webdav')" :class="s.connectBtn()">
                        {{ isConnecting ? '...' : t('sync_connect') }}
                      </button>
                    </div>
                  </template>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>
    </Transition>
  </section>
</template>
