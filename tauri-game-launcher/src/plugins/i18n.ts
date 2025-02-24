// src/i18n.ts
import { createI18n } from 'vue-i18n';

const messages = {
  en: {
    primary: {
      verifying: "Verifying game client...",
      nogame: "No game client found!",
      creating: "Creating transaction...",
      awaiting: "Awaiting your approval...",
      retry: "Ready to Retry",
      downloading: "Downloading patches...",
      complete: "Your game is up-to-date!"
    },
    secondary: {
      verifying: "Please wait while we verify your installation.",
      nogame: "Are you sure you're in the right directory?",
      creating: "Setting up your transaction.",
      awaiting: "Approve the transaction to proceed.",
      retry: "Please try again.",
      downloading: "Downloading...",
      complete: "You can now launch your game."
    },
    buttons: {
      verifying: "Play",
      nogame: "Select Game Directory",
      creating: "Start Transaction",
      awaiting: "Approve",
      retry: "Retry",
      downloading: "Downloading",
      complete: "Launch Game"
    },
    toasts: {
      verify: "Error verifying game integrity\n{ error }",
      directory: "Error selecting directory\n{ error }",
      creating: "Error creating transaction\n{ error }",
      downloading: "Error downloading\n{ error }",
    }
  },
  // Other languages can be added here
};

const i18n = createI18n({
  legacy: false, // you can use the Composition API mode
  locale: 'en',
  fallbackLocale: 'en',
  messages,
});

export default i18n;
