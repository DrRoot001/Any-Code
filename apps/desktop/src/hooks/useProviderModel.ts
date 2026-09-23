import { useCallback, useEffect, useState } from "react";
import { providerCommands, type ModelDefinition, type ProviderStatus } from "../lib/tauri";

/**
 * Provider + model selection, shared by every surface that talks to a model. Both the
 * provider list and the model list are fetched live — there is no hardcoded catalog, so
 * a provider with no key configured simply isn't offered.
 */
export function useProviderModel() {
  const [providers, setProviders] = useState<ProviderStatus[] | null>(null);
  const [providersError, setProvidersError] = useState<string | null>(null);
  const [provider, setProvider] = useState<string | null>(null);
  const [models, setModels] = useState<ModelDefinition[] | null>(null);
  const [modelsError, setModelsError] = useState<string | null>(null);
  const [model, setModel] = useState<string | null>(null);

  const refreshProviders = useCallback(() => {
    providerCommands
      .listProviders()
      .then((list) => {
        setProviders(list);
        setProvidersError(null);
        setProvider((current) => current ?? list.find((p) => p.ready)?.id ?? null);
      })
      .catch((error) => setProvidersError(String(error)));
  }, []);

  useEffect(refreshProviders, [refreshProviders]);

  useEffect(() => {
    if (!provider) {
      setModels(null);
      return;
    }
    setModel(null);
    setModelsError(null);
    providerCommands
      .listModels(provider)
      .then((list) => {
        setModels(list);
        setModel(list[0]?.id ?? null);
      })
      .catch((error) => setModelsError(String(error)));
  }, [provider]);

  /** Only providers that are configured. Whether one answers shows in its model list. */
  const connected = providers?.filter((p) => p.ready) ?? [];

  return {
    providers,
    providersError,
    connected,
    provider,
    setProvider,
    models,
    modelsError,
    model,
    setModel,
    refreshProviders,
  };
}
