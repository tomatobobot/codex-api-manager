import { invoke } from "@tauri-apps/api/core";

export type ProfileType = 'codex' | 'claude';

export interface Profile {
  id: string;
  name: string;
  apiKey: string;
  baseUrl: string;
  model: string;
  profileType: ProfileType;
}

export interface CodexPaths {
  authJson: string;
  configToml: string;
  claudeSettingsJson: string;
}

export interface ManagerState {
  profiles: Profile[];
  activeCodexProfileId: string | null;
  activeClaudeProfileId: string | null;
  codexPaths: CodexPaths;
}

export function loadManagerState() {
  return invoke<ManagerState>("load_manager_state");
}

export function saveProfiles(profiles: Profile[]) {
  return invoke<ManagerState>("save_profiles", { profiles });
}

export function activateProfile(profileId: string) {
  return invoke<ManagerState>("activate_profile", { profileId });
}
