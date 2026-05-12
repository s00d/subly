<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@/composables/useToast";
import { formatErrorForToast } from "@/utils/formatError";
import type { Subscription, Settings, Currency, PaymentMethod, HouseholdMember, Category, Tag, SubscriptionListItem, SubscriptionCredentialsMeta } from "@/schemas/appData";
import Modal from "@/components/ui/Modal.vue";
import AppInput from "@/components/ui/AppInput.vue";
import AppDatePicker from "@/components/ui/AppDatePicker.vue";
import AppTextarea from "@/components/ui/AppTextarea.vue";
import AppSelect from "@/components/ui/AppSelect.vue";
import AppCheckbox from "@/components/ui/AppCheckbox.vue";
import LogoPicker from "@/components/ui/LogoPicker.vue";
import SecretInput from "@/components/ui/SecretInput.vue";
import TagInput from "@/components/ui/TagInput.vue";
import type { SelectOption } from "@/components/ui/AppSelect.vue";
import { Sparkles, Globe, Loader2, KeyRound, ClipboardPaste, ImagePlus, DownloadCloud } from "@lucide/vue";
import { resolveFaviconFromInputUrl } from "@/services/logoClient";
import { getNextCycleDate as getNextCycleDateBackend, upsertSubscription } from "@/services/subscriptionsClient";
import {
  subscriptionCredentialsGet,
  subscriptionTotpDecodeQrBase64,
  subscriptionTotpImportOtpauth,
} from "@/services/subscriptionCredentialsClient";
import { subscriptionFormFieldsSchema, coerceSubscriptionFormForValidation } from "@/schemas/zod/subscriptionForm";
import { useZodLiveForm } from "@/composables/useZodLiveForm";
import type { ZodFieldMeta } from "@/composables/useZodErrors";

/**
 * Subset of `Subscription` fields that can be auto-filled by the AI quick-add
 * flow (or any future "create from template" feature). Every field is optional;
 * unspecified keys fall back to `createDefaultForm()`.
 */
export type SubscriptionPrefill = Partial<
  Pick<
    Subscription,
    | "name"
    | "logo"
    | "price"
    | "currencyId"
    | "cycle"
    | "frequency"
    | "categoryId"
    | "paymentMethodId"
    | "startDate"
    | "nextPayment"
    | "notes"
    | "url"
    | "tags"
  >
>;

const props = defineProps<{
  show: boolean;
  /**
   * Accept both `Subscription` and the richer `SubscriptionListItem` because
   * the page always opens the form with a list row, and we need its
   * `credentialsMeta` to know which Secret fields are already populated.
   */
  editSubscription?: Subscription | SubscriptionListItem | null;
  /** One-shot AI-supplied initial values; consumed once on modal open. */
  prefill?: SubscriptionPrefill | null;
  lookupData: {
    settings: Settings;
    currencies: Currency[];
    paymentMethods: PaymentMethod[];
    household: HouseholdMember[];
    categories: Category[];
    tags: Tag[];
  };
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
}>();

const settings = ref<Settings | null>(null);
const currencies = ref<Currency[]>([]);
const paymentMethods = ref<PaymentMethod[]>([]);
const household = ref<HouseholdMember[]>([]);
const categories = ref<Category[]>([]);
const tags = ref<Tag[]>([]);
const { t } = useI18n();
const { toast } = useToast();
function createDefaultForm(): Partial<Subscription> {
  return {
    name: "",
    logo: "",
    price: 0,
    currencyId: settings.value?.mainCurrencyId || "cur-2",
    nextPayment: new Date().toISOString().split("T")[0],
    startDate: new Date().toISOString().split("T")[0],
    cycle: 3,
    frequency: 1,
    notes: "",
    paymentMethodId: settings.value?.defaultPaymentMethodId || paymentMethods.value.find((p) => p.enabled)?.id || "",
    payerUserId: household.value[0]?.id || "",
    categoryId: settings.value?.defaultCategoryId || "cat-1",
    notify: true,
    notifyDaysBefore: -1,
    inactive: false,
    autoRenew: true,
    url: "",
    cancellationDate: null,
    replacementSubscriptionId: null,
    tags: [],
    favorite: false,
    credentials: emptyCreds(),
  };
}

const form = ref<Partial<Subscription>>(createDefaultForm());

