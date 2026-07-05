/-
ax:obs — The observation space.

There is a set Obs, the observation space, whose elements are arbitrary finite
records: logs, agent outputs, sensor frames, claims, tool responses. Obs carries
no decidable semantics: whether an observation means what it purports to mean
is not assumed computable.
-/

-- The observation space itself, as an abstract type of finite records.
axiom Obs : Type

-- The semantic predicate "this observation means what it purports to mean".
-- No decidability instance is assumed or derivable.
axiom ObsMeansWhatItPurports : Obs → Prop
