# Approach — Vault (Turbin3 Q3-26)

> Reasoning log, written in my own words. Log entries are **append-only**: I do not delete my
> mistakes, they are part of what is being graded.
>
> **Note on language:** I dictate by voice in French, then translate. Every quoted passage below
> is my own reasoning — the English is the working version for graders, and my original French
> dictation is preserved underneath each quote so the authorship is verifiable.

---

## 1. Understanding (before writing code)

**What the assignment asks, in my own words:**

An Anchor program implementing a vault. A user can initialize a personal vault, deposit
lamports into it, withdraw from it, and close it. Each user gets their own vault, derived from
their public key.

**Inputs, outputs and constraints I identified:**

- Two PDAs per user:
  - `vault_state` — seeds `[b"state", user.key()]`, owned by my program, stores data.
  - `vault` — seeds `[b"vault", user.key()]`, owned by the System Program, holds only lamports.
- `user` must be a signer and must pay.
- Planned instructions: `initialize` (done), then `deposit`, `withdraw`, `close`.

**Concepts I was unsure about:**

- Why the `vault` account cannot be `init`, only `mut`.
- At what point the `vault` account actually gets initialized.
- What `SystemAccount` means exactly.
- The relationship — or lack of one — between `mut` and initialization.

---

## 2. Design

**Data structures / accounts / state:**

`VaultState` holds only two fields: `vault_bump` and `state_bump`.

> **My own words:** The other PDA, `vault_state`, has a single purpose: storing data, and that
> data is the state bump and the vault bump. This will avoid recomputing the canonical bump for
> each of them, on every instruction.
>
> <sub>*Original (dictated in French): « L'autre PDA, vault_state, lui a une seule utilité : stocker des données, et ces données sont le bump de state et le bump du vault. Ce qui permettra d'éviter de recalculer le bump canonique pour chacun d'eux, pour chaque instruction. »*</sub>

**`initialize` instruction flow:**

1. Compute the rent-exempt amount required for `vault`.
2. Transfer that amount from `user` to `vault` via CPI to the System Program.
3. Write both bumps into `vault_state` using `set_inner`.

**What I understood about the `init` constraint:**

> **My own words:** The `init` constraint makes a CPI call to the System Program and enables
> three distinct things. First, it computes the size of the account based on each of its
> attributes. From that size, we can compute the amount of lamports needed for the account to be
> rent exempt. Finally, it sets the owner of the account. After a quick re-read of the `init`
> constraint, I can see there is a transfer of ownership from the System Program to the program
> that will operate on the account in question through the instruction. Then 8 bytes get written
> for the discriminator; those 8 bytes are written at the start of the array. This makes it
> possible to compare instructions or accounts in order to guarantee their integrity.
>
> <sub>*Original (dictated in French): « La contrainte init fait un appel au CPI vers le System Program et permet trois choses distinctes. D'une part on va calculer la taille du compte selon chacun de ses attributs. À partir de cette taille, on peut calculer le montant de lamports nécessaire pour que l'account soit rent exempt. Enfin, on va définir le owner de l'account. Après une courte relecture de la contrainte init, je vois qu'il y a un transfert de l'ownership du System Program vers le programme qui va exécuter le compte en question au travers de l'instruction. Ensuite il va y avoir une écriture de 8 bytes pour le discriminator ; ces 8 bytes seront écrits au début du tableau. Cela permet de comparer des instructions ou des accounts afin d'assurer leur intégrité. »*</sub>

**Why `vault_state` uses `init` but `vault` does not:**

> **My own words:** For `vault_state`, we use `init` via the `init` constraint because we need to
> compute everything by hand, since it is a struct, a custom state. On the other hand, being an
> account owned by the System Program, the only thing we have to do is compute the amount of
> lamports needed for it to be rent exempt, and then finally perform the transfer.
>
> <sub>*Original (dictated in French): « Pour le vault_state, on fait un init via la contrainte init parce qu'on a besoin de tout calculer à la main, parce que c'est une structure, une state custom. Par contre, en tant qu'account possédé par le système, la seule chose que l'on a à faire c'est de calculer le montant de lamports nécessaire pour que celui-ci soit rent exempt et donc enfin procéder au transfert. »*</sub>