const subscriptionFieldMeta: ZodFieldMeta = {
  name: "string",
  price: "number",
  currencyId: "string",
  nextPayment: "date",
  startDate: "date",
  frequency: "number",
  notifyDaysBefore: "number",
  cycle: "number",
};

function buildSubscriptionValidationInput(): Record<string, unknown> {
  const base = props.editSubscription
    ? { ...props.editSubscription, ...form.value }
    : { ...form.value };
  return coerceSubscriptionFormForValidation(base as Record<string, unknown>);
}

const {
  errors: subErrors,
  markDirty: markSubscriptionFieldDirty,
  clearDirty: clearSubscriptionZodDirty,
  validateStrict: validateSubscriptionStrict,
  watchSource: watchSubscriptionZod,
} = useZodLiveForm({
  getValues: buildSubscriptionValidationInput,
  schema: subscriptionFormFieldsSchema,
  fieldMeta: subscriptionFieldMeta,
  t,
  guardEmptyRequiredStrings: true,
  guardedStringFields: ["name"],
});

watchSubscriptionZod(form);

const isResolvingIcon = ref(false);
const qrFileInput = ref<HTMLInputElement | null>(null);

/**
 * "Did the user actually edit (or explicitly load) the credentials block in
 * this dialog session?" When `false` at submit time we omit the `credentials`
 * field entirely, which tells the backend's `credentials_apply_optional` to
 * leave the keyring untouched — the user only edited unrelated subscription
 * fields and shouldn't lose stored secrets.
 */
const credentialsTouched = ref(false);
const isLoadingSavedCreds = ref(false);

/** Non-secret bitmap echoed back by the list endpoint; drives Secret masks. */
const credentialsMeta = computed<SubscriptionCredentialsMeta>(() => {
  const edit = props.editSubscription as SubscriptionListItem | null | undefined;
  return (
    edit?.credentialsMeta ?? { hasLogin: false, hasPassword: false, hasTotp: false }
  );
});

const hasAnySavedCredential = computed(
  () =>
    credentialsMeta.value.hasLogin ||
    credentialsMeta.value.hasPassword ||
    credentialsMeta.value.hasTotp,
);

function emptyCreds() {
  return { login: "", password: "", totpSecret: "" };
}

function resetForm() {
  form.value = createDefaultForm();
}

function markCredsTouched() {
  credentialsTouched.value = true;
}

/**
 * Pull the saved credentials out of the keyring (one OS prompt) and drop
 * them into the form so the user can edit in place instead of overwriting
 * from scratch. Auto-marks the section as touched so the next submit sends
 * the values back, even if the user only nudged one field.
 */
