<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import {
  activateProfile,
  loadManagerState,
  saveProfiles,
  type ManagerState,
  type Profile,
  type ProfileType,
} from "../services/manager";

type ProfileForm = {
  id: string;
  name: string;
  apiKey: string;
  baseUrl: string;
  profileType: ProfileType;
};

const state = ref<ManagerState | null>(null);
const isLoading = ref(true);
const isSaving = ref(false);
const switchingId = ref<string | null>(null);
const message = ref("");
const editingId = ref<string | null>(null);
const form = reactive<ProfileForm>(emptyForm());

const profiles = computed(() => state.value?.profiles ?? []);
const activeCodexProfileId = computed(() => state.value?.activeCodexProfileId ?? null);
const activeClaudeProfileId = computed(() => state.value?.activeClaudeProfileId ?? null);
function isActiveProfile(profile: Profile) {
  if (profile.profileType === 'claude') return profile.id === activeClaudeProfileId.value;
  return profile.id === activeCodexProfileId.value;
}
const isBusy = computed(
  () => isLoading.value || isSaving.value || switchingId.value !== null,
);
const currentProfile = computed(() => {
  const targetId = editingId.value ?? activeCodexProfileId.value ?? activeClaudeProfileId.value;
  return profiles.value.find((profile) => profile.id === targetId) ?? null;
});
const editorHeading = computed(() =>
  editingId.value ? "编辑账号" : currentProfile.value ? "当前账号" : "新建账号",
);
const editorSubheading = computed(() => currentProfile.value?.name ?? "未命名账号");

onMounted(async () => {
  await refreshState();
});

function emptyForm(): ProfileForm {
  return {
    id: "",
    name: "",
    apiKey: "",
    baseUrl: "",
    profileType: "codex",
  };
}

function resetForm(clearMessage = true) {
  editingId.value = null;
  Object.assign(form, emptyForm());
  if (clearMessage) {
    message.value = "";
  }
}

function startNewProfile() {
  resetForm();
}

function editProfile(profile: Profile) {
  editingId.value = profile.id;
  Object.assign(form, profile);
  message.value = "";
}

async function handleSave() {
  message.value = "";

  if (!form.name.trim() || !form.apiKey.trim() || !form.baseUrl.trim()) {
    message.value = "请先填写名称、API Key 和 Base URL";
    return;
  }

  const nextProfile: Profile = {
    id: editingId.value ?? createProfileId(),
    name: form.name.trim(),
    apiKey: form.apiKey.trim(),
    baseUrl: form.baseUrl.trim(),
    profileType: form.profileType,
  };

  const nextProfiles = editingId.value
    ? profiles.value.map((profile) =>
        profile.id === editingId.value ? nextProfile : profile,
      )
    : [...profiles.value, nextProfile];

  isSaving.value = true;
  try {
    state.value = await saveProfiles(nextProfiles);
    message.value = `已保存到本地列表：${nextProfile.name}。切换账号后才会同步到 Codex 配置。`;
    resetForm(false);
  } catch (error) {
    message.value = toMessage(error);
  } finally {
    isSaving.value = false;
  }
}

async function removeProfile(profileId: string) {
  const profile = profiles.value.find((item) => item.id === profileId);
  if (!profile) {
    return;
  }

  isSaving.value = true;
  message.value = "";
  try {
    state.value = await saveProfiles(
      profiles.value.filter((item) => item.id !== profileId),
    );
    if (editingId.value === profileId) {
      resetForm(false);
    }
    message.value = `已从本地列表删除 ${profile.name}`;
  } catch (error) {
    message.value = toMessage(error);
  } finally {
    isSaving.value = false;
  }
}

async function switchProfile(profileId: string) {
  switchingId.value = profileId;
  message.value = "";
  try {
    state.value = await activateProfile(profileId);
    const active = profiles.value.find((profile) => profile.id === profileId);
    message.value = active ? `已切换到 ${active.name}，Codex 配置已同步。` : "切换完成";
  } catch (error) {
    message.value = toMessage(error);
  } finally {
    switchingId.value = null;
  }
}

async function refreshState() {
  isLoading.value = true;
  message.value = "";
  try {
    state.value = await loadManagerState();
  } catch (error) {
    message.value = toMessage(error);
  } finally {
    isLoading.value = false;
  }
}

function createProfileId() {
  return `profile-${Date.now()}`;
}

function toMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message;
  }
  return typeof error === "string" ? error : "操作失败，请稍后再试";
}
</script>