**On the rent computation:**

> **My own words:** The expression that gives the lamport amount is not a lucky accident: it is
> an amount computed for two years of rent so that the account becomes permanently exempt from
> the tax, meaning it is rent exempt. There is a fixed lamport cost per byte; I do not know that
> figure by heart without looking it up, but since that cost is fixed, it is not magic. That
> said, I know this account has no custom data structure.
>
> <sub>*Original (dictated in French): « L'expression qui permet d'obtenir le montant de lamports, ce n'est pas un heureux hasard : c'est un montant qui est calculé pour deux ans de loyer afin que le compte soit définitivement exempté de taxes, c'est-à-dire qu'il soit rent exempt. Il y a un coût fixe de lamports par octet ; je ne connais pas ce montant par cœur si je ne cherche pas la réponse en ligne, mais ce montant étant fixe, ce n'est pas de la magie. Néanmoins je sais que ce compte ne dispose pas de structure de données personnalisées. »*</sub>

**Where I expect this to be hard, and why:**

`withdraw` — I will need the program to sign for the `vault` PDA, and to handle the bounds of
the withdrawn amount.

---

## 3. Security model

**Who can call what, and how it is enforced:**

> **My own words:** The guarantee is that the seeds use the user's key. To derive a PDA's
> address you use the seeds and the bump, and although it is not written in the seeds array,
> the program ID is also part of the derivation.
>
> <sub>*Original (dictated in French): « La garantie est que dans les seeds, on utilise la clé de user. Pour récupérer l'adresse d'un PDA, on utilise les seeds, le bump, et dans les seeds cela n'est pas écrit mais il y a également le program ID du programme. »*</sub>

**How `withdraw` will have to work:**

> **My own words:** When I implement the `withdraw` instruction, the vault account will be
> debited in lamports and those will be credited to the user account. To do that, the
> transaction will have to be signed via the PDA. That works out, because `vault` is a PDA built
> from the seeds `VAULT_SEED` and `user.key()`, which will let the program sign on its behalf.
>
> <sub>*Original (dictated in French): « Lorsque je vais développer l'instruction withdraw, le vault account sera débité en lamports et ceux-ci seront crédités vers l'account user. Mais pour ce faire, il faudra signer la transaction via le PDA. Ça tombe bien parce que vault est un PDA qui contient les seeds VAULT_SEED et user.key(), qui permettront au programme de signer pour lui. »*</sub>

**What an attacker could try:**

> **My own words:** When the user asks to withdraw an `amount` equal to the entire balance, the
> vault account will simply be deleted because it will no longer be rent exempt. However, if the
> user asks to withdraw an `amount` that exceeds the vault balance, the risk here is an
> underflow — meaning the program will exit with an error and therefore the transaction, which
> contains several instructions, will fail. Because a transaction is atomic: every instruction
> must succeed. So before performing the transfer in the `withdraw` instruction, I should check
> that the amount passed as a parameter is lower than the available balance and greater than
> zero, to avoid an underflow as well.
>
> <sub>*Original (dictated in French): « Lorsque l'utilisateur demande à retirer le amount égal à la totalité du solde, le vault account sera simplement supprimé parce qu'il ne sera plus rent exempt. Cependant si l'utilisateur demande à retirer un montant d'amount qui dépasse le solde dans le vault, ici le risque est qu'il y ait un underflow, c'est-à-dire qu'il va y avoir une sortie de programme avec erreur et donc la transaction, qui elle contient plusieurs instructions, va échouer. Parce qu'une transaction est atomique : toutes les instructions doivent réussir. Donc il faudrait vérifier avant de faire le transfert dans l'instruction withdraw que le montant passé en paramètres est inférieur au solde disponible et supérieur à zéro pour éviter un underflow également. »*</sub>

**Trust assumptions I am making:**

- The bump read from `vault_state.vault_bump` is trustworthy, because `vault_state` is itself
  verified by its own `seeds` constraint.
- A PDA's seeds are public, and that is not a vulnerability: security comes from the `Signer`
  check, not from seed secrecy.
- `overflow-checks = true` is present in `[profile.release]` of the workspace `Cargo.toml`.
  This is **not** Cargo's default — without that line, a `u64` underflow would wrap silently
  instead of panicking.

