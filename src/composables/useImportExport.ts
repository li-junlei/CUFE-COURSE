// 导入导出逻辑 Composable
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import type { ScheduleMetadata } from '../types';

/**
 * 导出课表到文件
 */
export async function exportSchedule(schedule: ScheduleMetadata): Promise<boolean> {
  try {
    const filePath = await saveDialog({
      defaultPath: `${schedule.name}.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!filePath) return false;

    await invoke('export_schedule', { scheduleId: schedule.id, filePath });
    return true;
  } catch (error) {
    console.error('导出课表失败:', error);
    return false;
  }
}

/**
 * 从文件导入课表
 */
export async function importScheduleFromFile(): Promise<string | null> {
  try {
    const filePath = await openDialog({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });
    if (!filePath) return null;

    const scheduleId = await invoke<string>('import_schedule', { filePath });
    return scheduleId;
  } catch (error) {
    console.error('导入课表失败:', error);
    return null;
  }
}

/**
 * 从教务系统导入课表（带自动重登录）
 */
export async function importScheduleWithAutoRelogin(
  year: number,
  term: number,
  scheduleName: string
): Promise<string> {
  return await invoke<string>('import_schedule_with_auto_relogin', {
    year,
    term,
    scheduleName,
  });
}

/**
 * 更新课表（在线更新）
 */
export async function updateSchedule(scheduleId: string) {
  return await invoke<{ added_count: number; removed_count: number; modified_count: number; unchanged_count: number }>('update_schedule_with_diff', {
    scheduleId,
  });
}

/**
 * 刷新课表数据
 */
export async function refreshSchedule(): Promise<any[]> {
  return await invoke('refresh_schedule');
}

/**
 * 导入考试信息
 */
export async function importExams(scheduleId: string): Promise<any[]> {
  return await invoke('fetch_and_import_exams', { scheduleId });
}

/**
 * 上传背景图
 */
export async function uploadBackground(): Promise<string | null> {
  try {
    const filePath = await openDialog({
      multiple: false,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp'] }],
    });
    if (!filePath) return null;

    return await invoke<string>('save_background_image', { sourcePath: filePath });
  } catch (error) {
    console.error('上传背景图失败:', error);
    return null;
  }
}

/**
 * 删除背景图
 */
export async function deleteBackground(): Promise<void> {
  await invoke('delete_background_image');
}

export function useImportExport() {
  return {
    exportSchedule,
    importScheduleFromFile,
    importScheduleWithAutoRelogin,
    updateSchedule,
    refreshSchedule,
    importExams,
    uploadBackground,
    deleteBackground,
  };
}
