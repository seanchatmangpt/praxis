-- def:denialabstract
-- Fix n lanes; abstract denial word space Deny_n = {0,1}^n (Bool^n),
-- componentwise disjunction (or), all-zero word, support, and pointwise <=.

def Deny (n : Nat) : Type := Fin n → Bool

def Deny.zero (n : Nat) : Deny n := fun _ => false

def Deny.or {n : Nat} (d d' : Deny n) : Deny n :=
  fun i => (d i) || (d' i)

def Deny.isSet {n : Nat} (d : Deny n) (i : Fin n) : Prop := d i = true

def Deny.supp {n : Nat} (d : Deny n) : Fin n → Prop :=
  fun i => d i = true

def Deny.le {n : Nat} (d d' : Deny n) : Prop :=
  ∀ i : Fin n, d i = true → d' i = true

instance {n : Nat} : LE (Deny n) := ⟨Deny.le⟩