---

## 4. Test plan

<!-- TO BE COMPLETED BY ME, without AI.
     Questions I have to answer myself:
     - Which behaviors of `initialize` need coverage?
     - What happens if I call `initialize` twice for the same user?
     - How do I test that one user cannot touch another user's vault?
     - Which test would fail if I removed the `Signer` constraint from `user`?
     - How do I test the rent-exempt forbidden zone on `withdraw`?
     - What would a failing test teach me, in each case?
-->

**Behaviors I will test:**

**Edge cases and failure paths:**

---

## 5. Log (append, do not rewrite)

### 2026-09-03 — first implementation of `initialize`

**What I did:**
Implemented `initialize`: the `Initialize` accounts struct with `user` (Signer, mut),
`vault_state` (`init`), `vault` (`SystemAccount`, `mut`), and `system_program`. Then the body:
rent-exempt computation, `transfer` CPI, `set_inner` of both bumps.

**What blocked me:**
Not a compilation error — a conceptual misunderstanding. The `vault` account cannot be `init`,
only `mut`, whereas in my head that account is not initialized, since handling that is the whole
point of the instruction.

**My initial hypothesis (wrong, kept as-is):**

> **My own words:** I know that `SystemAccount` by definition makes the account in question tied
> to the program. But the program itself still has to be initialized, doesn't it? My theory:
> when you deploy a program to a cluster, the accounts it might manipulate are created at deploy
> time. But I think my theory is wrong. Because if you look at the `vault` account, it has a seed
> related to the payer. So my theory falls apart. So I don't understand at what point the `vault`
> account is initialized.
>
> <sub>*Original (dictated in French): « Je sais que SystemAccount par définition fait en sorte que l'account en question est lié au programme. Mais le programme en lui-même doit tout de même être initialisé, n'est-ce pas ? Ma théorie : lorsque l'on déploie un programme sur un cluster, lors du déploiement les comptes qu'il pourrait manipuler sont créés. Mais je pense que ma théorie est fausse. Parce que si on regarde l'account vault, celui-ci a un seed en rapport avec le payeur. Donc ma théorie tombe à l'eau. Donc je ne comprends pas à quel moment l'account vault est-il initialisé ? »*</sub>

**What I rejected on my own:**
I invalidated my own deploy-time theory by noticing that a seed derived from `user.key()` cannot
be known at deploy time — there are as many possible vaults as there are users.

---

### 2026-09-03 — after mentoring session

**Question I asked:**
At what point does the `vault` account get initialized, and why `mut` instead of `init`?

**What I figured out myself during the session (the breakthrough):**

> **My own words:** `mut` allows mutability, as its name suggests, but this `mut` constraint does
> not necessarily mean the account has been initialized or not. There is no relation between the
> two. It is really just a constraint on what you are allowed to do with that account.
>
> <sub>*Original (dictated in French): « Le mut permet la mutabilité, comme son nom l'indique, mais cette contrainte mut ne veut pas forcément dire que le compte a été initialisé ou non. Cela n'a pas de rapport. En fait, c'est juste une contrainte sur les choses que l'on peut faire pour cet account. »*</sub>

**What changed in my understanding:**

1. **`init` and `mut` are not on the same axis.** `init` is about *creation*, `mut` is about
   *write permission*. An account can be `mut` without existing yet. Solana requires every
   account to be declared writable or read-only **in the transaction message, before execution**,
   to allow parallelization (Sealevel). Without `mut`, the `transfer` would fail with "account is
   not writable", independently of any initialization question.

2. **`init` decomposes into three operations**: *allocate* (reserve space), *assign* (change the
   owner), *transfer* (fund the rent). My vault only needs **`transfer`**. That is why I don't
   use `init`: it was **too powerful**, it did two things too many.

3. **`init` sets MY program as owner, not the System Program.** I had said the opposite before
   correcting myself on re-reading. That is precisely the point: an account owned by my program
   is an account my program can write data into.

4. **`SystemAccount<'info>` is not a statement of intent, it is a constraint assertion** checked
   at runtime: "the owner of this account MUST be the System Program." I had understood the exact
   opposite. So `init` and `SystemAccount` are **contradictory** on the same account.

