

#[cfg(test)]
thread_local! {
    /// How many patterns the lookups of tiling patterns on this thread compared with the pattern
    /// they looked for.
    static TILINGS_COMPARED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// [`TILINGS_COMPARED`] of this thread.
#[cfg(test)]
pub(crate) fn tilings_compared() -> u64 {
    TILINGS_COMPARED.get()
}

