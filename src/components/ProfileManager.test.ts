import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import ProfileManager from "./ProfileManager.vue";
import {
  activateProfile,
  loadManagerState,
  saveProfiles,
  type ManagerState,
  type Profile,
} from "../services/manager";

vi.mock("../services/manager", () => ({
  loadManagerState: vi.fn(),
  saveProfiles: vi.fn(),
  activateProfile: vi.fn(),
}));

const mockedLoadManagerState = vi.mocked(loadManagerState);
const mockedSaveProfiles = vi.mocked(saveProfiles);
const mockedActivateProfile = vi.mocked(activateProfile);

function createState(overrides: Partial<ManagerState> = {}): ManagerState {
  const profiles: Profile[] = [
    {
      id: "primary",
      name: "Primary",
      apiKey: "sk-primary",
      baseUrl: "https://primary.example.com",
      model: "",
      profileType: "codex",
    },
  ];

  return {
    profiles,
    activeCodexProfileId: "primary",
    activeClaudeProfileId: null,
    codexPaths: {
      authJson: "C:/Users/demo/.codex/auth.json",
      configToml: "C:/Users/demo/.codex/config.toml",
      claudeSettingsJson: "C:/Users/demo/.claude/settings.json",
    },
    ...overrides,
  };
}

describe("ProfileManager", () => {
  beforeEach(() => {
    mockedLoadManagerState.mockResolvedValue(createState());
    mockedSaveProfiles.mockResolvedValue(createState());
    mockedActivateProfile.mockResolvedValue(createState());
  });

  it("renders the current profiles", async () => {
    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    expect(wrapper.text()).toContain("Primary");
  });

  it("renders the compact workspace structure", async () => {
    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    expect(wrapper.find('[data-testid="workspace-shell"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="profile-sidebar"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="profile-editor"]').exists()).toBe(true);
  });

  it("blocks save when required fields are missing", async () => {
    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    await wrapper.get('[data-testid="new-profile"]').trigger("click");
    await wrapper.get("form").trigger("submit.prevent");
    await nextTick();

    expect(mockedSaveProfiles).not.toHaveBeenCalled();
    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("API Key");
  });

  it("shows that save only writes to the local list", async () => {
    mockedSaveProfiles.mockResolvedValue(
      createState({
        profiles: [
          {
            id: "primary",
            name: "Primary",
            apiKey: "sk-primary",
            baseUrl: "https://primary.example.com",
            model: "",
            profileType: "codex",
          },
          {
            id: "local-only",
            name: "Local Only",
            apiKey: "sk-local",
            baseUrl: "https://local.example.com",
            model: "",
            profileType: "codex",
          },
        ],
      }),
    );

    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    expect(wrapper.get('[data-testid="local-save-note"]').text()).toContain("本地");
    expect(wrapper.get('[data-testid="local-save-note"]').text()).toContain("切换");

    await wrapper.get('[data-testid="new-profile"]').trigger("click");
    await wrapper.get('input[placeholder="例如：主账号"]').setValue("Local Only");
    await wrapper
      .get('input[placeholder="https://api.example.com"]')
      .setValue("https://local.example.com");
    await wrapper.get('[data-testid="api-key-input"]').setValue("sk-local");
    await wrapper.get("form").trigger("submit.prevent");
    await flushUi(wrapper);

    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("本地");
    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("切换");
  });

  it("disables switching while activation is in progress", async () => {
    let resolveSwitch: ((value: ManagerState) => void) | undefined;
    mockedLoadManagerState.mockResolvedValue(
      createState({
        profiles: [
          {
            id: "primary",
            name: "Primary",
            apiKey: "sk-primary",
            baseUrl: "https://primary.example.com",
            model: "",
            profileType: "codex",
          },
          {
            id: "backup",
            name: "Backup",
            apiKey: "sk-backup",
            baseUrl: "https://backup.example.com",
            model: "",
            profileType: "codex",
          },
        ],
        activeCodexProfileId: "primary",
        activeClaudeProfileId: null,
      }),
    );
    mockedActivateProfile.mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveSwitch = resolve;
        }),
    );

    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    await wrapper.findAll(".profile-main")[1].trigger("click");
    await flushUi(wrapper);

    const switchButton = wrapper.get('[data-testid="switch-current-profile"]');
    await switchButton.trigger("click");
    await nextTick();

    expect(switchButton.attributes("disabled")).toBeDefined();

    resolveSwitch?.(
      createState({
        profiles: [
          {
            id: "primary",
            name: "Primary",
            apiKey: "sk-primary",
            baseUrl: "https://primary.example.com",
            model: "",
            profileType: "codex",
          },
          {
            id: "backup",
            name: "Backup",
            apiKey: "sk-backup",
            baseUrl: "https://backup.example.com",
            model: "",
            profileType: "codex",
          },
        ],
        activeCodexProfileId: "backup",
        activeClaudeProfileId: null,
      }),
    );
    await flushUi(wrapper);

    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("同步");
  });
});

async function flushUi(wrapper: ReturnType<typeof mount>) {
  await Promise.resolve();
  await nextTick();
  await wrapper.vm.$nextTick();
}