5. **What makes an account exist is holding more than 0 lamports.** I had conflated "existing"
   and "being rent exempt." Rent exemption is the threshold above which an account can no longer
   be purged; at 0 lamports the account is erased from the ledger. So **the `transfer` itself is
   what creates the vault**: sending lamports to an address that never held anything brings it
   into existence. There is never a separate "initialization moment" for the vault.

6. **The discriminator is 8 bytes (64 bits), not 8 bits.** It is the first 8 bytes of
   `sha256("account:VaultState")`. Its precise function is preventing **type confusion**: if two
   structs had the same size, nothing would distinguish them. It is a security control, not just
   a tag.

7. **`anchor deploy` creates only two accounts**: the program account and its programdata. Zero
   state accounts.

8. **`self.vault.data_len()` evaluates to `0`**, but `minimum_balance(0)` is not 0 — it is
   **890,880 lamports** (~0.00089 SOL), because every account carries 128 bytes of intrinsic
   metadata and rent is computed over `128 + data_len`. To revisit: writing `minimum_balance(0)`
   would be more explicit than `data_len()` on an account that does not exist yet.

9. **Why this design was mandatory and not merely viable** — the rule I had missed: *only the
   program that owns an account may decrease its lamports*. So if I had used `init` on the vault,
   my program would have become its owner, and the System Program would have **refused** the
   `system_program::transfer` in `withdraw`. Second lock: `system_instruction::transfer` requires
   the `from` account to hold zero data, and `init` would allocate some. So `init` on the vault
   would break `withdraw` in two independent ways.

10. **The security chain behind `seeds`** — the guarantee that `vault_state.vault_bump` is
    trustworthy does not come from seed secrecy (seeds are public by nature, `user.key()` is a
    public key, and a program never sees a private key). It comes from two coupled links:
    `user: Signer` proves the caller's identity, and `seeds = [...]` makes Anchor recompute the
    PDA and **compare** it against the address that was passed in (`ConstraintSeeds` otherwise).
    An attacker can pass a victim's `vault_state`, but cannot sign in their place, so the
    recomputed PDA will not match. **`seeds` without `Signer` is worthless.**

11. **The rent-exempt forbidden zone**, which I had not spotted for `withdraw`: withdrawing
    everything (→ 0 lamports) is allowed and deletes the account; leaving ≥ 890,880 is fine; but
    leaving a non-zero balance **between 1 and 890,879** causes the **entire transaction to be
    rejected** with `InsufficientFundsForRent`. `withdraw` will have to handle that case
    explicitly.

12. **`overflow-checks = true` is not Cargo's release default** (the default is `false`). The
    Anchor template adds it. Without that line, `0u64 - 1` does not panic: it wraps silently to
    `u64::MAX` — one of the most classic exploit classes on Solana. My protection comes from a
    config line, not from the language.

**A mentor error I was right to push back on:**
The mentor claimed that `CpiContext::new(self.system_program.key(), ...)` was a compilation bug,
based on the Anchor 0.x API. I held my position and asked for verification. In **Anchor 1.1.2**,
`context.rs:188` reads `pub fn new(program_id: Pubkey, accounts: T)` — it really is a `Pubkey`,
so `.key()` is correct. The fields of CPI account structs, on the other hand, remain
`AccountInfo` (`Transfer { from: AccountInfo, to: AccountInfo }`), so `.to_account_info()` is
still correct there. `cargo check` passes with no errors and no warnings.
**Lesson: verify the version before accepting an API correction, including from an AI.**

---

### 2026-09-04 — `deposit`, `withdraw`, `close` and the code review

> Quotes below were dictated in English; only transcription artifacts have been cleaned.

**What I did:**
Implemented `deposit` and `withdraw`, then rebuilt `close`. Submitted all three for review.

**What broke — three bugs, all found by review, none found by the compiler:**

1. **I declared the vault as `Account<'info, VaultState>` in both `deposit` and `withdraw`**, where
   `initialize` correctly uses `SystemAccount<'info>`. The code compiled and would have failed on
   **every single call**, because `Account<'info, T>` asserts three things my vault satisfies none
   of: owner is my program (it is the System Program), the first 8 bytes are `VaultState`'s
   discriminator (there are 0 bytes of data), and the remaining bytes deserialize into `T` (there
   is nothing to deserialize). The runtime error is `AccountOwnedByWrongProgram`.

   This was the exact concept of my 09-03 session, and I got it wrong in code the very next day.
   Understanding a concept in conversation is not the same as having it available while writing.

