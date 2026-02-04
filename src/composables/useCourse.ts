import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Course, ScheduleMetadata } from '../types';

/**
 * 课表数据管理 Composable
 */
export function useCourse() {
  const courses = ref<Course[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  /**
   * 登录并获取课表
   */
  async function loginAndGetSchedule(
    username: string,
    password: string,
    baseUrl: string,
    _scheduleName: string = '教务系统导入'
  ): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Course[]>('login_and_get_schedule', {
        username,
        password,
        baseUrl,
      });
      courses.value = result;
      // 后端已自动获取并保存所有学期课表，不需要前端再次保存
      // await saveScheduleCache(result, scheduleName);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 刷新课表
   */
  async function refreshSchedule(baseUrl: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Course[]>('refresh_schedule', { baseUrl });
      courses.value = result;
      await saveScheduleCache(result, '未命名课表');
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 加载缓存的课表
   */
  async function loadCachedSchedule(scheduleId?: string): Promise<boolean> {
    try {
      const result = await invoke<Course[]>('load_cached_schedule', {
        scheduleId
      });
      courses.value = result;
      return true;
    } catch (e) {
      // 缓存不存在或已过期
      return false;
    }
  }

  /**
   * 保存课表到缓存
   */
  async function saveScheduleCache(
    coursesData: Course[],
    name: string,
    scheduleId?: string,
    firstDay?: number,
    maxPeriods?: number,
    weeksCount?: number,
    timeTableId?: string
  ): Promise<string> {
    try {
      const id = await invoke<string>('save_schedule_cache', {
        courses: coursesData,
        name,
        scheduleId,
        firstDay,
        maxPeriods,
        weeksCount,
        timeTableId
      });
      return id;
    } catch (e) {
      console.error('保存缓存失败:', e);
      throw e;
    }
  }

  /**
   * 更新课表信息
   */
  async function updateScheduleInfo(
    scheduleId: string,
    firstDay?: number,
    maxPeriods?: number,
    weeksCount?: number,
    timeTableId?: string
  ): Promise<void> {
    try {
      await invoke('update_schedule_info', {
        scheduleId,
        firstDay,
        maxPeriods,
        weeksCount,
        timeTableId
      });
    } catch (e) {
      console.error('更新课表信息失败:', e);
      throw e;
    }
  }

  /**
   * 将当前课表设置应用到全部
   */
  async function applySettingsToAll(sourceScheduleId: string): Promise<void> {
    try {
      await invoke('apply_settings_to_all', { sourceScheduleId });
    } catch (e) {
      console.error('应用设置到全部失败:', e);
      throw e;
    }
  }

  /**
   * 重命名课表
   */
  async function renameSchedule(scheduleId: string, newName: string): Promise<void> {
    try {
      await invoke('rename_schedule', { scheduleId, newName });
    } catch (e) {
      console.error('重命名课表失败:', e);
      throw e;
    }
  }

  /**
   * 重新排序课表
   */
  async function reorderSchedules(sortedIds: string[]): Promise<void> {
    try {
      await invoke('reorder_schedules', { sortedIds });
    } catch (e) {
      console.error('重新排序失败:', e);
      throw e;
    }
  }

  /**
   * 解析 HTML 获取课表
   */
  async function parseHtmlSchedule(html: string, parserType?: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Course[]>('parse_html_schedule', {
        html,
        parserType: parserType || 'cufe_default'
      });
      courses.value = result;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取所有课表列表
   */
  async function listSchedules(): Promise<ScheduleMetadata[]> {
    try {
      return await invoke<ScheduleMetadata[]>('list_schedules');
    } catch (e) {
      console.error('获取课表列表失败:', e);
      throw e;
    }
  }

  /**
   * 删除指定课表
   */
  async function deleteSchedule(scheduleId: string): Promise<void> {
    try {
      await invoke('delete_schedule', { scheduleId });
    } catch (e) {
      console.error('删除课表失败:', e);
      throw e;
    }
  }

  /**
   * 切换当前课表
   */
  async function switchSchedule(scheduleId: string): Promise<void> {
    try {
      await invoke('switch_schedule', { scheduleId });
    } catch (e) {
      console.error('切换课表失败:', e);
      throw e;
    }
  }

  /**
   * 计算最大周次
   */
  const maxWeek = computed(() => {
    let max = 0;
    for (const course of courses.value) {
      if (course.weeks && course.weeks.length > 0) {
        // rust端的weeks已经是展开的周次列表了，直接取最大值
        const courseMax = Math.max(...course.weeks);
        max = Math.max(max, courseMax);
      }
    }
    return max || 20;
  });

  return {
    courses,
    loading,
    error,
    loginAndGetSchedule,
    parseHtmlSchedule,
    refreshSchedule,
    loadCachedSchedule,
    saveScheduleCache,
    listSchedules,
    deleteSchedule,
    switchSchedule,
    updateScheduleInfo,
    applySettingsToAll,
    renameSchedule,
    reorderSchedules,
    maxWeek,
  };
}

/**
 * 监听浏览器导入事件
 */
export function useBrowserImport() {
  const courses = ref<Course[]>([]);
  const isImporting = ref(false);

  // 监听来自浏览器的导入事件
  async function setupImportListener(callback: (result: { schedule_id: string; course_count: number; schedule_name: string }) => void) {
    try {
      const unlisten = await listen<{ schedule_id: string; course_count: number; schedule_name: string }>('schedule-imported', (event) => {
        courses.value = []; // 清空之前的课程数据
        callback(event.payload);
      });
      return unlisten;
    } catch (e) {
      console.error('设置导入监听器失败:', e);
      throw e;
    }
  }

  return {
    courses,
    isImporting,
    setupImportListener,
  };
}
