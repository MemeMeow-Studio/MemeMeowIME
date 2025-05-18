<script setup lang="ts">
import { ref, onMounted, reactive, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface ShortcutConfig {
  modifiers: string[];
  key: string;
  action: string;
}

interface UserPreferences {
  copyToClipboard: boolean;
  shortcuts: {
    toggleApp: ShortcutConfig;
    // 可以添加更多快捷键配置
  };
}

const preferences = reactive<UserPreferences>({
  copyToClipboard: true,
  shortcuts: {
    toggleApp: {
      modifiers: ['ctrl', 'alt'],
      key: 'v',
      action: '切换应用窗口'
    }
  }
});

const isRecordingShortcut = ref(false);
const currentShortcutTarget = ref('');
const shortcutText = ref('');
const errorMessage = ref('');
const successMessage = ref('');
const isLoading = ref(false);

// 按键映射表
const keyDisplayMap: Record<string, string> = {
  'Control': 'Ctrl',
  'Alt': 'Alt',
  'Shift': 'Shift',
  'Meta': '⌘',
  // 更多键映射...
};

onMounted(async () => {
  try {
    isLoading.value = true;
    // 从后端加载用户偏好设置
    const userPrefs = await invoke('get_user_preferences');
    // 合并已存在的设置
    if (userPrefs) {
      // 将后端的蛇形命名转换为前端的驼峰命名
      preferences.copyToClipboard = userPrefs.copy_to_clipboard;
      
      // 获取当前快捷键配置
      const shortcuts = await invoke('get_shortcuts');
      if (shortcuts) {
        preferences.shortcuts.toggleApp = shortcuts.toggle_app;
      }
    }
  } catch (error) {
    console.error('加载设置失败:', error);
    errorMessage.value = `加载设置失败: ${error}`;
  } finally {
    isLoading.value = false;
  }
  
  // 监听快捷键注册失败事件
  listen('shortcut-registration-failed', (event) => {
    errorMessage.value = `${event.payload}`;
    setTimeout(() => { errorMessage.value = ''; }, 7000);
  });
});

// 改进的invokeWithTimeout函数，确保始终会释放loading状态
const invokeWithTimeout = async (command: string, args: any = {}, timeout = 5000) => {
  console.log(`调用命令: ${command}，参数:`, args);
  let timeoutId: any;
  
  try {
    const result = await Promise.race([
      invoke(command, args),
      new Promise((_, reject) => {
        timeoutId = setTimeout(() => {
          reject(new Error(`命令 ${command} 执行超时`));
        }, timeout);
      })
    ]);
    
    clearTimeout(timeoutId);
    return result;
  } catch (error) {
    if (timeoutId) clearTimeout(timeoutId);
    console.error(`执行命令 ${command} 失败:`, error);
    throw error;
  }
};

// 添加一个防抖机制，避免连续多次保存
let saveDebounceTimer: any = null;
const savePreferences = async () => {
  // 防止重复点击
  if (isLoading.value) return;
  
  // 清除之前的定时器
  if (saveDebounceTimer) clearTimeout(saveDebounceTimer);
  
  console.log('开始保存设置...');
  saveDebounceTimer = setTimeout(async () => {
    try {
      isLoading.value = true;
      errorMessage.value = '';
      successMessage.value = '';
      
      console.log('设置保存中:', preferences);
      
      // 使用标志来跟踪各个设置的保存状态
      let clipboardSaved = false;
      let shortcutsSaved = false;
      let shortcutsRefreshed = false;
      let errors = [];
      
      // 保存复制到剪贴板设置
      try {
        console.log('正在保存剪贴板设置...');
        await invokeWithTimeout('set_copy_to_clipboard', { enabled: preferences.copyToClipboard }, 3000);
        console.log('剪贴板设置已保存');
        clipboardSaved = true;
      } catch (clipboardError) {
        console.error('保存剪贴板设置失败:', clipboardError);
        errors.push(`剪贴板设置保存失败: ${clipboardError}`);
        // 不再抛出错误，继续执行下一步
      }
      
      // 保存快捷键设置
      try {
        console.log('正在保存快捷键设置...');
        await invokeWithTimeout('set_shortcuts', {
          shortcuts: {
            toggle_app: preferences.shortcuts.toggleApp
          }
        }, 3000);
        console.log('快捷键设置已保存');
        shortcutsSaved = true;
      } catch (shortcutError) {
        console.error('保存快捷键设置失败:', shortcutError);
        errors.push(`快捷键设置保存失败: ${shortcutError}`);
        // 不再抛出错误，继续执行下一步
      }
      
      // 只有在快捷键设置成功保存的情况下才尝试刷新快捷键
      if (shortcutsSaved) {
        try {
          console.log('正在刷新快捷键...');
          await invokeWithTimeout('refresh_shortcuts', {}, 3000);
          console.log('快捷键已刷新');
          shortcutsRefreshed = true;
        } catch (refreshError) {
          console.warn('快捷键注册失败:', refreshError);
          errors.push(`快捷键注册失败: ${refreshError}`);
        }
      }
      
      // 根据各个设置的保存情况设置反馈信息
      if (clipboardSaved && shortcutsSaved && shortcutsRefreshed) {
        successMessage.value = '所有设置已保存并应用';
      } else if ((clipboardSaved || shortcutsSaved) && errors.length > 0) {
        successMessage.value = '部分设置已保存';
        errorMessage.value = errors.join('\n');
      } else {
        errorMessage.value = '保存设置失败: ' + errors.join('\n');
      }
      
    } catch (error) {
      console.error('保存设置失败:', error);
      errorMessage.value = `保存设置失败: ${error}`;
    } finally {
      isLoading.value = false;
      console.log('保存设置流程结束');
      
      // 延时清除成功消息
      if (successMessage.value) {
        setTimeout(() => { 
          successMessage.value = ''; 
        }, 3000);
      }
    }
  }, 200); // 200ms防抖延迟
};

const startRecordingShortcut = (target: string) => {
  isRecordingShortcut.value = true;
  currentShortcutTarget.value = target;
  shortcutText.value = '请按下快捷键组合...';
  
  // 清除错误和成功信息
  errorMessage.value = '';
  successMessage.value = '';
};

const handleKeyDown = (event: KeyboardEvent) => {
  if (!isRecordingShortcut.value) return;
  
  event.preventDefault();
  
  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push('ctrl');
  if (event.altKey) modifiers.push('alt');
  if (event.shiftKey) modifiers.push('shift');
  if (event.metaKey) modifiers.push('meta');
  
  const key = event.key.toLowerCase();
  
  // 忽略单独的修饰键
  if (key === 'control' || key === 'alt' || key === 'shift' || key === 'meta') {
    return;
  }
  
  // 至少需要一个修饰键
  if (modifiers.length === 0) {
    errorMessage.value = '快捷键必须包含至少一个修饰键 (Ctrl, Alt, Shift, Meta)';
    setTimeout(() => { errorMessage.value = ''; }, 3000);
    isRecordingShortcut.value = false;
    shortcutText.value = '';
    return;
  }
  
  // 更新快捷键显示文本
  shortcutText.value = [
    ...modifiers.map(mod => keyDisplayMap[mod] || mod.toUpperCase()),
    key.toUpperCase()
  ].join(' + ');
  
  // 更新快捷键配置
  if (currentShortcutTarget.value === 'toggleApp') {
    preferences.shortcuts.toggleApp.modifiers = modifiers;
    preferences.shortcuts.toggleApp.key = key;
  }
  
  // 停止记录
  isRecordingShortcut.value = false;
};

const resetShortcut = (target: string) => {
  errorMessage.value = '';
  successMessage.value = '';
  
  if (target === 'toggleApp') {
    preferences.shortcuts.toggleApp = {
      modifiers: ['ctrl', 'alt'],
      key: 'v',
      action: '切换应用窗口'
    };
    shortcutText.value = '';
  }
};

// 格式化快捷键显示
const formatShortcut = (shortcut: ShortcutConfig): string => {
  return [
    ...shortcut.modifiers.map(mod => keyDisplayMap[mod] || mod.toUpperCase()),
    shortcut.key.toUpperCase()
  ].join(' + ');
};

// 确保在快捷键录制状态下有合理的超时机制
watch(isRecordingShortcut, (newValue) => {
  if (newValue) {
    // 如果开始录制快捷键，设置10秒后自动取消录制状态
    setTimeout(() => {
      if (isRecordingShortcut.value) {
        isRecordingShortcut.value = false;
        shortcutText.value = '';
        errorMessage.value = '快捷键录制超时，请重试';
        setTimeout(() => { errorMessage.value = ''; }, 3000);
      }
    }, 10000);
  }
});
</script>

<template>
  <div class="settings-container">
    <h2>设置</h2>
    
    <div v-if="isLoading" class="loading-overlay">
      <div class="loading-spinner"></div>
      <p>加载中...</p>
    </div>
    
    <div v-if="errorMessage" class="error-message">
      <p>{{ errorMessage }}</p>
      <button class="close-button" @click="errorMessage = ''">关闭</button>
    </div>
    
    <div v-if="successMessage" class="success-message">
      <p>{{ successMessage }}</p>
      <button class="close-button" @click="successMessage = ''">关闭</button>
    </div>
    
    <div class="settings-section">
      <h3>基本设置</h3>
      <div class="setting-item">
        <label>
          <input type="checkbox" v-model="preferences.copyToClipboard" />
          选择表情后自动复制到剪贴板
        </label>
      </div>
    </div>
    
    <div class="settings-section">
      <h3>快捷键设置</h3>
      <p class="shortcut-info">配置全局快捷键以快速访问应用。如果快捷键与系统或其他应用的快捷键冲突，将无法注册。</p>
      <div class="setting-item" @keydown="handleKeyDown">
        <div class="shortcut-setting">
          <span class="shortcut-label">切换应用窗口:</span>
          <div class="shortcut-container">
            <button 
              class="shortcut-button" 
              @click="startRecordingShortcut('toggleApp')"
              :class="{ 'recording': isRecordingShortcut && currentShortcutTarget === 'toggleApp' }"
            >
              {{ isRecordingShortcut && currentShortcutTarget === 'toggleApp'
                ? shortcutText
                : formatShortcut(preferences.shortcuts.toggleApp) }}
            </button>
            <button class="reset-button" @click="resetShortcut('toggleApp')">重置</button>
          </div>
        </div>
        <p class="shortcut-hint">点击按钮录制快捷键。必须包含至少一个修饰键 (Ctrl, Alt, Shift)。如果快捷键注册失败，请尝试其他组合。</p>
      </div>
    </div>
    
    <div class="button-row">
      <button class="save-button" @click="savePreferences" :disabled="isLoading">保存设置</button>
    </div>
  </div>
</template>

<style scoped>
.settings-container {
  padding: 1rem;
  text-align: left;
  position: relative;
}

.settings-section {
  margin-bottom: 1.5rem;
  padding: 1rem;
  background-color: #f5f5f5;
  border-radius: 8px;
}

.setting-item {
  margin-bottom: 1rem;
}

.shortcut-info {
  font-size: 0.9rem;
  color: #666;
  margin-bottom: 1rem;
}

.shortcut-setting {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.shortcut-label {
  min-width: 150px;
}

.shortcut-container {
  display: flex;
  gap: 0.5rem;
}

.shortcut-button {
  padding: 0.5rem 1rem;
  background-color: #e0e0e0;
  border: 1px solid #ccc;
  border-radius: 4px;
  min-width: 200px;
  text-align: center;
}

.shortcut-button.recording {
  background-color: #ffecb3;
  border-color: #ffc107;
}

.shortcut-hint {
  font-size: 0.8rem;
  color: #666;
  margin-top: 0.5rem;
}

.reset-button {
  padding: 0.5rem;
  background-color: #f0f0f0;
  border: 1px solid #ccc;
  border-radius: 4px;
}

.reset-button:hover {
  background-color: #e0e0e0;
}

.button-row {
  display: flex;
  justify-content: flex-end;
  margin-top: 1rem;
}

.save-button {
  padding: 0.5rem 1.5rem;
  background-color: #4caf50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.save-button:hover {
  background-color: #388e3c;
}

.save-button:disabled {
  background-color: #a5d6a7;
  cursor: not-allowed;
}

.error-message, .success-message {
  position: relative;
  background-color: #ffebee;
  border-left: 3px solid #f44336;
  color: #b71c1c;
  padding: 0.75rem 1rem;
  margin-bottom: 1rem;
  border-radius: 4px;
}

.success-message {
  background-color: #e8f5e9;
  border-left: 3px solid #4caf50;
  color: #1b5e20;
}

.close-button {
  position: absolute;
  top: 5px;
  right: 5px;
  background: transparent;
  border: none;
  font-size: 0.8rem;
  cursor: pointer;
  color: inherit;
  opacity: 0.7;
}

.close-button:hover {
  opacity: 1;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.8);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  z-index: 10;
  cursor: wait;
}

.loading-spinner {
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  width: 30px;
  height: 30px;
  animation: spin 2s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