2. **I read the bump from the wrong account** — `self.vault.vault_bump` instead of
   `self.vault_state.vault_bump`. This only compiled *because* of bug 1: since I had wrongly typed
   the vault as `VaultState`, the field access type-checked. The wrong type did not merely fail to
   help me — it **silenced an independent second bug**. Fixing bug 1 turned this line into a
   compile error immediately.

3. **My rent-exemption guard could not do its job.** I wrote
   `require!(vault_balance - amount >= rent_exempt, ...)`. The subtraction is evaluated *inside*
   the condition, before the comparison, so when `amount > vault_balance` it panics while
   computing the condition — the `require!` never gets to reject anything. A guard cannot reject
   the input that destroys the guard.

   With `overflow-checks = true` the caller gets `ProgramFailedToComplete` instead of my typed
   error. With `overflow-checks = false` the guard is **bypassed entirely**:
   `890_880 - u64::MAX` wraps to `890_881`, which is `>= 890_880`, so the check passes. Verified
   by running it. One config flag separated "confusing" from "exploitable".

**Two errors of mine in the review itself, worth recording:**

- I merged two different triples. `init`'s three operations are *allocate / assign / transfer*.
  `Account<'info, T>`'s three validations are *owner / discriminator / deserialize*. `init`
  **creates**; `Account<T>` **validates**. Different jobs, different lists.
- I stated that an underflow "wraps silently" while `overflow-checks = true` — inverting my own
  point 12 from the day before. With the flag on it **panics**; without it, it wraps.

---

**On the division of labour between `withdraw` and `close`:**

> **My own words:** Am I meant to work out that I should close the account after `withdraw` to
> refund the rent to the payer? To close the vault there will be the `close` instruction.

**Reconciliation — this supersedes point 11 of the 09-03 entry, which said the opposite.**
Point 11 claimed that withdrawing everything down to 0 is allowed. My code forbids it, and that
is **deliberate**: `withdraw` never breaks rent exemption, and `close` is the only path that
empties the vault. Leaving point 11 as written, per the append-only rule.

---

**On the ordering of the `close` constraint:**

> **My own words:** Inside the `close` instruction we have to withdraw everything from the vault
> balance before being able to close the account. After the `close` function block has ended, the
> `close` constraint will be run and the account will be closed internally — and by internally I
> mean that because we are using an Anchor constraint, Anchor will do that for us. On the other
> hand, if we withdraw using the transfer, we drain the vault to exactly zero, and in Solana the
> account will be deleted.

Both correct. The `close` constraint executes in Anchor's **exit phase**, after the instruction
body returns, which is why my body can still read `self.vault_state.vault_bump` at that point.

**Where I got stuck:**

> **My own words:** This is where I am struggling a little, because if the account will be deleted
> automatically, maybe the `close` constraint is not really necessary. Or maybe I misunderstood
> and the `close` constraint will not close the account by itself, because it will be closed
> automatically once the vault drains to exactly zero — so the `close` constraint is just deciding
> who to return the rent-exempt amount to.

**What changed in my understanding — there are TWO accounts, not one:**

I was reasoning as if a single account were being closed by two competing mechanisms. In fact
`close = user` sits on `vault_state`, **not** on `vault`. They are different accounts with
different owners, and each needs its own closing mechanism:

|                  | `vault`                  | `vault_state`                    |
| ---------------- | ------------------------ | -------------------------------- |
| Type             | `SystemAccount`          | `Account<'info, VaultState>`     |
| Owner            | System Program           | **my program**                   |
| Data             | 0 bytes                  | 8 (discriminator) + 2 (bumps)    |
| Rent it holds    | 890,880                  | **960,480**                      |
| Closed by        | my manual CPI `transfer` | Anchor's `close = user`          |

They are not redundant. Remove `close = user` and `vault_state` survives forever with **960,480
lamports locked inside it**, which the user paid during `initialize` and would never recover.

