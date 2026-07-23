import { useEffect, useState } from 'react';
import { X } from 'react-feather';
import { useTranslation } from '../../utils/i18n/useTranslation';
import backend from '../../utils/backend';

interface Props {
    value: string;
    onChange: (shortcut: string) => void;
}

const MODIFIER_KEYS = new Set(['Control', 'Alt', 'Shift', 'Meta']);

const KEY_MAP: Record<string, string> = {
    ' ': 'Space',
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'ArrowUp': 'Up',
    'ArrowDown': 'Down',
    'Escape': 'Escape',
    'Enter': 'Return',
    'Tab': 'Tab',
    'Backspace': 'Backspace',
    'Delete': 'Delete',
    'Home': 'Home',
    'End': 'End',
    'PageUp': 'PageUp',
    'PageDown': 'PageDown',
    'Insert': 'Insert',
};

function buildShortcut(e: KeyboardEvent): string | null {
    if (MODIFIER_KEYS.has(e.key)) return null;

    const parts: string[] = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');

    if (parts.length === 0) return null;

    const key = KEY_MAP[e.key] ?? (e.key.length === 1 ? e.key.toUpperCase() : e.key);
    parts.push(key);

    return parts.join('+');
}

const ShortcutCapture = ({ value, onChange }: Props) => {
    const t = useTranslation();
    const [listening, setListening] = useState(false);

    useEffect(() => {
        if (!listening) return;

        // A combo already bound to another profile is registered as a
        // system-wide hotkey — Windows delivers it as WM_HOTKEY instead of a
        // normal keystroke, so the page's own keydown listener below would
        // never see it at all while it's still registered. Suspend every
        // global shortcut for the duration of capture so any combo can be
        // read, then restore them (from whatever's actually saved) as soon
        // as capture ends, however it ends.
        backend.shortcuts.suspend();
        return () => {
            backend.shortcuts.resume();
        };
    }, [listening]);

    useEffect(() => {
        if (!listening) return;

        const onKeyDown = (e: KeyboardEvent) => {
            e.preventDefault();
            e.stopPropagation();
            if (e.key === 'Escape') {
                setListening(false);
                return;
            }
            const shortcut = buildShortcut(e);
            if (shortcut) {
                onChange(shortcut);
                setListening(false);
            }
        };

        const onMouseDown = (e: MouseEvent) => {
            const target = e.target as Element;
            if (!target.closest('[data-shortcut-capture]')) {
                setListening(false);
            }
        };

        document.addEventListener('keydown', onKeyDown, true);
        document.addEventListener('mousedown', onMouseDown);
        return () => {
            document.removeEventListener('keydown', onKeyDown, true);
            document.removeEventListener('mousedown', onMouseDown);
        };
    }, [listening, onChange]);

    return (
        <div className="flex gap-2" data-shortcut-capture>
            <div
                className={`input w-full flex items-center cursor-pointer select-none ${
                    listening ? 'input-accent' : ''
                }`}
                onClick={() => setListening(true)}
                data-shortcut-capture
            >
                {listening ? (
                    <span className="opacity-60 text-sm">{t('profile.shortcut.listening')}</span>
                ) : value ? (
                    <span className="font-mono text-sm tracking-wide">{value}</span>
                ) : (
                    <span className="opacity-40 text-sm">{t('profile.shortcut.placeholder')}</span>
                )}
            </div>
            {value && !listening && (
                <button
                    className="btn btn-ghost btn-sm btn-square shrink-0"
                    onClick={(e) => { e.stopPropagation(); onChange(''); }}
                    title={t('profile.shortcut.clear')}
                    data-shortcut-capture
                >
                    <X size={14} />
                </button>
            )}
        </div>
    );
};

export default ShortcutCapture;