<template>
  <section class="manager-shell">
    <header class="toolbar">
      <div class="toolbar-title">
        <div class="brand-mark">C</div>
        <span class="toolbar-name">Codex API Manager</span>
      </div>
      <div class="toolbar-actions">
        <button class="secondary-button" data-testid="new-profile" type="button" @click="startNewProfile" :disabled="isBusy">+ 新建</button>
        <button class="secondary-button" type="button" @click="refreshState" :disabled="isLoading">{{ isLoading ? '读取中...' : '刷新' }}</button>
      </div>
    </header>

    <div class="workspace" data-testid="workspace-shell">
      <aside class="sidebar" data-testid="profile-sidebar">
        <div v-if="isLoading" class="sidebar-empty">正在读取...</div>
        <div v-else-if="profiles.length === 0" class="sidebar-empty">还没有账号，先新建一条。</div>
        <ul v-else class="profile-list">
          <li
            v-for="profile in profiles"
            :key="profile.id"
            class="profile-card"
            :class="{ active: isActiveProfile(profile), editing: profile.id === editingId }"
          >
            <button class="profile-main" type="button" @click="editProfile(profile)">
              <div class="profile-line">
                <strong class="profile-name">{{ profile.name }}</strong>
                <span class="type-badge" :class="profile.profileType">{{ profile.profileType === 'claude' ? 'Claude' : 'Codex' }}</span>
              </div>
              <p class="profile-url">{{ profile.baseUrl }}</p>
              <span v-if="isActiveProfile(profile)" class="active-tag">✓ 生效中</span>
            </button>
          </li>
        </ul>
      </aside>

      <section class="editor" data-testid="profile-editor">
        <div class="editor-head">
          <div class="editor-title">
            <span class="section-label">{{ editorHeading }}</span>
            <h2>{{ editorSubheading }}</h2>
          </div>
          <div class="editor-head-actions">
            <button v-if="editingId" class="icon-button danger" type="button" @click="removeProfile(editingId)">删除</button>
            <button
              v-if="currentProfile && !isActiveProfile(currentProfile)"
              class="primary-button" type="button" data-testid="switch-current-profile"
              @click="switchProfile(currentProfile.id)"
              :disabled="switchingId !== null"
            >{{ switchingId === currentProfile.id ? '切换中...' : '切换为当前' }}</button>
          </div>
        </div>

        <form class="editor-form" @submit.prevent="handleSave">
          <div class="type-selector">
            <label class="type-option" :class="{ selected: form.profileType === 'codex' }">
              <input type="radio" v-model="form.profileType" value="codex" />
              <span>Codex</span>
            </label>
            <label class="type-option" :class="{ selected: form.profileType === 'claude' }">
              <input type="radio" v-model="form.profileType" value="claude" />
              <span>Claude Code</span>
            </label>
          </div>

          <div class="field-row">
            <label class="field">
              <span>名称</span>
              <input v-model="form.name" placeholder="例如：主账号" />
            </label>
            <label class="field">
              <span>Base URL</span>
              <input v-model="form.baseUrl" placeholder="https://api.example.com" />
            </label>
          </div>

          <label class="field">
            <span>API Key</span>
            <input v-model="form.apiKey" data-testid="api-key-input" placeholder="sk-..." />
          </label>

          <div class="status-bar" v-if="state">
            <div class="path-stack">
              <template v-if="form.profileType === 'codex'">
                <div class="path-row">
                  <span class="path-label">auth.json</span>
                  <code>{{ state.codexPaths.authJson }}</code>
                </div>
                <div class="path-row">
                  <span class="path-label">config.toml</span>
                  <code>{{ state.codexPaths.configToml }}</code>
                </div>
              </template>
              <template v-else>
                <div class="path-row">
                  <span class="path-label">settings.json</span>
                  <code>{{ state.codexPaths.claudeSettingsJson }}</code>
                </div>
              </template>
            </div>
          </div>

          <div class="form-footer">
            <p v-if="message" class="status-text" data-testid="status-message">{{ message }}</p>
            <button class="primary-button" type="submit" :disabled="isSaving || isLoading">
              {{ isSaving ? '保存中...' : editingId ? '保存修改' : '保存账号' }}
            </button>
          </div>
        </form>
      </section>
    </div>
  </section>
</template>

<style scoped>
/* === Tokens === */
:root {
  --sidebar-bg: #f0f4f8;
  --sidebar-text: #1e293b;
  --sidebar-text-muted: #64748b;
  --sidebar-card-bg: #ffffff;
  --sidebar-tag-bg: rgba(37, 99, 235, 0.1);
  --sidebar-tag-color: #1d4ed8;
}

/* === Layout === */
.manager-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 12px;
  gap: 10px;
  max-width: 1200px;
  margin: 0 auto;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-card);
  box-shadow: var(--shadow-sm);
  flex-shrink: 0;
}

.toolbar-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: linear-gradient(145deg, #0f172a, #1e293b);
  color: #f8fafc;
  font-weight: 700;
  font-size: 13px;
  flex-shrink: 0;
}

.toolbar-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.toolbar-actions {
  display: flex;
  gap: 6px;
}

.workspace {
  display: grid;
  grid-template-columns: 220px 1fr;
  gap: 10px;
  flex: 1;
  min-height: 0;
}

/* === Sidebar === */
.sidebar {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 8px;
  background: var(--sidebar-bg);
  color: var(--sidebar-text);
  overflow-y: auto;
  box-shadow: var(--shadow-sm);
}

.sidebar-empty {
  padding: 16px;
  border-radius: var(--radius-sm);
  background: var(--color-gray-100);
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
}

