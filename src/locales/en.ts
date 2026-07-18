const en = {
    "toast": {
        "buttons": {
            "dismiss": "Dismiss"
        }
    },
    "profile": {
        "process": {
            "title": "Process",
            "description": "The application you want to create a resize profile for. If you launched the program after opening this screen you should click the refresh button.",
            "select": "Select a process",
            "showAll": "Show all processes",
            "manualEntry": "Enter process name manually…",
            "manualPlaceholder": "e.g. RRRE64.exe"
        },
        "profileName": {
            "title": "Profile name",
            "description": "The name of the profile you want to create. This will be used to identify the preset in the list of profiles.",
        },
        "preset": {
            "title": "Preset",
            "description": "Pre calculated values for some triple monitor setups.",
            "select": "Copy values from preset",
            "options": {
                "triple1080p": "Triple 1080p",
                "triple1440p": "Triple 1440p",
                "triple4k": "Triple 4k",
            }
        },
        "window" : {
            "width": {
                "title": "Window width",
                "description": "Probably self-explanatory, the intended width of the window. For triple monitor setups this is 3x a single screens horizontal pixels.",
            },
            "height": {
                "title": "Window height",
                "description": "Probably self-explanatory, the intended height of the window. For triple monitor setups this is the height of a single screen.",
            },
            "posY": {
                "title": "Window position y",
                "description": "The vertical position of the window on the screen. Usually 0.",
            },
            "posX": {
                "title": "Window position x",
                "description": "The horizontal position of the window on the screen. Depends on your setup. For triple monitors usually a negative value equal to the width of a single screen.",
            },
            "borderless": {
                "title": "Remove borders",
                "description": "Remove the borders from the window. Only use this if you cannot select borderless in-game.",
            },
            "titlebarOffscreen": {
                "title": "Remove titlebar (experimental)",
                "description": "Pushes the window up and off the top of the screen by the height of its title bar, so the title bar itself ends up hidden above the display. Currently only works for some Store/UWP games (e.g. Forza Horizon 4) that still show a title bar after removing borders — does nothing for other games.",
            }
        },
        "autoResize": {
            "title": "Auto resize",
            "description": "Allow this profile to be automatically applied when the program is launched (requires global process watching).",
            "enabled": "Automatic",
            "disabled": "Manual",
        },
        "autoResizeDelay": {
            "title": "Auto resize delay (ms)",
            "description": "The delay in milliseconds between the program launching and the profile being applied.",
        },
        "shortcut": {
            "title": "Shortcut Key",
            "description": "Global hotkey to apply this profile even when the app is minimized. Must include at least one modifier (Ctrl, Alt, Shift).",
            "placeholder": "Click to set shortcut",
            "listening": "Press a key combo... (Esc to cancel)",
            "clear": "Clear shortcut",
        },
        "buttons": {
            "test": "Test | Apply",
            "cancel": "Cancel",
            "save": "Save",
            "saveAndClose": "Save & Close",
            "delete": "Delete",
        },
    },
    "home": {
        "homepage": "Homepage",
        "processWatcher": "Process watcher",
        "beta": "beta",
    },
    "import": {
        "label": "Import Resize Raccoon Profiles",
        "button": "Import",
        "success": "Imported {{count}} profile(s) from Resize Raccoon.",
        "none": "No new profiles found in Resize Raccoon data.",
        "error": "Could not find Resize Raccoon data to import.",
        "promptTitle": "Import Resize Raccoon Profiles?",
        "promptMessage": "We found existing Resize Raccoon profiles but no Resize Rabbit profiles yet. Import them now?",
    },
    "settings": {
        "language": {
            "title": "Language",
        },
        "checkForUpdates": {
            "title": "Check for updates on launch",
        },
        "processPollRate": {
            "title": "Process poll rate (ms)",
            "description": "How often should we check for new applications being launched.",
        },
        "launchOnStart": {
            "title": "Start with windows"
        },
        "startMinimized": {
            "title": "Start minimized",
            "description": "Start the program minimized to the system tray.",
        },
        "closeToTray": {
            "title": "Close to tray",
            "description": "Close the program to the system tray instead of exiting.",
        },
        "loggingEnabled": {
            "title": "Logging",
            "description": "Write diagnostic logs to a 'logs' folder next to the application. Useful when troubleshooting an issue — turn off when you're done.",
        }
    },
    "attribution": "Based on {{name}} by {{author}}",
    "errors": {
        "window_manager": {
            "process_not_found": "Process not found, make sure the application your trying to control is running.",
            "apply_failed": "Failed to apply profile, not sure why. Try again?",
            "access_deined": "Failed to apply profile, access denied. Try running Resize Rabbit as admin.",
            "invalid_pid": "Failed to apply profile, invalid process id. You'll never see this error. If you do, you win a prize.",
        },
        "profile": {
            "not_found": "Profile not found, restart the program to sync profiles.",
            "profile_path_error": "Couldnt locate the profile path, this is bad, but shouldnt happen. Idk, just dont delete the application data folder I guess.",
        },
        "settings": {
            "launch_on_start_error": "Unable to toggle launch on start",
            "unable_to_fetch_app_data": "Unable to fetch application meta data",
            "settings_path_error": "Could not locate the settings.json file",
        },
        "unknown": "Unexpected error"
    }
}

export default en;
