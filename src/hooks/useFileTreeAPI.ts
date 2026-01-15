import { useCallback } from "react";
import { useFileTreeStore, FileTreeNode } from "@/stores/fileTreeStore";
import { logger } from "@/utils/logger";
import { workspaceAPI } from "@/api";

// Module-level deduplication for loadFileTree calls
let pendingLoadFileTree: Promise<void> | null = null;

interface FolderStructure {
  name: string;
  path: string;
  relativePath: string;
  type: "file" | "folder" | "directory" | "dir";
  children?: FolderStructure[] | null;
}

const normalizePath = (path: string) =>
  path
    .replace(/\\/g, "/")
    .replace(/^\.\//, "")
    .replace(/^\/+/, "")
    .replace(/\/+$/, "");

function deriveRelativePath(rawPath: string, workspacePath?: string): string {
  const normalizedWorkspace = workspacePath?.replace(/\\/g, "/") ?? "";
  const normalizedRaw = rawPath.replace(/\\/g, "/");

  if (normalizedWorkspace && normalizedRaw.startsWith(normalizedWorkspace)) {
    const trimmed = normalizedRaw
      .slice(normalizedWorkspace.length)
      .replace(/^\/+/, "");
    return normalizePath(trimmed || rawPath);
  }

  return normalizePath(rawPath);
}

function mapStructure(
  node: FolderStructure,
  workspacePath?: string,
): FileTreeNode | null {
  const nodeType =
    node.type === "directory" || node.type === "dir" ? "folder" : node.type;
  const basePath = node.relativePath || node.path || node.name;
  const normalizedPath = deriveRelativePath(basePath, workspacePath);

  if (nodeType === "folder") {
    const children =
      node.children
        ?.map((child) => mapStructure(child, workspacePath))
        .filter((child): child is FileTreeNode => Boolean(child)) ?? [];

    return {
      name: node.name,
      path: normalizedPath,
      type: "folder",
      children,
    };
  }

  if (nodeType === "file") {
    return {
      name: node.name,
      path: normalizedPath,
      type: "file",
    };
  }

  logger.warn("[useFileTreeAPI] Unknown node type in structure", { node });
  return null;
}

function toFileTree(
  nodes: FolderStructure[],
  workspacePath?: string,
): FileTreeNode[] {
  return (
    nodes
      ?.map((node) => mapStructure(node, workspacePath))
      .filter((node): node is FileTreeNode => Boolean(node)) ?? []
  );
}

export function useFileTreeAPI() {
  const {
    setTree,
    setActiveFolder,
    setSelectedFile,
    setLoading,
    setError,
    setCounts,
    expandAll,
    collapseAll,
  } = useFileTreeStore();

  const loadFileTree = useCallback(async () => {
    // Deduplicate: if same request is already pending, reuse it
    // if (pendingLoadFileTree) {
    //   logger.info(
    //     "[useFileTreeAPI.loadFileTree] Deduplicating request, reusing pending call",
    //   );
    //   return pendingLoadFileTree;
    // }

    setLoading(true);
    setError(null);

    const doLoad = async () => {
      try {
        const activeWorkspace = await workspaceAPI.getActive();

        const workspaceId = activeWorkspace.success
          ? activeWorkspace.data?.workspace?.id
          : undefined;

        if (!workspaceId) {
          logger.warn(
            "[useFileTreeAPI.loadFileTree] No active workspace id returned",
          );
          setError("No active workspace selected");
          setTree([]);
          return;
        }

        const { activeFolder: prevActive, selectedFile: prevSelected } =
          useFileTreeStore.getState();

        logger.info("[useFileTreeAPI.loadFileTree] Invoking scan", {
          workspaceId,
        });
        const response = await workspaceAPI.scan(workspaceId);
        logger.info("[useFileTreeAPI.loadFileTree] Scan response:", {
          success: response.success,
          fileCount: response.data?.files?.length,
          structureCount: response.data?.structure?.length,
          error: response.error,
        });

        if (response.success && response.data) {
          const rawSample = (response.data.structure || [])
            .slice(0, 8)
            .map((n) => ({
              name: n.name,
              type: n.type,
              path: n.path,
              relativePath: n.relativePath,
              hasChildren: Array.isArray(n.children) ? n.children.length : 0,
            }));

          const workspacePath = activeWorkspace.data?.workspace?.path;
          const tree = toFileTree(response.data.structure || [], workspacePath);
          const treeSample = tree.slice(0, 8).map((n) => ({
            name: n.name,
            type: n.type,
            path: n.path,
            children: n.children?.length || 0,
          }));

          logger.info("[useFileTreeAPI.loadFileTree] Structure samples", {
            rawSample,
            treeSample,
          });
          logger.info("[useFileTreeAPI.loadFileTree] Mapped tree:", {
            nodeCount: tree.length,
          });

          if (tree.length === 0) {
            logger.warn(
              "[useFileTreeAPI.loadFileTree] Tree is empty after mapping",
            );
          }

          setTree(tree);
          if (response.data.counts) {
            setCounts(response.data.counts);
          }
          if (prevActive) {
            setActiveFolder(prevActive);
            if (prevSelected) {
              setSelectedFile(prevSelected);
            }
          } else {
            setActiveFolder(null);
          }
          logger.info("[useFileTreeAPI.loadFileTree] Tree loaded");
        } else {
          logger.error(
            "[useFileTreeAPI.loadFileTree] Scan failed",
            response.error,
          );
          setError(
            response.error?.message || "Failed to load workspace structure",
          );
        }
      } catch (error) {
        logger.error('[useFileTreeAPI.loadFileTree] Exception during load', error);
        setError(
          error instanceof Error
            ? error.message
            : "Failed to load workspace structure",
        );
      } finally {
        setLoading(false);
        pendingLoadFileTree = null;
      }
    };

    pendingLoadFileTree = doLoad();
    return pendingLoadFileTree;
  }, [setLoading, setError, setTree, setActiveFolder]);

  const createFolder = useCallback(
    async ({
      parentPath,
      name,
    }: { parentPath?: string | null; name?: string } = {}) => {
      try {
        const response = await workspaceAPI.createFolder(
          name ?? "New Folder",
          parentPath ?? undefined,
        );

        if (response.success && response.data) {
          return response.data.folderPath;
        }

        setError(response.error?.message || "Failed to create folder");
        return null;
      } catch (error) {
        setError(
          error instanceof Error ? error.message : "Failed to create folder",
        );
        return null;
      }
    },
    [setError],
  );

  const renameFolder = useCallback(
    async (path: string, name: string) => {
      try {
        const response = await workspaceAPI.renameFolder(path, name);

        if (response.success && response.data) {
          return response.data.folderPath;
        }

        setError(response.error?.message || "Failed to rename folder");
        return null;
      } catch (error) {
        setError(
          error instanceof Error ? error.message : "Failed to rename folder",
        );
        return null;
      }
    },
    [setError],
  );

  const deleteFolder = useCallback(
    async (path: string) => {
      try {
        const response = await workspaceAPI.deleteFolder(path);

        if (response.success && response.data) {
          return response.data.success;
        }

        setError(response.error?.message || "Failed to delete folder");
        return false;
      } catch (error) {
        setError(
          error instanceof Error ? error.message : "Failed to delete folder",
        );
        return false;
      }
    },
    [setError],
  );

  const moveFolder = useCallback(
    async (sourcePath: string, destinationPath: string | null) => {
      try {
        const response = await workspaceAPI.moveFolder(
          sourcePath,
          destinationPath,
        );

        if (response.success && response.data) {
          return response.data.folderPath;
        }

        setError(response.error?.message || "Failed to move folder");
        return null;
      } catch (error) {
        setError(
          error instanceof Error ? error.message : "Failed to move folder",
        );
        return null;
      }
    },
    [setError],
  );

  return {
    loadFileTree,
    expandAll,
    collapseAll,
    createFolder,
    renameFolder,
    deleteFolder,
    moveFolder,
  };
}
