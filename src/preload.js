// This script runs in the context of the iframe/webview
// We need to access Tauri's IPC, so we expect the main window to proxy or we inject __TAURI_INTERNALS__

// Wait for the DOM to be ready
document.addEventListener('DOMContentLoaded', () => {
  console.log('Unifikátor Preload Script Injected');

  // 1. Notification Shim
  const OriginalNotification = window.Notification;

  if (OriginalNotification) {
    class CustomNotification {
      constructor(title, options) {
        this.title = title;
        this.options = options;

        console.log('Intercepted Notification:', title, options);

        // Notify the parent window (our Tauri shell) which will relay to Rust
        window.parent.postMessage({
          type: 'TRIGGER_NOTIFICATION',
          payload: {
            title: title,
            body: options ? options.body : '',
            icon: options ? options.icon : '',
            service: window.location.href.includes('chat.google.com') ? 'chat' : 'messages'
          }
        }, '*');
      }

      static get permission() {
        return 'granted';
      }

      static requestPermission(callback) {
        if (callback) callback('granted');
        return Promise.resolve('granted');
      }
    }

    // Override the global Notification object
    window.Notification = CustomNotification;
  }

  // 2. TTS Mutation Observer (Text-to-Speech)
  // This is highly dependent on Google's DOM structure.
  // Google Messages uses custom elements like 'mws-message-part'
  // Google Chat uses div elements with specific classes or roles.

  const observeChatMutations = () => {
    const isMessages = window.location.href.includes('messages.google.com');
    const isChat = window.location.href.includes('chat.google.com');

    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === 'childList' && mutation.addedNodes.length > 0) {
          mutation.addedNodes.forEach(node => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              let textToRead = null;

              if (isMessages) {
                // Heuristic for Google Messages incoming message
                // Usually it's not authored by us.
                if (node.tagName && node.tagName.toLowerCase() === 'mws-message-part') {
                  // Ensure it's an incoming message (heuristic: check class or parent classes)
                  // For simplicity, we just grab text and assume it's incoming if it lacks a "sent" class.
                  // Real implementation requires precise selectors.
                  const content = node.textContent;
                  if (content && content.trim().length > 0) {
                      textToRead = content;
                  }
                }
              } else if (isChat) {
                // Heuristic for Google Chat
                // Chat bubbles often have role="listitem" or specific jsnames
                if (node.getAttribute('role') === 'listitem' || node.classList.contains('c-P')) {
                  // Try to find the text content div inside the bubble
                  const textContentDiv = node.querySelector('[data-text]');
                  if (textContentDiv) {
                    textToRead = textContentDiv.textContent;
                  } else {
                    textToRead = node.textContent;
                  }
                }
              }

              if (textToRead && textToRead.trim().length > 0) {
                console.log('Extracted text for TTS:', textToRead);
                window.parent.postMessage({
                  type: 'PLAY_TTS',
                  payload: { text: textToRead.trim() }
                }, '*');
              }
            }
          });
        }
      }
    });

    // Start observing the body for added nodes
    // In a real scenario, we'd observe a more specific chat container
    observer.observe(document.body, { childList: true, subtree: true });
  };

  // Give the app some time to load before observing
  setTimeout(observeChatMutations, 3000);

  // 3. STT (Speech-to-Text) Listener
  window.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'START_STT') {
      console.log('Received STT trigger in iframe');

      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (!SpeechRecognition) {
        console.error('SpeechRecognition API not supported in this environment');
        return;
      }

      const recognition = new SpeechRecognition();
      recognition.lang = 'cs-CZ'; // Default to Czech as per persona, could be configurable
      recognition.interimResults = false;
      recognition.maxAlternatives = 1;

      recognition.start();

      recognition.onresult = (event) => {
        const transcript = event.results[0][0].transcript;
        console.log('STT Result:', transcript);

        // Find the active input field
        // Google Messages uses a textarea or contenteditable div
        // Google Chat uses a contenteditable div usually with aria-label="New message" or similar

        let inputField = document.querySelector('textarea[placeholder="Text message"]') ||
                         document.querySelector('[contenteditable="true"]');

        if (inputField) {
          inputField.focus();

          if (inputField.tagName.toLowerCase() === 'textarea') {
            inputField.value = transcript;
            inputField.dispatchEvent(new Event('input', { bubbles: true }));
          } else {
            // ContentEditable
            inputField.textContent = transcript;
            inputField.dispatchEvent(new Event('input', { bubbles: true }));
          }

          // Simulate Enter key to send
          setTimeout(() => {
            const enterEvent = new KeyboardEvent('keydown', {
              bubbles: true, cancelable: true, keyCode: 13, key: 'Enter'
            });
            inputField.dispatchEvent(enterEvent);
          }, 100);
        } else {
           console.warn('Could not find input field to inject STT text');
        }
      };

      recognition.onerror = (event) => {
        console.error('STT Error:', event.error);
      };
    }
  });
});
