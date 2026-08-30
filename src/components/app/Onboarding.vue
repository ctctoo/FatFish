<script setup lang="ts">
import { computed, ref } from "vue";
import { ArrowLeft, ArrowRight, Check, Sparkles } from "lucide-vue-next";
import { useI18n } from "../../i18n";
import { useSettingsStore, type Gender } from "../../stores/settings";

const emit = defineEmits<{ close: [] }>();

const { t, locale } = useI18n();
const settings = useSettingsStore();

const step = ref(0); // 0 欢迎 / 1 称呼 / 2 性别 / 3 工作
const name = ref("");
const gender = ref<Gender>("unspecified");
const occupation = ref("");

const GENDERS: Gender[] = ["male", "female", "unspecified"];

const occupationChips = computed(() =>
  locale.value === "zh"
    ? ["学生", "开发者", "设计师", "教师", "研究者", "产品经理"]
    : ["Student", "Developer", "Designer", "Teacher", "Researcher", "Product Manager"]
);

const canNext = computed(() => step.value !== 1 || name.value.trim().length > 0);

function next() {
  if (!canNext.value) return;
  if (step.value < 3) step.value += 1;
  else finish();
}

function back() {
  if (step.value > 1) step.value -= 1;
}

function finish() {
  settings.profile = {
    name: name.value.trim(),
    gender: gender.value,
    occupation: occupation.value.trim(),
  };
  settings.onboarded = true;
  emit("close");
}

function skip() {
  settings.onboarded = true;
  emit("close");
}
</script>

<template>
  <Teleport to="body">
    <div class="ob-mask">
      <div class="ob-card">
        <div class="ob-logo">
          <img src="../../assets/logo.png" alt="FatFish" />
        </div>

        <!-- 第 0 步：欢迎 -->
        <template v-if="step === 0">
          <h1 class="ob-title">{{ t("onboarding.welcomeTitle") }}</h1>
          <p class="ob-sub">{{ t("onboarding.welcomeSub") }}</p>
          <button class="ob-btn primary" @click="step = 1">
            <Sparkles :size="16" :stroke-width="1.8" />
            {{ t("onboarding.start") }}
          </button>
        </template>

        <!-- 第 1 步：称呼 -->
        <template v-else-if="step === 1">
          <h1 class="ob-title">{{ t("onboarding.nameTitle") }}</h1>
          <p class="ob-sub">{{ t("onboarding.nameSub") }}</p>
          <input
            v-model="name"
            class="ob-input"
            :placeholder="t('onboarding.namePlaceholder')"
            maxlength="30"
            autofocus
            @keydown.enter="next"
          />
        </template>

        <!-- 第 2 步：性别 -->
        <template v-else-if="step === 2">
          <h1 class="ob-title">{{ t("onboarding.genderTitle") }}</h1>
          <p class="ob-sub">{{ t("onboarding.genderSub") }}</p>
          <div class="ob-genders">
            <button
              v-for="g in GENDERS"
              :key="g"
              class="ob-gender"
              :class="{ active: gender === g }"
              @click="gender = g"
            >
              <Check v-if="gender === g" :size="15" :stroke-width="2.2" />
              {{ t(`gender.${g}`) }}
            </button>
          </div>
        </template>

        <!-- 第 3 步：工作 -->
        <template v-else>
          <h1 class="ob-title">{{ t("onboarding.workTitle") }}</h1>
          <p class="ob-sub">{{ t("onboarding.workSub") }}</p>
          <input
            v-model="occupation"
            class="ob-input"
            :placeholder="t('onboarding.workPlaceholder')"
            maxlength="30"
            @keydown.enter="finish"
          />
          <div class="ob-chips">
            <button
              v-for="chip in occupationChips"
              :key="chip"
              class="ob-chip"
              @click="occupation = chip"
            >
              {{ chip }}
            </button>
          </div>
        </template>

        <!-- 底部导航 -->
        <div class="ob-footer">
          <button class="ob-skip" @click="skip">{{ t("onboarding.skip") }}</button>
          <div class="ob-dots">
            <span
              v-for="i in 3"
              :key="i"
              class="ob-dot"
              :class="{ on: step === i }"
            ></span>
          </div>
          <div class="ob-nav">
            <button v-if="step > 1" class="ob-btn ghost" @click="back">
              <ArrowLeft :size="15" :stroke-width="1.8" />
              {{ t("onboarding.back") }}
            </button>
            <button v-if="step > 0" class="ob-btn primary" :disabled="!canNext" @click="next">
              {{ step === 3 ? t("onboarding.finish") : t("onboarding.next") }}
              <ArrowRight :size="15" :stroke-width="1.8" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.ob-mask {
  position: fixed;
  inset: 0;
  z-index: 300;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg);
}

