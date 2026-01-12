/**
 * Settings API - IPC channel wrappers for settings and database operations
 *
 * Implements: specs/api.ts#SettingsAPI, specs/api.ts#DatabaseAPI, specs/api.ts#SystemAPI
 * Pure functions that wrap IPC channels. No React, no stores.
 */

import { invokeIpc } from '../lib/tauri-ipc';
import {
  SETTINGS_COMMANDS,
  DATABASE_COMMANDS,
  SYSTEM_COMMANDS,
} from '../constants/tauriCommands';
import type {
  Settings,
  DatabaseStatus,
  BackupResult,
  VacuumResult,
  IntegrityResult,
  Migration,
  IpcResponse,
} from '../types';

export const settingsAPI = {
  /**
   * Get a setting by key
   */
  get: <T = string>(key: string): Promise<IpcResponse<{ value: T | null }>> =>
    invokeIpc(SETTINGS_COMMANDS.GET, { key }),

  /**
   * Set a setting
   */
  set: <T = string>(key: string, value: T): Promise<IpcResponse<void>> =>
    invokeIpc(SETTINGS_COMMANDS.SET, { key, value }),

  /**
   * Get all settings
   */
  getAll: (): Promise<IpcResponse<{ settings: Settings[] }>> =>
    invokeIpc(SETTINGS_COMMANDS.GET_ALL, {}),
};

export const databaseAPI = {
  /**
   * Get database status
   */
  getStatus: (): Promise<IpcResponse<DatabaseStatus>> =>
    invokeIpc(DATABASE_COMMANDS.GET_STATUS, {}),

  /**
   * Run pending migrations
   */
  runMigrations: (): Promise<IpcResponse<{ applied: number }>> =>
    invokeIpc(DATABASE_COMMANDS.RUN_MIGRATIONS, {}),

  /**
   * Create a backup
   */
  backup: (path?: string): Promise<IpcResponse<BackupResult>> =>
    invokeIpc(DATABASE_COMMANDS.BACKUP, { path }),

  /**
   * Restore from backup
   */
  restore: (path: string): Promise<IpcResponse<void>> =>
    invokeIpc(DATABASE_COMMANDS.RESTORE, { path }),

  /**
   * Export database
   */
  export: (format: 'json' | 'sqlite'): Promise<IpcResponse<{ path: string }>> =>
    invokeIpc(DATABASE_COMMANDS.EXPORT, { format }),

  /**
   * Import database
   */
  import: (path: string): Promise<IpcResponse<void>> =>
    invokeIpc(DATABASE_COMMANDS.IMPORT, { path }),

  /**
   * Vacuum database (optimize)
   */
  vacuum: (): Promise<IpcResponse<VacuumResult>> => invokeIpc(DATABASE_COMMANDS.VACUUM, {}),

  /**
   * Check database integrity
   */
  checkIntegrity: (): Promise<IpcResponse<IntegrityResult>> =>
    invokeIpc(DATABASE_COMMANDS.CHECK_INTEGRITY, {}),

  /**
   * Get migration history
   */
  getMigrationHistory: (): Promise<IpcResponse<{ migrations: Migration[] }>> =>
    invokeIpc(DATABASE_COMMANDS.GET_MIGRATION_HISTORY, {}),
};

export const systemAPI = {
  /**
   * Get available system fonts
   */
  getFonts: (): Promise<IpcResponse<{ fonts: string[] }>> =>
    invokeIpc(SYSTEM_COMMANDS.GET_FONTS, {}),
};
