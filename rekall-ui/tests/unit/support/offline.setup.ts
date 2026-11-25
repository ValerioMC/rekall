/** A request no test fakes stays pending: its failure would raise a toast during a later assertion, or after the environment is torn down. */
globalThis.fetch = () => new Promise<never>(() => {})
