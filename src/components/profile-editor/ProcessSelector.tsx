import { useEffect, useState } from "react";
import Process from "../../types/ProcessType";
import backend from "../../utils/backend";
import LoadingButton from "../LoadingButton";
import { RefreshCw } from "react-feather";
import { useTranslation } from "../../utils/i18n/useTranslation";

interface Props {
    selectedProcess?: Process;
    processNameValue?: string;
    onChange: (process: Process) => void;
}

const MANUAL_ENTRY_VALUE = "manual";

const sortByName = (processes: Process[]) =>
    [...processes].sort((a, b) =>
        a.name.toLowerCase().localeCompare(b.name.toLowerCase())
    );

const ProcessSelector = ({ selectedProcess, processNameValue, onChange }: Props) => {
    const t = useTranslation();
    const [processes, setProcesses] = useState<Process[]>([]);
    const [showAll, setShowAll] = useState(false);
    const [manualEntry, setManualEntry] = useState(false);
    const [manualValue, setManualValue] = useState("");

    const fetchProcesses = (showAllProcesses: boolean) =>
        backend.process.running(showAllProcesses).then((list) =>
            setProcesses(sortByName(list))
        );

    useEffect(() => {
        fetchProcesses(showAll);
    }, [showAll]);

    const handleReloadProcesses = (stopLoading: () => void) => {
        fetchProcesses(showAll).finally(stopLoading);
    };

    const handleSelectChange = (value: string) => {
        if (value === MANUAL_ENTRY_VALUE) {
            setManualEntry(true);
            return;
        }

        setManualEntry(false);
        onChange(processes.find((p) => p.pid === parseInt(value))!);
    };

    const handleManualChange = (value: string) => {
        setManualValue(value);
        onChange({ name: value.trim() });
    };

    return (
        <div>
            <div className="flex gap-2">
                <select
                    id="process"
                    className="select w-full"
                    onChange={(e) => handleSelectChange(e.target.value)}
                    value={manualEntry ? MANUAL_ENTRY_VALUE : selectedProcess?.pid}
                    defaultValue="default"
                >
                    {processNameValue && !selectedProcess ? (
                        <option value="default">{processNameValue}</option>
                    ) : (
                        <option value="default">{t('profile.process.select')}</option>
                    )}

                    <option value={MANUAL_ENTRY_VALUE}>
                        {t('profile.process.manualEntry')}
                    </option>

                    {processes.map((p) => (
                        <option key={p.pid} value={p.pid}>
                            {p.name}
                        </option>
                    ))}
                </select>
                <LoadingButton
                    className="btn btn-outline"
                    onClick={handleReloadProcesses}
                    onlySpinner
                >
                    <RefreshCw size={16} />
                </LoadingButton>
            </div>

            {manualEntry && (
                <input
                    type="text"
                    className="input w-full mt-2"
                    placeholder={t('profile.process.manualPlaceholder')}
                    value={manualValue}
                    onChange={(e) => handleManualChange(e.target.value)}
                    autoFocus
                />
            )}

            <label className="label cursor-pointer justify-start gap-2 items-center p-0 mt-2">
                <input
                    type="checkbox"
                    className="toggle toggle-accent toggle-sm"
                    checked={showAll}
                    onChange={(e) => setShowAll(e.target.checked)}
                />
                <span className="label-text text-2xs">
                    {t('profile.process.showAll')}
                </span>
            </label>
        </div>
    );
};

export default ProcessSelector;
