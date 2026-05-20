document.addEventListener('DOMContentLoaded', () => {
  console.log('Unifikátor Preload Script Injected');

  const OriginalNotification = window.Notification;
  if (OriginalNotification) {
    class CustomNotification {
      constructor(title, options) {
        this.title = title;
        this.options = options;
        window.__TAURI_INTERNALS__.invoke('trigger_os_notification', {
            title: title,
            body: options ? options.body : '',
            service: window.location.href.includes('chat.google.com') ? 'chat' : 'messages'
        });
      }
      static get permission() { return 'granted'; }
      static requestPermission(callback) { if (callback) callback('granted'); return Promise.resolve('granted'); }
    }
    window.Notification = CustomNotification;
  }

  const observeChatMutations = () => {
    const isMessages = window.location.href.includes('messages.google.com');
    const isChat = window.location.href.includes('chat.google.com');
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === 'childList' && mutation.addedNodes.length > 0) {
          mutation.addedNodes.forEach(node => {
            if (node.nodeType === Node.ELEMENT_NODE) {
              let textToRead = null;
              if (isMessages && node.tagName && node.tagName.toLowerCase() === 'mws-message-part') {
                  const content = node.textContent;
                  if (content && content.trim().length > 0) textToRead = content;
              } else if (isChat && (node.getAttribute('role') === 'listitem' || node.classList.contains('c-P'))) {
                  const textContentDiv = node.querySelector('[data-text]');
                  textToRead = textContentDiv ? textContentDiv.textContent : node.textContent;
              }
              if (textToRead && textToRead.trim().length > 0) {
                window.__TAURI_INTERNALS__.invoke('play_edge_tts', { text: textToRead.trim() });
              }
            }
          });
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
  };
  observeChatMutations();

  window.__TAURI_INTERNALS__.listen('start_stt', (event) => {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (!SpeechRecognition) return;
      const recognition = new SpeechRecognition();
      recognition.lang = 'cs-CZ';
      recognition.interimResults = false;
      recognition.maxAlternatives = 1;
      recognition.onresult = (e) => {
        const transcript = e.results[0][0].transcript;
        let inputField = document.querySelector('textarea[placeholder="Text message"]') || document.querySelector('[contenteditable="true"]');
        if (inputField) {
          inputField.focus();
          if (inputField.tagName.toLowerCase() === 'textarea') {
            inputField.value = transcript;
            inputField.dispatchEvent(new Event('input', { bubbles: true }));
          } else {
            inputField.textContent = transcript;
            inputField.dispatchEvent(new Event('input', { bubbles: true }));
          }
          setTimeout(() => {
            const enterEvent = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, key: 'Enter', code: 'Enter' });
            inputField.dispatchEvent(enterEvent);
          }, 100);
        }
      };
      recognition.start();
  });
});