.ob-card {
  width: min(440px, 90vw);
  min-height: 380px;
  display: flex;
  flex-direction: column;
  padding: 40px 36px 28px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 20px;
  box-shadow: var(--shadow-modal);
  animation: ob-in 0.3s cubic-bezier(0.22, 1, 0.36, 1);
}

@keyframes ob-in {
  from {
    opacity: 0;
    transform: translateY(16px) scale(0.98);
  }
}

.ob-logo {
  width: 56px;
  height: 56px;
  margin-bottom: 22px;
}
.ob-logo img {
  width: 100%;
  height: 100%;
  border-radius: 14px;
}

.ob-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 8px;
}

.ob-sub {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text-secondary);
  margin: 0 0 24px;
}

.ob-input {
  width: 100%;
  padding: 11px 14px;
  font-size: 15px;
  color: var(--text-primary);
  background: var(--surface-muted);
  border: 1px solid var(--border);
  border-radius: 10px;
  outline: none;
  transition: border-color 0.15s, background 0.15s;
}
.ob-input:focus {
  border-color: var(--accent);
  background: var(--surface);
}

.ob-genders {
  display: flex;
  gap: 10px;
}
.ob-gender {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 11px 0;
  font-size: 14px;
  color: var(--text-secondary);
  background: var(--surface-muted);
  border: 1px solid var(--border);
  border-radius: 10px;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s, color 0.15s;
}
.ob-gender:hover {
  border-color: var(--border-strong);
}
.ob-gender.active {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

.ob-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 14px;
}
.ob-chip {
  padding: 5px 12px;
  font-size: 13px;
  color: var(--text-secondary);
  background: none;
  border: 1px solid var(--border);
  border-radius: 999px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}
.ob-chip:hover {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-soft);
}

.ob-footer {
  display: flex;
  align-items: center;
  margin-top: auto;
  padding-top: 28px;
}
.ob-skip {
  font-size: 13px;
  color: var(--text-tertiary);
  background: none;
  border: none;
  cursor: pointer;
}
.ob-skip:hover {
  color: var(--text-secondary);
}

.ob-dots {
  display: flex;
  gap: 6px;
  margin: 0 auto;
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
}
.ob-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--border-strong);
  transition: background 0.2s, width 0.2s;
}
.ob-dot.on {
  background: var(--accent);
  width: 18px;
  border-radius: 4px;
}

.ob-nav {
  display: flex;
  gap: 8px;
  margin-left: auto;
}

.ob-btn {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 9px 18px;
  font-size: 14px;
  font-weight: 500;
  border-radius: 10px;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s, transform 0.1s;
}
.ob-btn:active {
  transform: scale(0.98);
}
.ob-btn.primary {
  color: #fff;
  background: var(--accent);
  border: none;
}
.ob-btn.primary:hover {
  background: color-mix(in srgb, var(--accent) 88%, #000);
}
.ob-btn.primary:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.ob-btn.ghost {
  color: var(--text-secondary);
  background: none;
  border: 1px solid var(--border);
}
.ob-btn.ghost:hover {
  border-color: var(--border-strong);
  color: var(--text-primary);
}

.ob-card {
  position: relative;
}
</style>
