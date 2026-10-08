import { useEffect, useState } from "preact/hooks";
import { errorMessage, getSettings, saveSettings } from "../../shared/api";
import { text } from "../../shared/text";
import type { Settings } from "../../shared/types";

export function useSettingsPage() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [value, setValue] = useState("");
  const [fixRandomSeed, setFixRandomSeed] = useState(true);
  const [error, setError] = useState("");
  const [saved, setSaved] = useState(false);
  const [saving, setSaving] = useState(false);
  const [attempt, setAttempt] = useState(0);
  const valid = /^\d+$/.test(value) && Number(value) >= 1 && Number(value) <= 999999;

  useEffect(() => {
    let active = true;
    setError("");
    void getSettings().then((result) => {
      if (!active) return;
      setSettings(result);
      setValue(String(result.logRetention));
      setFixRandomSeed(result.fixRandomSeed);
    }).catch((reason: unknown) => {
      if (active) setError(errorMessage(reason));
    });
    return () => { active = false; };
  }, [attempt]);

  async function save() {
    if (saving || !settings) return;
    setError("");
    setSaved(false);
    if (!valid) { setError(text("pages.settings.invalid")); return; }
    setSaving(true);
    try {
      const result = await saveSettings(Number(value), fixRandomSeed);
      setSettings(result);
      setValue(String(result.logRetention));
      setSaved(true);
      setFixRandomSeed(result.fixRandomSeed);
    } catch (reason) {
      setError(`${text("pages.settings.saveError")} ${errorMessage(reason)}`);
    } finally { setSaving(false); }
  }

  return {
    settings, value, fixRandomSeed, error, saved, saving, valid, save,
    changeFixRandomSeed(next: boolean) { setFixRandomSeed(next); setSaved(false); },
    changeValue(next: string) { setValue(next); setSaved(false); },
    retry() { setAttempt((previous) => previous + 1); },
  };
}