.profile-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.profile-card {
  border-radius: var(--radius-sm);
  background: var(--sidebar-card-bg);
  border: 1px solid var(--border-default);
  overflow: hidden;
  transition: border-color var(--duration-fast) var(--ease-default),
    box-shadow var(--duration-fast) var(--ease-default);
}

.profile-card:hover {
  border-color: var(--border-strong);
  box-shadow: var(--shadow-xs);
}

.profile-card.active {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px rgba(37, 99, 235, 0.12);
}

.profile-card.editing .profile-main {
  background: var(--color-gray-50);
}

.profile-main {
  width: 100%;
  border: 0;
  padding: 8px 10px;
  text-align: left;
  color: inherit;
  background: transparent;
  cursor: pointer;
  transition: background var(--duration-fast) var(--ease-default);
}

.profile-main:hover {
  background: var(--color-gray-50);
}

.profile-line {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}

.profile-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.profile-url {
  margin: 0;
  color: var(--text-muted);
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.active-tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  background: var(--color-success-bg);
  color: var(--color-success);
  font-size: 10px;
  font-weight: 700;
  white-space: nowrap;
  flex-shrink: 0;
  margin-top: 4px;
  border: 1px solid currentColor;
  letter-spacing: 0.02em;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  padding: 1px 6px;
  border-radius: var(--radius-full);
  font-size: 10px;
  font-weight: 600;
  white-space: nowrap;
  flex-shrink: 0;
}

.type-badge.codex {
  background: rgba(100, 116, 139, 0.12);
  color: var(--color-gray-600);
}

.type-badge.claude {
  background: rgba(217, 119, 6, 0.12);
  color: #b45309;
}

/* === Editor === */
.editor {
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  padding: 16px;
  background: var(--bg-card);
  box-shadow: var(--shadow-sm);
  overflow-y: auto;
}

.editor-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}

.editor-title .section-label {
  margin: 0 0 2px;
  font-size: 11px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.editor-title h2 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.editor-head-actions {
  display: flex;
  gap: 6px;
}

.editor-form {
  display: grid;
  gap: 12px;
}

.type-selector {
  display: flex;
  gap: 6px;
}

.type-option {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-default);
  background: var(--bg-subtle);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-default),
    background var(--duration-fast) var(--ease-default);
}

.type-option input[type="radio"] {
  display: none;
}

.type-option.selected {
  border-color: var(--color-primary);
  background: rgba(37, 99, 235, 0.06);
  color: var(--color-primary);
  font-weight: 600;
}

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.field {
  display: grid;
  gap: 4px;
}

.field span {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.field input {
  width: 100%;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  padding: 8px 10px;
  background: var(--bg-subtle);
  color: var(--text-primary);
  font: inherit;
  font-size: 13px;
  box-shadow: var(--shadow-xs);
  outline: none;
  transition: border-color var(--duration-fast) var(--ease-default),
    box-shadow var(--duration-fast) var(--ease-default);
}

.field input:focus {
  border-color: var(--color-primary);
  background: var(--bg-card);
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

/* === Status Bar === */
.status-bar {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  background: var(--bg-subtle);
  border: 1px solid var(--border-default);
}

.path-stack {
  display: flex;
  gap: 16px;
  min-width: 0;
}

.path-row {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.path-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.path-row code {
  color: var(--text-secondary);
  font-size: 11px;
  word-break: break-all;
  font-family: "IBM Plex Mono", monospace;
}

.form-footer {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  gap: 10px;
}

.status-text {
  margin: 0;
  color: var(--color-warning);
  font-size: 12px;
}

/* === Buttons === */
.primary-button,
.secondary-button,
.icon-button {
  border: 0;
  border-radius: var(--radius-sm);
  min-height: 32px;
  padding: 0 12px;
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background-color var(--duration-fast) var(--ease-default),
    opacity var(--duration-fast) var(--ease-default),
    transform var(--duration-fast) var(--ease-default);
}

.primary-button:hover,
.secondary-button:hover,
.icon-button:hover {
  transform: translateY(-1px);
}

.primary-button:disabled,
.secondary-button:disabled,
.icon-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
  transform: none;
}

.primary-button {
  background: var(--color-primary);
  color: var(--color-primary-fg);
}

.primary-button:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.secondary-button,
.icon-button {
  background: var(--color-gray-100);
  color: var(--color-gray-700);
}

.secondary-button:hover:not(:disabled),
.icon-button:hover:not(:disabled) {
  background: var(--color-gray-200);
}

.icon-button.danger {
  background: var(--color-danger-bg);
  color: var(--color-danger);
}

.icon-button.danger:hover:not(:disabled) {
  filter: brightness(0.92);
}

/* === Responsive === */
@media (max-width: 720px) {
  .workspace {
    grid-template-columns: 1fr;
  }
  .field-row,
  .status-bar {
    flex-direction: column;
    align-items: stretch;
  }
  .status-text {
    max-width: none;
    text-align: left;
  }
}
</style>
