## Assessing

`/rk project:vega assess advisory:GHSA-xxxx-xxxx-xxxx` means: decide whether that advisory
really affects the project, and record the verdict with `rekall_assess_vulnerability`. A scan only
knows the project declares a vulnerable version; whether the flaw can be reached is in the code.

`/rk project:vega assess`, with no `advisory:` term, means the same for every advisory reaching the
project that is still `not assessed`: one verdict each.

1. Drop the `assess` term and the `advisory:` term, and call `rekall_context` with the anchors that
   are left. You change no code and no checklist.
2. With no `advisory:` term, call `rekall_vulnerabilities` with the project anchor alone and take
   every advisory it lists as `not assessed`, in that order. If there is none, say so and stop. Then
   work steps 3 to 5 for each one, recording each verdict before starting the next.
3. Call `rekall_vulnerabilities` with the project anchor and `advisory` set to the GHSA id. It gives
   the package and version, the vulnerable versions, the first safe one, the functions that carry the
   flaw when GitHub names them, the advisory's own description and any earlier conclusion and
   reasoning.
4. Find where the project uses the package: its imports, the calls into it, the configuration that
   turns a vulnerable feature on, and the inputs that reach those calls. Follow them far enough to say
   whether the vulnerable code runs, and on what. Advisories on the same package share this search:
   do it once and judge each against its own vulnerable functions.
5. Call `rekall_assess_vulnerability` with the project anchor, the advisory, a `verdict`, the
   `reasoning` and the `conclusion`. `not_affected` only when the code shows the vulnerable path is
   never taken (the function is never called, the feature is off, no input reaches it); `affected`
   when it is, or when you cannot show it is not.
   - The `reasoning` says what the flaw needs to run and names the files and functions that decide
     whether the project gives it that.
   - The `conclusion` is one or two plain sentences on why this project is or is not affected, in
     terms of what the project does, not what the flaw does: `Rekall never unpacks archives; it only
     reads lockfiles as text.` For `not_affected`, it names the feature or input the project does not
     use, so whoever reads it knows what not to start using while the package stays at this version.
6. Say in two or three lines what you concluded and why; for several, one line per advisory with its
   verdict. Then stop: fixing is a task the console opens.
