# **Project Unifikátor: Autonomous Agent Guidelines**

Hello Jules. You are tasked with a major refactoring project. You are converting an old, unmaintained Electron-based Google Messages wrapper into a lightweight, modern Tauri-based application called "Unifikátor".

## **Core Philosophy (The "Goat Principle")**

1. **Function over Form:** We prefer pragmatic, working solutions over over-engineered ones. If a simple Vanilla JS hack solves a DOM injection issue better than a 500-line state manager, use the hack.  
2. **Lightweight & Lean:** NO heavy frontend frameworks (No React, No Vue, No Svelte). Use standard Vanilla JS/TS, ES6 modules, and basic HTML/CSS. The target memory footprint is \< 150MB.  
3. **OS Native:** Rely entirely on OS webviews (WebKitGTK on Linux, WebView2 on Windows) via Tauri. Do absolutely not bundle Chromium.  
4. **DeepMind Method (Generate \-\> Filter \-\> Apply):** Never apply extensive changes directly. Generate the solution, verify it (Linter/Dry-Run), and only then patch. Prevent hallucinations in the final commit.

## **The Goal & Architecture**

### **1\. Rip out Electron**

* Completely remove all Electron dependencies from package.json.  
* Delete src/main.js, src/background.js, and any Electron-specific IPC bridge files.  
* Remove old build pipelines meant for Electron Builder.

### **2\. Initialize Tauri Backend (Rust)**

* Set up a fresh Tauri V2 project structure.  
* Configure tauri.conf.json with strict security scopes. Only expose necessary IPC endpoints (e.g., trigger\_os\_notification, play\_edge\_tts).  
* Configure the build target primarily for Linux AppImage (and MSI for Windows later).

### **3\. Unified Shell Interface (Frontend)**

* Create a lightweight HTML shell (index.html).  
* **Layout:** A thin left sidebar (CSS Flexbox/Grid, \~64px wide) containing system icons (Messages, Chat, Mute, Settings).  
* **View Strategy:** Do NOT force both Google services into a single DOM. Use two independent iframe elements or separate Tauri webviews for https://messages.google.com/web and https://chat.google.com/.  
* Switch visibility via basic CSS (display: none or z-index) based on sidebar clicks to preserve session state and avoid reloads.

### **4\. Autonomous Voice Features (No API Keys required)**

* **STT (Speech-to-Text \- Input):** \* Register a global system shortcut in Rust (e.g., Ctrl+Space).  
  * When triggered, send an IPC event to the frontend.  
  * The frontend preload.js script catches this and triggers the native browser window.SpeechRecognition API.  
  * Inject the recognized text directly into the active contenteditable div of the current Google service and simulate an "Enter" keypress event.  
* **TTS (Text-to-Speech \- Output):**  
  * Use a MutationObserver inside preload.js to monitor the DOM for newly appended chat bubbles.  
  * Extract the text of new, incoming messages and send it to Rust via window.\_\_TAURI\_\_.invoke('play\_edge\_tts', { text }).  
  * **Rust Backend:** Implement an IPC listener using crates like reqwest (or a websocket client) to fetch audio from the undocumented MS Edge TTS API (which provides high-quality neural voices for free). Play the resulting audio buffer asynchronously using the rodio crate.

### **5\. System Integration & Notifications**

* **Notification Shim:** Google uses Service Workers and standard window.Notification. In your preload.js, completely override the window.Notification object.  
* Intercept the title and body, block the default web notification, and emit an IPC call to Rust.  
* **Rust Handling:** Catch the IPC call and trigger a native OS notification using Tauri's notification API.  
* **System Tray:** Implement tauri::SystemTray. Add a basic context menu (Show App, Mute TTS, Quit). When a notification arrives, update the tray icon to a version with a red notification badge.

## **Execution Steps for Jules**

1. **Environment Validation:** Execute jules\_setup.sh and verify all GTK/WebKit dependencies are installed on your VM.  
2. **Purge Legacy:** Strip out all Electron code. Clean the repository.  
3. **Scaffold Tauri:** Run standard Tauri initialization. Set up the Rust backend and the Vanilla HTML/JS frontend shell.  
4. **Implement Shell & Views:** Build the sidebar and the switching logic for the two Google iframes/webviews. Handle session persistence (cookies/storage).  
5. **Inject Bridges:** Write the preload.js script to handle DOM mutation tracking, SpeechRecognition, and Notification overriding.  
6. **Rust IPC & Audio:** Implement the TTS Edge API fetcher, audio playback (rodio), and global shortcut listener.  
7. **Tray Integration:** Wire up the System Tray and OS notifications.  
8. **Test Build:** Run pnpm tauri build. Ensure it successfully compiles into a Linux AppImage without errors.  
9. **Pull Request:** Submit a structured PR with a summary of the architectural changes.
