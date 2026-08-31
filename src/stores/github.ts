import { defineStore } from "pinia";
import { ref } from "vue";
import { tauriApi } from "../services/tauri";
import type {
  GithubAccount,
  GithubDeviceCode,
  GithubLoginResult,
  GithubRepo,
} from "../types";

export const useGithubStore = defineStore("github", () => {
  const account = ref<GithubAccount | null>(null);
  const repos = ref<GithubRepo[]>([]);
  const loadingRepos = ref(false);

  /** 启动时恢复登录态（令牌在本地 SQLite，由后端读出） */
  async function fetchStatus() {
    account.value = await tauriApi.githubStatus();
  }

  /** 拉取仓库列表（未登录时跳过） */
  async function fetchRepos() {
    if (!account.value) return;
    loadingRepos.value = true;
    try {
      repos.value = await tauriApi.githubListRepos();
    } finally {
      loadingRepos.value = false;
    }
  }

  function startLogin(clientId: string): Promise<GithubDeviceCode> {
    return tauriApi.githubLoginStart(clientId);
  }

  function pollLogin(clientId: string, deviceCode: string): Promise<GithubLoginResult> {
    return tauriApi.githubLoginPoll(clientId, deviceCode);
  }

  function setAccount(acc: GithubAccount) {
    account.value = acc;
  }

  async function logout() {
    await tauriApi.githubLogout();
    account.value = null;
    repos.value = [];
  }

  return {
    account,
    repos,
    loadingRepos,
    fetchStatus,
    fetchRepos,
    startLogin,
    pollLogin,
    setAccount,
    logout,
  };
});
