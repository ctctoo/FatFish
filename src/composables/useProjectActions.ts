import { openUrl } from "@tauri-apps/plugin-opener";
import { useProjectStore } from "../stores/project";
import { useUiStore } from "../stores/ui";
import { useI18n } from "../i18n";
import type { Project, ProjectStatus } from "../types";

interface ProjectActionsOptions {
  /** 编辑项目（打开编辑对话框） */
  onEdit?: (project: Project) => void;
  /** 添加链接（打开链接对话框） */
  onAddLink?: (project: Project) => void;
  /** 删除项目（弹确认框或直接删除） */
  onDelete?: (project: Project) => void;
  /** 状态变更成功后回调（用于同步本地列表项） */
  onStatusChanged?: (project: Project) => void;
}

/**
 * 卡片/菜单动作的统一分发。
 * 项目列表页 / 首页 / 集合页共用，避免每处重复写 openFolder / openTerminal /
 * copyPath / refreshGit / status 切换 + toast 的逻辑。
 */
export function useProjectActions(options: ProjectActionsOptions = {}) {
  const projectStore = useProjectStore();
  const uiStore = useUiStore();
  const { t } = useI18n();

  function handleAction(action: string, project: Project) {
    const run = async () => {
      if (action.startsWith("status:")) {
        const status = action.slice("status:".length) as ProjectStatus;
        const updated = await projectStore.changeStatus(project, status);
        uiStore.showToast(t("toast.statusChanged"), "success");
        options.onStatusChanged?.(updated);
        return;
      }
      switch (action) {
        case "open":
        case "open-folder":
          await projectStore.openInFolder(project);
          break;
        case "open-terminal":
          await projectStore.openTerminal(project);
          break;
        case "open-github": {
          const gh = project.links.find((l) => l.linkType === "github");
          if (gh) await openUrl(gh.url);
          break;
        }
        case "copy-path":
          await navigator.clipboard.writeText(project.path);
          uiStore.showToast(t("toast.copied"), "success");
          break;
        case "edit":
          options.onEdit?.(project);
          break;
        case "add-link":
          options.onAddLink?.(project);
          break;
        case "refresh-git":
          await projectStore.refreshGit(project.id);
          uiStore.showToast(t("toast.gitRefreshed"), "success");
          break;
        case "delete":
          await options.onDelete?.(project);
          break;
      }
    };
    run().catch((e) => uiStore.showToast(String(e), "error"));
  }

  return { handleAction };
}
