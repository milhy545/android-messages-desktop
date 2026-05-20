import settings from 'electron-settings';
import { IS_LINUX, SETTING_TRAY_ENABLED, SETTING_TRAY_CLICK_SHORTCUT, SETTING_CUSTOM_WORDS } from '../constants';

const defaults = {
  autoHideMenuPref: false,
  startInTrayPref: false,
  notificationSoundEnabledPref: true,
  pressEnterToSendPref: true,
  hideNotificationContentPref: false,
  useSystemDarkModePref: true,
  seenMinimizeToTrayWarningPref: false,
  [SETTING_TRAY_ENABLED]: !IS_LINUX,
  [SETTING_TRAY_CLICK_SHORTCUT]: 'double-click',
  [SETTING_CUSTOM_WORDS]: {}
};

const deepCloneFallback = (value) => {
  if (value === null || typeof value !== 'object') {
    return value;
  }

  if (typeof structuredClone === 'function') {
    return structuredClone(value);
  }

  if (Array.isArray(value)) {
    return value.map((item) => deepCloneFallback(item));
  }

  return Object.keys(value).reduce((clone, key) => {
    clone[key] = deepCloneFallback(value[key]);
    return clone;
  }, {});
};

class SettingsManager {
  /**
   * Gets a setting value.
   * @param {string} key - The setting key to get.
   * @param {*} [defaultValue] - Optional default value to return if the setting does not exist and has no defined default.
   * @returns {*} The setting value.
   */
  get(key, defaultValue = undefined) {
    const fallback = key in defaults ? defaults[key] : defaultValue;
    const safeFallback = deepCloneFallback(fallback);
    return settings.get(key, safeFallback);
  }

  /**
   * Sets a setting value.
   * @param {string} key - The setting key to set.
   * @param {*} value - The value to set.
   */
  set(key, value) {
    settings.set(key, value);
  }

  /**
   * Watches a setting for changes.
   * @param {string} key - The setting key to watch.
   * @param {Function} handler - The callback function when the setting changes.
   */
  watch(key, handler) {
    settings.watch(key, handler);
  }
}

const settingsManager = new SettingsManager();
export default settingsManager;
