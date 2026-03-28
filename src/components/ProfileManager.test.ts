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
      profileType: "codex",
    },
  ];

  return {
    profiles,
    activeProfileId: "primary",
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

  it("renders saved profiles after loading", async () => {
    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    expect(wrapper.text()).toContain("Primary");
  });

  it("renders a compact workspace layout", async () => {
    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    expect(wrapper.find('[data-testid="workspace-shell"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="profile-sidebar"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="profile-editor"]').exists()).toBe(true);
  });

  it("shows validation when required fields are missing", async () => {
    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    await wrapper.get('[data-testid="new-profile"]').trigger("click");
    await wrapper.get("form").trigger("submit.prevent");
    await nextTick();

    expect(mockedSaveProfiles).not.toHaveBeenCalled();
    expect(wrapper.get('[data-testid="status-message"]').text()).toContain(
      "请先填写名称、API Key 和 Base URL",
    );
  });

  it("tells the user save only updates local data", async () => {
    mockedSaveProfiles.mockResolvedValue(
      createState({
        profiles: [
          {
            id: "primary",
            name: "Primary",
            apiKey: "sk-primary",
            baseUrl: "https://primary.example.com",
            profileType: "codex",
          },
          {
            id: "local-only",
            name: "Local Only",
            apiKey: "sk-local",
            baseUrl: "https://local.example.com",
            profileType: "codex",
          },
        ],
      }),
    );

    const wrapper = mount(ProfileManager);
    await flushUi(wrapper);

    expect(wrapper.get('[data-testid="local-save-note"]').text()).toContain("只会写入本地列表");

    await wrapper.get('[data-testid="new-profile"]').trigger("click");
    await wrapper.get('input[placeholder="例如：主账号"]').setValue("Local Only");
    await wrapper
      .get('input[placeholder="https://api.example.com"]')
      .setValue("https://local.example.com");
    await wrapper.get('[data-testid="api-key-input"]').setValue("sk-local");
    await wrapper.get("form").trigger("submit.prevent");
    await flushUi(wrapper);

    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("已保存到本地列表");
    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("切换账号后");
  });

  it("disables switch buttons while switching profile", async () => {
    let resolveSwitch: ((value: ManagerState) => void) | undefined;
    mockedLoadManagerState.mockResolvedValue(
      createState({
        profiles: [
          {
            id: "primary",
            name: "Primary",
            apiKey: "sk-primary",
            baseUrl: "https://primary.example.com",
            profileType: "codex",
          },
          {
            id: "backup",
            name: "Backup",
            apiKey: "sk-backup",
            baseUrl: "https://backup.example.com",
            profileType: "codex",
          },
        ],
        activeProfileId: "primary",
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

    const switchButton = wrapper.get('[data-testid="switch-backup"]');
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
            profileType: "codex",
          },
          {
            id: "backup",
            name: "Backup",
            apiKey: "sk-backup",
            baseUrl: "https://backup.example.com",
            profileType: "codex",
          },
        ],
        activeProfileId: "backup",
      }),
    );
    await flushUi(wrapper);

    expect(wrapper.get('[data-testid="status-message"]').text()).toContain("Codex 配置已同步");
  });
});

async function flushUi(wrapper: ReturnType<typeof mount>) {
  await Promise.resolve();
  await nextTick();
  await wrapper.vm.$nextTick();
}
