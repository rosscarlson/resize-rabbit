import { getVersion } from "@tauri-apps/api/app";
import { checkUpdate, installUpdate } from "@tauri-apps/api/updater";
import { relaunch } from "@tauri-apps/api/process";
import { ask } from "@tauri-apps/api/dialog";
import backend from "./utils/backend";
import i18n from "./utils/i18n/i18n";
import translate from "./utils/i18n/translate";
import { setSettings } from "./state/settingsState";
import { setAppVersion } from "./state/appVersionState";
import { refreshProfiles } from "./state/profileState";
import { BootState, setBootState } from "./state/bootState";
import { batch } from "@preact/signals-react";

async function loadLocale(lang: string): Promise<void> {
    if (!lang || lang === 'en') return; // en is already bundled statically
    try {
        const json: string = await backend.locale.load(lang);
        const data = JSON.parse(json);
        i18n.addResourceBundle(lang, 'translation', data, true, true);
        await i18n.changeLanguage(lang);
    } catch {
        // silently fall back to English
    }
}

// Runs on every launch (not just first install/run): if there are no Resize
// Rabbit profiles yet but legacy Resize Raccoon profiles are available, offer
// to import them - equivalent to clicking Import in Settings.
async function maybePromptLegacyImport(): Promise<void> {
    try {
        const profiles = await backend.profile.all();
        if (profiles.length > 0) return;

        const legacyAvailable = await backend.profile.legacyAvailable();
        if (!legacyAvailable) return;

        const shouldImport = await ask(translate('import.promptMessage'), {
            title: translate('import.promptTitle'),
            type: 'info',
        });

        if (shouldImport) {
            await backend.profile.importLegacy();
            await refreshProfiles();
        }
    } catch {
        // silently skip the prompt on any failure
    }
}

(async () => {
    const [settings, version] = await Promise.all([
        backend.settings.all(),
        getVersion(),
    ]);

    await loadLocale(settings.language);

    batch(() => {
        setSettings(settings);
        setAppVersion(version);
        setBootState(BootState.READY);
    });

    maybePromptLegacyImport();

    if (settings.checkForUpdates) {
        checkUpdate().then(async ({ shouldUpdate, manifest }) => {
            if (shouldUpdate) {
                const yes = await ask(
                    `Version ${manifest?.version} is available. Install update now?`,
                    { title: 'Update Available', type: 'info' }
                );
                if (yes) {
                    await installUpdate();
                    await relaunch();
                }
            }
        }).catch(() => {});
    }
})();
