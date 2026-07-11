import { useState } from 'react';
import { Info, Download } from 'react-feather';
import { getSettings, updateSettings } from '../state/settingsState';
import { useTranslation } from '../utils/i18n/useTranslation';
import backend from '../utils/backend';
import { refreshProfiles } from '../state/profileState';
import { addToast } from './toast/ToastSystem';
import { ToastType } from './toast/toast.types';

const SettingsMenu = () => {
    const t = useTranslation();
    const [processWatcherPollRate, setProcessWatcherPollRate] =
        useState<string>(String(getSettings().pollRate));

    const handlePollRateBlur = async () => {
        if (isNaN(Number(processWatcherPollRate))) {
            setProcessWatcherPollRate(String(getSettings().pollRate));
            return;
        }

        if (Number(processWatcherPollRate) === getSettings().pollRate) return;

        updateSettings({ pollRate: Number(processWatcherPollRate) });
    };

    const handleCheckForUpdatesToggle = (
        e: React.ChangeEvent<HTMLInputElement>
    ) => {
        updateSettings({ checkForUpdates: e.target.checked });
    };

    const handleLaunchOnStartToggle = (
        e: React.ChangeEvent<HTMLInputElement>
    ) => {
        updateSettings({ launchOnStart: e.target.checked }, true);
        backend.settings.toggleLaunchOnStart(e.target.checked).catch(() => {
            updateSettings({ launchOnStart: !e.target.checked }, true);
        });
    };

    const handleStartMinimizedToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
        updateSettings({ startMinimized: e.target.checked });
    }

    const handleCloseToTrayToggle = (e: React.ChangeEvent<HTMLInputElement>) => {
        updateSettings({ closeToTray: e.target.checked });
    }

    const [importing, setImporting] = useState(false);
    const handleImportLegacy = async () => {
        setImporting(true);
        try {
            const count: number = await backend.profile.importLegacy();
            if (count > 0) {
                await refreshProfiles();
                addToast({ type: ToastType.SUCCESS, message: `Imported ${count} profile(s) from Resize Raccoon.` });
            } else {
                addToast({ type: ToastType.INFO, message: 'No new profiles found in Resize Raccoon data.' });
            }
        } catch {
            addToast({ type: ToastType.ERROR, message: 'Could not find Resize Raccoon data to import.' });
        } finally {
            setImporting(false);
        }
    };

    return (
        <div className="sidebar bg-base-100 drop-shadow-2xl flex flex-col p-3 pt-16">
            <div className="form-control w-full">
                <label className="label pb-1" htmlFor="launch_on_start">
                    <span className="text-2xs uppercase font-semibold">
                        {t('settings.launchOnStart.title')}
                    </span>
                </label>
                <input
                    id="launch_on_start"
                    type="checkbox"
                    className="toggle toggle-accent toggle-md"
                    checked={getSettings().launchOnStart}
                    onChange={handleLaunchOnStartToggle}
                />
            </div>
            <div className="divider mt-2 mb-1" />
            <div className="form-control w-full">
                <label className="label pb-1" htmlFor="start_minimized">
                    <span className="text-2xs uppercase font-semibold">
                        {t('settings.startMinimized.title')}
                    </span>
                    <div
                        className="tooltip before:w-[300px] before:-left-[10px] before:translate-x-0"
                        data-tip={t('settings.startMinimized.description')}
                    >
                        <Info size=".8em" />
                    </div>
                </label>
                <input
                    id="start_minimized"
                    type="checkbox"
                    className="toggle toggle-accent toggle-md"
                    checked={getSettings().startMinimized}
                    onChange={handleStartMinimizedToggle}
                />
            </div>
            <div className="divider mt-2 mb-1" />
            <div className="form-control w-full">
                <label className="label pb-1" htmlFor="close_to_tray">
                    <span className="text-2xs uppercase font-semibold">
                        {t('settings.closeToTray.title')}
                    </span>
                    <div
                        className="tooltip before:w-[300px] before:-left-[10px] before:translate-x-0"
                        data-tip={t('settings.closeToTray.description')}
                    >
                        <Info size=".8em" />
                    </div>
                </label>
                <input
                    id="close_to_tray"
                    type="checkbox"
                    className="toggle toggle-accent toggle-md"
                    checked={getSettings().closeToTray}
                    onChange={handleCloseToTrayToggle}
                />
            </div>
            <div className="divider mt-2 mb-1"></div>
            <div className="form-control w-full">
                <label className="label pb-1" htmlFor="updates_on_launch">
                    <span className="text-2xs uppercase font-semibold">
                        {t('settings.checkForUpdates.title')}
                    </span>
                </label>
                <input
                    id="updates_on_launch"
                    type="checkbox"
                    className="toggle toggle-accent toggle-md"
                    checked={getSettings().checkForUpdates}
                    onChange={handleCheckForUpdatesToggle}
                />
            </div>
            <div className="divider mt-2 mb-1"></div>
            <div className="form-control w-full mb-4">
                <label className="label pb-1">
                    <span className="text-2xs uppercase font-semibold">
                        Import profiles
                    </span>
                </label>
                <button
                    className="btn btn-outline btn-sm gap-2"
                    onClick={handleImportLegacy}
                    disabled={importing}
                >
                    {importing
                        ? <span className="loading loading-spinner w-4" />
                        : <Download size={14} />
                    }
                    Import from Resize Raccoon
                </button>
            </div>
            <div className="divider mt-2 mb-1"></div>
            <div className="form-control w-full mb-4">
                <label
                    htmlFor="pollRate"
                    className="label pb-1 justify-start gap-2"
                >
                    <span className="text-2xs uppercase font-semibold">
                        {t('settings.processPollRate.title')}
                    </span>
                    <div
                        className="tooltip before:w-[300px] before:-left-[10px] before:translate-x-0"
                        data-tip={t('settings.processPollRate.description')}
                    >
                        <Info size=".8em" />
                    </div>
                </label>
                <input
                    type="number"
                    id="pollRate"
                    className="input input-bordered w-full"
                    value={processWatcherPollRate}
                    onChange={(e) => setProcessWatcherPollRate(e.target.value)}
                    onBlur={handlePollRateBlur}
                />
            </div>
        </div>
    );
};

export default SettingsMenu;