async function loadSavedCredentials() {
  if (!props.editSubscription) return;
  isLoadingSavedCreds.value = true;
  try {
    const fetched = await subscriptionCredentialsGet(props.editSubscription.id);
    if (!fetched) {
      toast(t("credentials_not_found"), "error");
      return;
    }
    form.value.credentials = {
      login: fetched.login ?? "",
      password: fetched.password ?? "",
      totpSecret: fetched.totpSecret ?? "",
    };
    credentialsTouched.value = true;
    toast(t("credentials_loaded"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isLoadingSavedCreds.value = false;
  }
}

function applyPrefill(target: Partial<Subscription>, source: SubscriptionPrefill): Partial<Subscription> {
  const next: Partial<Subscription> = { ...target };
  // Only assign keys the AI actually returned; never overwrite defaults with empty/undefined values.
  for (const [key, value] of Object.entries(source) as Array<[keyof SubscriptionPrefill, unknown]>) {
    if (value === undefined || value === null) continue;
    if (typeof value === "string" && value.trim() === "") continue;
    if (typeof value === "number" && !Number.isFinite(value)) continue;
    (next as Record<string, unknown>)[key] = value;
  }
  return next;
}

watch(
  () => props.show,
  (val) => {
    if (!val) return;
    loadLookupData();
    clearSubscriptionZodDirty();
    credentialsTouched.value = false;
    if (props.editSubscription) {
      // Credentials no longer ship with the list row — they live in the
      // keyring and are fetched only when the user clicks "Load saved".
      // Start the form with blank credentials and let `credentialsMeta`
      // drive the SecretInput masks instead.
      form.value = {
        ...props.editSubscription,
        credentials: emptyCreds(),
      };
    } else {
      resetForm();
      if (props.prefill) {
        form.value = applyPrefill(form.value, props.prefill);
      }
    }
  },
);

const isEdit = computed(() => !!props.editSubscription);
const title = computed(() => isEdit.value ? t("edit_subscription") : t("add_subscription"));

// Select options
const currencyOptions = computed<SelectOption[]>(() =>
  currencies.value.map((c) => ({ value: c.id, label: `${c.name} (${c.code})` }))
);

const cycleOptions = computed<SelectOption[]>(() => [
  { value: 1, label: t("days") },
  { value: 2, label: t("weeks") },
  { value: 3, label: t("months") },
  { value: 4, label: t("years") },
]);

const paymentMethodOptions = computed<SelectOption[]>(() =>
  paymentMethods.value.filter((pm) => pm.enabled).map((pm) => ({ value: pm.id, label: pm.name, icon: pm.icon }))
);

const payerOptions = computed<SelectOption[]>(() =>
  household.value.map((m) => ({ value: m.id, label: m.name }))
);

const categoryOptions = computed<SelectOption[]>(() =>
  [...categories.value].sort((a, b) => a.sortOrder - b.sortOrder).map((c) => ({ value: c.id, label: c.name, icon: c.icon || undefined }))
);

async function calculateNextPayment() {
  if (!form.value.startDate || !form.value.cycle || !form.value.frequency) return;
  const today = new Date().toISOString().split("T")[0];
  let next = form.value.startDate;
  while (next < today) {
    next = await getNextCycleDateBackend(next, Number(form.value.cycle), Number(form.value.frequency));
  }
  form.value.nextPayment = next;
}

async function applyDomainIcon() {
  if (isResolvingIcon.value) return;
  isResolvingIcon.value = true;
  try {
    const faviconUrl = await resolveFaviconFromInputUrl(form.value.url || "");
    if (!faviconUrl) {
      toast(t("favicon_load_failed"), "error");
      return;
    }
    form.value.logo = faviconUrl;
    toast(t("favicon_loaded_from_domain"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  } finally {
    isResolvingIcon.value = false;
  }
}

async function handleSubmit() {
  if (!validateSubscriptionStrict()) {
    toast(t("fill_required_fields"), "error");
    return;
  }

  const base = isEdit.value && props.editSubscription
    ? { ...props.editSubscription, ...form.value }
    : { ...form.value, id: crypto.randomUUID(), createdAt: new Date().toISOString() };

  const cred = form.value.credentials ?? emptyCreds();
  const raw: Record<string, unknown> = {
    ...base,
    price: Number(base.price) || 0,
    frequency: Number(base.frequency) || 1,
    notifyDaysBefore: Number(base.notifyDaysBefore ?? 1),
    cycle: Number(base.cycle) || 3,
  };
  if (credentialsTouched.value) {
    // Send the (possibly partially-empty) credentials. The backend treats
    // an all-empty triplet as "delete the keyring entry".
    raw.credentials = {
      login: cred.login.trim(),
      password: cred.password,
      totpSecret: cred.totpSecret.trim(),
    };
  }
  // When `credentials` is absent, `credentials_apply_optional` early-returns
  // and the keyring entry stays exactly as it was — important so that
  // editing a subscription's name (etc.) without touching the secrets block
  // doesn't wipe stored logins.

  try {
    await upsertSubscription(raw);
    toast(t("success"));
    emit("saved");
    emit("close");
  } catch (e) {
    console.error("Subscription save failed:", e);
    toast(formatErrorForToast(e, t), "error");
  }
}

async function pasteOtpauthFromClipboard() {
  try {
    const text = await navigator.clipboard.readText();
    if (!text?.trim()) {
      toast(t("otpauth_clipboard_empty"), "error");
      return;
    }
    const imported = await subscriptionTotpImportOtpauth(text.trim());
    if (!form.value.credentials) form.value.credentials = emptyCreds();
    form.value.credentials.totpSecret = imported.totpSecret;
    markCredsTouched();
    toast(t("totp_imported"));
  } catch (e) {
    toast(formatErrorForToast(e, t), "error");
  }
}

function triggerQrPicker() {
  qrFileInput.value?.click();
}

async function onQrFileChange(ev: Event) {
  const input = ev.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  const reader = new FileReader();
  reader.onload = async () => {
    const data = reader.result as string;
    try {
      const imported = await subscriptionTotpDecodeQrBase64(data);
      if (!form.value.credentials) form.value.credentials = emptyCreds();
      form.value.credentials.totpSecret = imported.totpSecret;
      markCredsTouched();
      toast(t("totp_imported"));
    } catch (e) {
      toast(formatErrorForToast(e, t), "error");
    }
  };
  reader.readAsDataURL(file);
}

function loadLookupData() {
  settings.value = props.lookupData.settings;
  currencies.value = props.lookupData.currencies;
  paymentMethods.value = props.lookupData.paymentMethods;
  household.value = props.lookupData.household;
  categories.value = props.lookupData.categories;
  tags.value = props.lookupData.tags;
}
</script>

<template>
  <Modal :show="show" :title="title" @close="emit('close')" maxWidth="42rem">
    <form @submit.prevent="handleSubmit()" class="space-y-4 sm:space-y-5">
      <!-- Name + Logo -->
      <div class="flex gap-3 sm:gap-4 items-start">
        <div class="flex-1">
          <AppInput
            :modelValue="form.name || ''"
            @update:modelValue="(v) => { form.name = String(v); markSubscriptionFieldDirty('name'); }"
            :label="t('subscription_name') + ' *'"
            :placeholder="t('subscription_name')"
            :error="subErrors.name"
            required
          />
        </div>
        <div class="pt-5">
          <LogoPicker v-model="form.logo!" />
        </div>
      </div>

      <!-- Price + Currency -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="flex-1">
          <AppInput
            v-model="form.price!"
            type="number"
            :label="t('price') + ' *'"
            :error="subErrors.price"
            step="0.01"
            min="0"
            required
          />
        </div>
        <div class="sm:w-44">
          <AppSelect
            v-model="form.currencyId!"
            :options="currencyOptions"
            :label="t('currencies')"
            :error="subErrors.currencyId"
            searchable
          />
        </div>
      </div>

      <!-- Cycle + Frequency + Auto Renew -->
      <div class="flex flex-wrap gap-3 items-end">
        <div class="w-20">
          <AppInput
            v-model="form.frequency!"
            type="number"
            :label="t('frequency')"
            :error="subErrors.frequency"
            min="1"
            max="366"
          />
        </div>
        <div class="flex-1 min-w-[120px]">
          <AppSelect
            v-model="form.cycle!"
            :options="cycleOptions"
            :label="t('payment_every')"
            :error="subErrors.cycle"
          />
        </div>
        <div class="pb-1">
          <AppCheckbox v-model="form.autoRenew!" :label="t('auto_renewal')" />
        </div>
      </div>

      <!-- Dates -->
      <div class="flex flex-row gap-2 sm:gap-3 items-end">
        <div class="flex-1 min-w-0">
          <AppDatePicker
            v-model="form.startDate!"
            :label="t('start_date')"
            :error="subErrors.startDate"
          />
        </div>
        <button
          type="button"
          @click="calculateNextPayment"
          class="shrink-0 p-2 rounded-lg bg-surface-hover text-text-secondary hover:bg-primary hover:text-white transition-colors"
          :title="t('calculate_next_payment_date')"
        >
          <Sparkles :size="18" />
        </button>
        <div class="flex-1 min-w-0">
          <AppDatePicker
            v-model="form.nextPayment!"
            :label="t('next_payment') + ' *'"
            :error="subErrors.nextPayment"
          />
        </div>
      </div>

      <!-- Payment Method + Payer -->
      <div class="flex flex-col sm:flex-row gap-3">
        <div class="flex-1">
          <AppSelect
            v-model="form.paymentMethodId!"
            :options="paymentMethodOptions"
            :label="t('payment_method')"
          />
        </div>
        <div class="flex-1">
          <AppSelect
            v-model="form.payerUserId!"
            :options="payerOptions"
            :label="t('paid_by')"
          />
        </div>
      </div>

      <!-- Category -->
      <AppSelect
        v-model="form.categoryId!"
        :options="categoryOptions"
        :label="t('category')"
        searchable
      />

      <!-- Tags -->
      <TagInput
        v-model="form.tags!"
        :label="t('tags')"
        :availableTags="tags"
      />

      <!-- URL -->
      <div class="space-y-2">
        <AppInput
          v-model="form.url!"
          :label="t('url')"
          :placeholder="'https://...'"
        />
        <button
          type="button"
          @click="applyDomainIcon"
          :disabled="isResolvingIcon"
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
        >
          <Loader2 v-if="isResolvingIcon" :size="14" class="animate-spin" />
          <Globe v-else :size="14" />
          {{ t("get_icon_from_domain") }}
        </button>
      </div>

      <!-- Credentials (secure storage, not in sync snapshot) -->
      <div class="rounded-xl border border-border p-3 sm:p-4 space-y-3 bg-surface-secondary/50">
        <div class="flex items-center justify-between gap-2">
          <div class="flex items-center gap-2 min-w-0">
            <KeyRound :size="16" class="text-primary shrink-0" />
            <span class="text-sm font-medium text-text-primary">{{ t("credentials_section") }}</span>
          </div>
          <button
            v-if="isEdit && hasAnySavedCredential"
            type="button"
            class="shrink-0 inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover disabled:opacity-50"
            :disabled="isLoadingSavedCreds"
            @click="loadSavedCredentials"
          >
            <Loader2 v-if="isLoadingSavedCreds" :size="12" class="animate-spin" />
            <DownloadCloud v-else :size="12" />
            {{ t("credentials_load_saved") }}
          </button>
        </div>
        <p class="text-xs text-text-muted">{{ t("credentials_secure_hint") }}</p>
        <SecretInput
          :modelValue="form.credentials!.login"
          :has-saved-value="credentialsMeta.hasLogin && !credentialsTouched"
          type="text"
          :label="t('login_username')"
          @update:modelValue="(v: string) => { form.credentials!.login = v; markCredsTouched(); }"
        />
        <SecretInput
          :modelValue="form.credentials!.password"
          :has-saved-value="credentialsMeta.hasPassword && !credentialsTouched"
          type="password"
          :label="t('password')"
          @update:modelValue="(v: string) => { form.credentials!.password = v; markCredsTouched(); }"
        />
        <SecretInput
          :modelValue="form.credentials!.totpSecret"
          :has-saved-value="credentialsMeta.hasTotp && !credentialsTouched"
          type="text"
          :label="t('totp_secret')"
          :placeholder="t('totp_secret_placeholder')"
          @update:modelValue="(v: string) => { form.credentials!.totpSecret = v; markCredsTouched(); }"
        />
        <div class="flex flex-wrap gap-2">
          <button
            type="button"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover"
            @click="pasteOtpauthFromClipboard"
          >
            <ClipboardPaste :size="14" />
            {{ t("paste_otpauth_clipboard") }}
          </button>
          <button
            type="button"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg border border-border text-xs font-medium text-text-secondary hover:bg-surface-hover"
            @click="triggerQrPicker"
          >
            <ImagePlus :size="14" />
            {{ t("load_totp_qr") }}
          </button>
          <input
            ref="qrFileInput"
            type="file"
            accept="image/png,image/jpeg,image/jpg,image/gif,image/webp"
            class="hidden"
            @change="onQrFileChange"
          />
        </div>
      </div>

      <!-- Notes -->
      <AppTextarea
        v-model="form.notes!"
        :label="t('notes')"
        :placeholder="t('notes')"
        :rows="2"
      />

      <!-- Toggles -->
      <div class="flex flex-wrap gap-x-6 gap-y-3 pt-1">
        <AppCheckbox v-model="form.notify!" :label="t('enable_notifications')" />
        <AppCheckbox v-model="form.inactive!" :label="t('inactive')" />
      </div>
    </form>

    <template #footer>
      <div class="flex items-center justify-end gap-2 w-full">
        <button
          @click="emit('close')"
          class="px-4 py-2 rounded-lg border border-border text-sm font-medium text-text-secondary hover:bg-surface-hover transition-colors"
        >{{ t('cancel') }}</button>
        <button
          @click="handleSubmit()"
          class="px-5 py-2 rounded-lg bg-primary text-white text-sm font-medium hover:bg-primary-hover transition-colors"
        >{{ t('save') }}</button>
      </div>
    </template>
  </Modal>
</template>