**And the reason the two mechanisms differ is the same rule as everything else:**

> **My own words:** I abandoned `CloseAccount` because Anchor will do that for us for the
> `vault_state` account — only my program is the owner of `vault_state`, so Anchor can do it on my
> behalf. I can't do that for the vault account, because if I had added the `close` constraint to
> the vault account it would fail, since the vault account is owned by the System Program and only
> the owner can debit the account's lamports.

*Only the owner program can debit an account.* That single rule has now decided four separate
design questions in this program: why the vault is a `SystemAccount`, why `init` on it would break
`withdraw`, why `withdraw` needs `new_with_signer`, and why `close` needs two different closing
paths in one instruction.

**Two corrections to that answer:**

- `CloseAccount` is not Anchor's — it belongs to **SPL Token** (`anchor_spl::token::CloseAccount`)
  and closes *token accounts* owned by the *Token Program*. Neither of my accounts is a token
  account, and `anchor_spl` is not even in my `Cargo.toml`, so it could never have compiled.
- `close = user` on a `SystemAccount` would not "fail at runtime" — it **would not compile**.
  `AccountsClose` is implemented for `Account`, `LazyAccount`, `AccountLoader`, `InterfaceAccount`,
  `Box<T>` and `Option<T>`. `SystemAccount` implements `AccountsExit` but deliberately **not**
  `AccountsClose`.

---

**On calling `close` twice:**

> **My own words:** If someone calls `close` twice, the first time it will work, but the second
> time we will get an error saying the account isn't allocated.

Correct instinct; the precise name is **`AccountNotInitialized`**. Anchor's `close`
(`anchor-lang-1.1.2/src/common.rs:7-17`) does four things:

```rust
sol_destination.add_lamports(info.lamports())?;   // all lamports → user
**info.lamports.borrow_mut() = 0;                 // account now holds 0
info.assign(&system_program::ID);                 // owner → System Program
info.resize(0)                                    // data truncated to 0 bytes
```

The end state — owner is the System Program, lamports are 0 — is exactly the branch at
`account.rs:315` that returns `AccountNotInitialized`. Step 3 is the important one: Anchor hands
the account **back to the System Program**, which is what makes reinitialization attacks fail and
leaves a closed account indistinguishable from one that never existed.

---

**On checked arithmetic:**

> **My own words:** You always have to use `.checked_add` / `.checked_sub` instead of the `+` and
> `-` operators. `checked_sub` will automatically check the value; if it would underflow there
> will be no program crash — instead our domain `VaultError` is triggered.

**The limit on "always":** use checked arithmetic on any value I don't control — user-supplied
parameters, lamport balances, anything read from an account. Plain operators are fine for
compile-time constants or values whose bounds I have already proven; `checked_*` everywhere adds
noise that hides the places that genuinely matter.

**And the real motivation is not "avoid the wrap."** With `overflow-checks = true` a panic is
*safe* — nothing is silently corrupted. The two actual reasons are: (1) a panic gives the caller
`ProgramFailedToComplete` instead of my typed `VaultError`, and (2) my panic-safety would depend
on one line in `Cargo.toml` that anyone could flip, whereas `checked_sub` is correct regardless of
the build profile. **Never let a security property depend on a build flag.**

---

**The lesson underneath all three bugs — the type system:**

Three times in one day, the *correct* type either caught a mistake at build time or would have
prevented it: the wrong vault type silenced the wrong-bump bug; fixing it turned that bug into an
immediate compile error; and `close` on a `SystemAccount` is refused by the compiler rather than
by the runtime. Anchor did not encode "only owners can debit" as a runtime check I have to hope a
test triggers — it encoded it in the **type system, so the mistake cannot ship**. A wrong type
annotation does not merely fail to help me; it actively disables the compiler as a safety net.

**What I will do next, without AI:**

- [x] Replace `data_len()` with `0` in the rent-exempt computation, for explicitness.
- [ ] Write section 4 (test plan) of this file.
- [x] Implement `deposit`.
- [x] Implement `withdraw`, handling the rent-exempt forbidden zone and the amount bounds.
- [ ] Run the thought experiment to completion: remove `Signer` from `user` and write the test that proves the exploit, to verify I really understand the `Signer` + `seeds` chain.
