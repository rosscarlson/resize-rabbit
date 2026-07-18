import { useEffect, useMemo, useState } from 'react';
import Process from '../types/ProcessType';
import { Profile } from '../types/ProfileTypes';
import { v4 as uuidv4 } from 'uuid';
import backend from '../utils/backend';
import { presets } from '../constants';
import { setScreen } from '../state/screenState';
import { Screen } from '../types/ScreenTypes';
import { refreshProfiles, removeProfile } from '../state/profileState';
import FormControl from '../components/profile-editor/FormControl';
import ShortcutCapture from '../components/profile-editor/ShortcutCapture';
import { useTranslation } from '../utils/i18n/useTranslation';
import ProcessSelector from '../components/profile-editor/ProcessSelector';
import ProfileEditorHeader from '../components/profile-editor/ProfileEditorHeader';
import ProfileEditorFooter from '../components/profile-editor/ProfileEditorFooter';

interface Props {
    profile?: Profile;
}

const ProfileEditor = ({ profile }: Props) => {
    const t = useTranslation();
    const [name, setName] = useState<string>(profile?.name || '');
    const [selectedProcess, setSelectedProcess] = useState<Process>();
    const [processName] = useState<string>(profile?.processName || '');
    const [windowWidth, setWindowWidth] = useState<string>(
        String(profile?.windowWidth) || ''
    );
    const [windowHeight, setWindowHeight] = useState<string>(
        String(profile?.windowHeight) || ''
    );
    const [windowPosX, setWindowPosX] = useState<string>(
        String(profile?.windowPosX) || ''
    );
    const [windowPosY, setWindowPosY] = useState<string>(
        String(profile?.windowPosY) || ''
    );
    const [autoResize, setAutoResize] = useState<boolean>(
        profile?.auto || false
    );
    const [removeBorders, setRemoveBorders] = useState<boolean>(
        profile?.removeBorders || false
    );
    const [shiftTitlebarOffscreen, setShiftTitlebarOffscreen] =
        useState<boolean>(profile?.shiftTitlebarOffscreen || false);
    const [delay, setDelay] = useState<string>(String(profile?.delay) || '');
    const [shortcut, setShortcut] = useState<string>(profile?.shortcut || '');
    const [hasBeenSaved, setHasBeenSaved] = useState<boolean>(!!profile);
    const uuid = useMemo(() => {
        if (profile) {
            return profile.uuid;
        }

        return uuidv4();
    }, [profile]);

    const getUpdatedProfile = (): Profile => ({
        uuid,
        name,
        processName: processName || selectedProcess?.name || '',
        windowWidth: Number(windowWidth),
        windowHeight: Number(windowHeight),
        windowPosX: Number(windowPosX),
        windowPosY: Number(windowPosY),
        delay: Number(delay) || 0,
        auto: autoResize,
        removeBorders,
        shiftTitlebarOffscreen,
        shortcut: shortcut || undefined,
    });

    const handleCancel = () => {
        setScreen(Screen.HOME);
    };

    useEffect(() => {
        if (selectedProcess && !name) {
            setName(selectedProcess.name.split('.exe')[0]);
        }
    }, [selectedProcess, name]);

    const onTest = (stopLoading: () => void) => {
        const testProfile = getUpdatedProfile();
        backend.profile.test(testProfile).finally(stopLoading);
    };

    const testEnabled = useMemo(() => {
        return !!(
            windowWidth !== '' &&
            windowHeight !== '' &&
            (processName || selectedProcess)
        );
    }, [selectedProcess, processName, windowWidth, windowHeight]);

    const canSave = useMemo(() => {
        return !!(
            name &&
            (processName || selectedProcess) &&
            windowWidth &&
            windowHeight
        );
    }, [selectedProcess, processName, windowWidth, windowHeight, name]);

    const persistProfile = async () => {
        const updatedProfile = getUpdatedProfile();
        const endpoint = hasBeenSaved ? backend.profile.update : backend.profile.add;

        await endpoint(updatedProfile);
        setHasBeenSaved(true);
        await refreshProfiles();
    };

    const handleSaveAndClose = async () => {
        await persistProfile();
        setScreen(Screen.HOME);
    };

    const handleSaveOnly = async (stopLoading: () => void) => {
        await persistProfile();
        stopLoading();
    };

    const handleDelete = () => {
        if (!profile?.uuid) return;
        backend.profile.delete(profile).then(() => {
            removeProfile(profile.uuid);
            setScreen(Screen.HOME);
        });
    };

    const handlePresetSelection = (presetName: string) => {
        const preset = presets.find((p) => p.name === presetName);
        if (!preset) return;

        setWindowWidth(String(preset.windowWidth));
        setWindowHeight(String(preset.windowHeight));
        setWindowPosX(String(preset.windowPosX));
        setWindowPosY(String(preset.windowPosY));
    };

    return (
        <div className="flex flex-col h-screen bg-gradient-to-t from-[#660e99] to-[#941882]">
            <ProfileEditorHeader />
            <div className="flex-grow w-full h-[100vh] grid grid-rows-[1fr_auto] pr-8 pl-8">
                <div className="grid grid-cols-2 gap-8 mb-8 ">
                    <div>
                        <FormControl
                            label={t('profile.process.title')}
                            id="process"
                            description={t('profile.process.description')}
                            tooltip="top-right"
                        >
                            <ProcessSelector
                                selectedProcess={selectedProcess}
                                processNameValue={processName}
                                onChange={(process) =>
                                    setSelectedProcess(process)
                                }
                            />
                        </FormControl>
                        <FormControl
                            id="name"
                            label={t('profile.profileName.title')}
                            description={t('profile.profileName.description')}
                            tooltip="top-right"
                        >
                            <input
                                type="text"
                                id="name"
                                className="input w-full"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                            />
                        </FormControl>
                        <div className="grid grid-cols-2 gap-4 mb-4">
                            <FormControl
                                id="auto_resize"
                                label={t('profile.autoResize.title')}
                                description={t(
                                    'profile.autoResize.description'
                                )}
                                tooltip="top-right"
                            >
                                <div className="h-[48px] flex items-center">
                                    <input
                                        id="auto_resize"
                                        type="checkbox"
                                        className="toggle toggle-accent toggle-lg"
                                        checked={autoResize}
                                        onChange={(e) =>
                                            setAutoResize(e.target.checked)
                                        }
                                    />
                                </div>
                            </FormControl>
                            <FormControl
                                id="auto_delay"
                                label={t('profile.autoResizeDelay.title')}
                                description={t(
                                    'profile.autoResizeDelay.description'
                                )}
                                tooltip="top-left"
                            >
                                <input
                                    type="number"
                                    id="auto_delay"
                                    className="input w-full"
                                    value={delay}
                                    onChange={(e) => setDelay(e.target.value)}
                                />
                            </FormControl>
                        </div>
                        <div className="grid grid-cols-2 gap-4 mb-4">
                            <FormControl
                                id="remove_borders"
                                label={t('profile.window.borderless.title')}
                                description={t(
                                    'profile.window.borderless.description'
                                )}
                                tooltip="top-right"
                            >
                                <div className="h-[48px] flex items-center">
                                    <input
                                        id="remove_borders"
                                        type="checkbox"
                                        className="toggle toggle-accent toggle-lg"
                                        checked={removeBorders}
                                        onChange={(e) =>
                                            setRemoveBorders(e.target.checked)
                                        }
                                    />
                                </div>
                            </FormControl>
                            <FormControl
                                id="shift_titlebar_offscreen"
                                label={t(
                                    'profile.window.titlebarOffscreen.title'
                                )}
                                description={t(
                                    'profile.window.titlebarOffscreen.description'
                                )}
                                tooltip="top-left"
                            >
                                <div className="h-[48px] flex items-center">
                                    <input
                                        id="shift_titlebar_offscreen"
                                        type="checkbox"
                                        className="toggle toggle-accent toggle-lg"
                                        checked={shiftTitlebarOffscreen}
                                        onChange={(e) =>
                                            setShiftTitlebarOffscreen(
                                                e.target.checked
                                            )
                                        }
                                    />
                                </div>
                            </FormControl>
                        </div>
                        <FormControl
                            id="shortcut"
                            label={t('profile.shortcut.title')}
                            description={t('profile.shortcut.description')}
                            tooltip="top-right"
                        >
                            <ShortcutCapture value={shortcut} onChange={setShortcut} />
                        </FormControl>
                    </div>
                    <div>
                        <FormControl
                            id="preset"
                            label={t('profile.preset.title')}
                            description={t('profile.preset.description')}
                            tooltip="top-center"
                        >
                            <select
                                id="preset"
                                className="select w-full"
                                onChange={(e) =>
                                    handlePresetSelection(e.target.value)
                                }
                                defaultValue="default"
                            >
                                <option value="default">
                                    Copy values from preset
                                </option>
                                {presets.map((p) => (
                                    <option key={p.name} value={p.name}>
                                        {p.name}
                                    </option>
                                ))}
                            </select>
                        </FormControl>
                        <div className="grid grid-cols-2 gap-4">
                            <FormControl
                                id="window_width"
                                label={t('profile.window.width.title')}
                                description={t(
                                    'profile.window.width.description'
                                )}
                                tooltip="top-center"
                            >
                                <input
                                    type="number"
                                    id="window_width"
                                    className="input w-full"
                                    value={windowWidth}
                                    onChange={(e) =>
                                        setWindowWidth(e.target.value)
                                    }
                                />
                            </FormControl>
                            <FormControl
                                id="window_height"
                                label={t('profile.window.height.title')}
                                description={t(
                                    'profile.window.height.description'
                                )}
                                tooltip="top-left"
                            >
                                <input
                                    type="number"
                                    id="window_height"
                                    className="input w-full"
                                    value={windowHeight}
                                    onChange={(e) =>
                                        setWindowHeight(e.target.value)
                                    }
                                />
                            </FormControl>
                        </div>
                        <div className="grid grid-cols-2 gap-4">
                            <FormControl
                                id="window_pos_x"
                                label={t('profile.window.posX.title')}
                                description={t(
                                    'profile.window.posX.description'
                                )}
                                tooltip="top-center"
                            >
                                <input
                                    type="number"
                                    id="window_pos_x"
                                    className="input w-full"
                                    value={windowPosX}
                                    onChange={(e) =>
                                        setWindowPosX(e.target.value)
                                    }
                                />
                            </FormControl>
                            <FormControl
                                id="window_pos_y"
                                label={t('profile.window.posY.title')}
                                description={t(
                                    'profile.window.posY.description'
                                )}
                                tooltip="top-left"
                            >
                                <input
                                    type="number"
                                    id="window_pos_y"
                                    className="input w-full"
                                    value={windowPosY}
                                    onChange={(e) =>
                                        setWindowPosY(e.target.value)
                                    }
                                />
                            </FormControl>
                        </div>
                    </div>
                </div>
            </div>
            <ProfileEditorFooter
                onTest={onTest}
                canTest={testEnabled}
                onDelete={profile ? handleDelete : undefined}
                onCancel={handleCancel}
                onSave={handleSaveAndClose}
                onSaveOnly={handleSaveOnly}
                canSave={canSave}
            />
        </div>
    );
};

export default ProfileEditor;
