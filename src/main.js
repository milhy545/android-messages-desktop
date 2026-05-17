const { Window } = window.__TAURI__.window;
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const appWindow = new Window('main');

document.addEventListener('DOMContentLoaded', () => {
  // Titlebar controls
  document.getElementById('titlebar-close').addEventListener('click', () => {
    // Instead of exiting, we hide the main window to tray
    appWindow.hide();
  });

  // Navigation Logic
  const navButtons = document.querySelectorAll('.nav-button');

  navButtons.forEach(btn => {
    btn.addEventListener('click', async () => {
      const targetId = btn.getAttribute('data-target');

      // Update buttons
      navButtons.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');

      // Tell backend to switch the active webview
      try {
        await invoke('switch_webview', { service: targetId });
      } catch (e) {
        console.error("Failed to switch webview", e);
      }
    });
  });

  // Action Buttons
  const btnMuteTts = document.getElementById('btn-mute-tts');
  let isMuted = false;
  btnMuteTts.addEventListener('click', () => {
    isMuted = !isMuted;
    btnMuteTts.classList.toggle('muted', isMuted);
    invoke('toggle_tts_mute', { muted: isMuted }).catch(console.error);
  });

  // Listen for navigation requests from backend (e.g., from tray or notification click)
  listen('navigate_to', (event) => {
    const service = event.payload.service;
    const btn = document.querySelector(`.nav-button[data-target="${service}"]`);
    if (btn) btn.click();
  });
});
