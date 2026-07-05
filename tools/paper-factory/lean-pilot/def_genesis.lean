/- def:genesis
   The genesis chain value is Genesis = chainH(0_32); a run may additionally bind
   a domain seed Genesis_dom = chainH("project-version-genesis") so chains from
   distinct projects or versions cannot cross-verify.

   Bare Lean 4 core, no mathlib. `chainH` (the chain hash function) is modeled as
   an opaque function from byte arrays to digests, since its concrete
   construction (BLAKE3) is outside the scope of this statement. -/

/-- A chain digest, represented as a byte array. -/
def Digest := ByteArray

/-- The chain hash function, treated abstractly. -/
axiom chainH : ByteArray → Digest

/-- 32 zero bytes, `0_{32}`. -/
def zeros32 : ByteArray := ByteArray.mk (List.replicate 32 (0 : UInt8)).toArray

/-- The genesis chain value, `Genesis = chainH(0_{32})`. -/
noncomputable def genesis : Digest := chainH zeros32

/-- Domain-seed bytes, `"project-version-genesis"`. -/
def domainSeedBytes : ByteArray := (String.toUTF8 "project-version-genesis")

/-- The domain-bound genesis value, `Genesis_dom = chainH("project-version-genesis")`,
    used so chains from distinct projects or versions cannot cross-verify. -/
noncomputable def genesisDom : Digest := chainH domainSeedBytes
